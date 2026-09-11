use prost::Message;
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};
use tokio::time::sleep;

mod proto {
    include!(concat!(env!("OUT_DIR"), "/demo.rs"));
}
const LANGUAGES: [&str; 4] = ["rust", "python", "cpp", "typescript"];
const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/..");

async fn client() -> zenoh::Result<()> {
    let seconds: u32 = std::env::var("DEMO_SECONDS")
        .unwrap_or_else(|_| "30".into())
        .parse()?;
    if seconds == 0 {
        return Err("DEMO_SECONDS must be positive".into());
    }
    let session = zenoh::open(zenoh::Config::from_file(format!(
        "{ROOT}/config/client.json5"
    ))?)
    .await?;
    let subscriber = session.declare_subscriber("demo/example/**").await?;
    let receiver = tokio::spawn(async move {
        while let Ok(sample) = subscriber.recv_async().await {
            let result = proto::Result::decode(sample.payload().to_bytes().as_ref())?;
            let expected = match result.language.as_str() {
                "rust" => ("rust.sequence", vec![f64::from(result.sequence)]),
                "python" => ("torch.matmul", vec![4., 5.]),
                "cpp" => ("opencv.threshold", vec![2.]),
                "typescript" => ("node.sum", vec![6.]),
                _ => return Err("Unknown protobuf language".into()),
            };
            if sample.key_expr().as_str() != format!("demo/example/{}", result.language)
                || result.operation != expected.0
                || result.values != expected.1
            {
                return Err(format!("Unexpected result: {result:?}").into());
            }
            println!(
                "RECV {} {} {} {:?}",
                sample.key_expr(),
                result.language,
                result.operation,
                result.values
            );
        }
        Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
    });
    println!("READY rust");
    if let Ok(gate) = std::env::var("DEMO_START_FILE") {
        let deadline = Instant::now() + Duration::from_secs(60);
        while !Path::new(&gate).exists() {
            if Instant::now() > deadline {
                return Err("Startup barrier timed out".into());
            }
            sleep(Duration::from_millis(50)).await;
        }
    }
    for sequence in 0..seconds {
        sleep(Duration::from_secs(1)).await;
        let result = proto::Result {
            language: "rust".into(),
            sequence,
            operation: "rust.sequence".into(),
            values: vec![f64::from(sequence)],
        };
        session
            .put("demo/example/rust", result.encode_to_vec())
            .await?;
    }
    sleep(Duration::from_millis(300)).await;
    session.close().await?;
    receiver.await??;
    Ok(())
}

struct Processes(Vec<(String, Child)>);
impl Processes {
    fn start(
        &mut self,
        name: &str,
        command: &mut Command,
        seconds: u32,
        gate: &Path,
    ) -> zenoh::Result<()> {
        let log = File::create(format!("{ROOT}/.logs/{name}.log"))?;
        let child = command
            .current_dir(ROOT)
            .env("DEMO_SECONDS", seconds.to_string())
            .env("DEMO_START_FILE", gate)
            .stdout(log.try_clone()?)
            .stderr(Stdio::from(log))
            .spawn()?;
        self.0.push((name.into(), child));
        Ok(())
    }
    fn check_running(&mut self) -> zenoh::Result<()> {
        for (name, child) in &mut self.0 {
            if let Some(status) = child.try_wait()? {
                return Err(format!("{name} exited early ({status}); see .logs/{name}.log").into());
            }
        }
        Ok(())
    }
}
impl Drop for Processes {
    fn drop(&mut self) {
        for (_, child) in &mut self.0 {
            let _ = child.kill();
        }
        for (_, child) in &mut self.0 {
            let _ = child.wait();
        }
    }
}
struct Gate(PathBuf);
impl Drop for Gate {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

async fn orchestrate(seconds: u32) -> zenoh::Result<()> {
    // Hold both reservations until just before spawning the bridge.
    let tcp = std::net::TcpListener::bind("127.0.0.1:7447")?;
    let ws = std::net::TcpListener::bind("127.0.0.1:10000")?;
    fs::create_dir_all(format!("{ROOT}/.logs"))?;
    let gate = Gate(PathBuf::from(format!(
        "{ROOT}/.logs/start-{}",
        std::process::id()
    )));
    if gate.0.exists() {
        fs::remove_file(&gate.0)?;
    }
    let mut processes = Processes(Vec::new());
    drop((tcp, ws));
    processes.start(
        "bridge",
        Command::new(format!("{ROOT}/.zenoh/bridge/zenoh-bridge-remote-api")).args([
            "--listen",
            "tcp/127.0.0.1:7447",
            "--ws-port",
            "127.0.0.1:10000",
            "--no-multicast-scouting",
        ]),
        seconds,
        &gate.0,
    )?;
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        processes.check_running()?;
        if tokio::net::TcpStream::connect("127.0.0.1:10000")
            .await
            .is_ok()
        {
            break;
        }
        if Instant::now() > deadline {
            return Err("Bridge startup timed out".into());
        }
        sleep(Duration::from_millis(100)).await;
    }
    processes.start(
        "python",
        &mut Command::new(format!("{ROOT}/python/.venv/bin/zenoh-python")),
        seconds,
        &gate.0,
    )?;
    processes.start(
        "cpp",
        &mut Command::new(format!("{ROOT}/cpp/build/zenoh-cpp")),
        seconds,
        &gate.0,
    )?;
    processes.start(
        "typescript",
        Command::new("node").arg(format!("{ROOT}/ts/src/demo.ts")),
        seconds,
        &gate.0,
    )?;
    processes.start(
        "rust",
        Command::new(std::env::current_exe()?).arg("--client"),
        seconds,
        &gate.0,
    )?;
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        processes.check_running()?;
        if LANGUAGES.iter().all(|name| {
            fs::read_to_string(format!("{ROOT}/.logs/{name}.log"))
                .is_ok_and(|log| log.contains(&format!("READY {name}")))
        }) {
            break;
        }
        if Instant::now() > deadline {
            return Err("Client readiness timed out; see .logs/".into());
        }
        sleep(Duration::from_millis(100)).await;
    }
    // Allow subscription propagation before releasing every publisher together.
    sleep(Duration::from_millis(500)).await;
    fs::write(&gate.0, b"start")?;
    println!("All clients ready; exchanging protobuf results for {seconds}s. Logs: {ROOT}/.logs");
    let deadline = Instant::now() + Duration::from_secs(u64::from(seconds) + 15);
    loop {
        let mut done = true;
        for (name, child) in &mut processes.0 {
            match child.try_wait()? {
                Some(status) if name == "bridge" || !status.success() => {
                    return Err(format!("{name} exited ({status}); see .logs/{name}.log").into());
                }
                None if name != "bridge" => done = false,
                _ => (),
            }
        }
        if done {
            break;
        }
        if Instant::now() > deadline {
            return Err("Clients timed out; see .logs/".into());
        }
        sleep(Duration::from_millis(100)).await;
    }
    for receiver in LANGUAGES {
        let log = fs::read_to_string(format!("{ROOT}/.logs/{receiver}.log"))?;
        for sender in LANGUAGES {
            if sender != receiver && !log.contains(&format!("RECV demo/example/{sender} {sender} "))
            {
                return Err(
                    format!("Missing {sender} -> {receiver}; see .logs/{receiver}.log").into(),
                );
            }
        }
        println!("PASS {receiver}: decoded results from all three other languages");
    }
    println!("PASS: all 12 protobuf routes and numerical results verified.");
    Ok(())
}

#[tokio::main]
async fn main() -> zenoh::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args == ["--client"] {
        return client().await;
    }
    let seconds = match args.as_slice() {
        [] => 10,
        [flag, value] if flag == "--seconds" => value.parse::<u32>()?,
        _ => return Err("Usage: rust [--seconds N | --client]".into()),
    };
    if seconds < 3 {
        return Err("--seconds must be at least 3".into());
    }
    tokio::select! {
        result = orchestrate(seconds) => result,
        signal = tokio::signal::ctrl_c() => { signal?; Err("Interrupted; child processes stopped".into()) }
    }
}

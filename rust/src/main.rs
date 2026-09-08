use std::time::Duration;

#[tokio::main]
async fn main() -> zenoh::Result<()> {
    let config = zenoh::Config::from_file(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../config/client.json5"
    ))?;
    let session = zenoh::open(config).await?;
    let subscriber = session.declare_subscriber("demo/example/**").await?;
    let receiver = tokio::spawn(async move {
        while let Ok(sample) = subscriber.recv_async().await {
            println!(
                "RECV {} {}",
                sample.key_expr(),
                sample.payload().try_to_string().unwrap_or_default()
            );
        }
    });
    println!("READY rust");
    let seconds: u64 = std::env::var("DEMO_SECONDS")
        .unwrap_or_else(|_| "30".into())
        .parse()?;
    for i in 0..seconds {
        tokio::time::sleep(Duration::from_secs(1)).await;
        session
            .put("demo/example/rust", format!("Hello from Rust! {i}"))
            .await?;
    }
    session.close().await?;
    receiver.abort();
    Ok(())
}

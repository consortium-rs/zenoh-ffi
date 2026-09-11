# Zenoh in four languages

Rust orchestrates a local Zenoh bridge and Rust, Python, C++, and TypeScript
clients. All four publish binary protobuf messages on `demo/example/<language>`
and decode messages from `demo/example/**`. TypeScript runs directly in Node.

The shared schema is [`proto/demo.proto`](proto/demo.proto): each `Result` carries
its language, sequence number, operation, and numeric values.

| Client | Small computation | Published values |
| --- | --- | --- |
| Rust | Sequence counter | `[sequence]` |
| Python | CPU PyTorch matrix multiplication | `[4, 5]` |
| C++ | OpenCV threshold of a 2×2 grayscale image, then count bright pixels | `[2]` |
| TypeScript / Node | Sum of `[1, 2, 3]` | `[6]` |

No model, image download, GPU, browser, or camera is required.

## Setup

The setup script is tested on macOS Apple Silicon. Install Rust/Cargo, Python 3.10+, uv, Vite+
(`vp`), Node 24.20+ (tested), CMake, and Xcode Command Line Tools. Install the
protobuf compiler and C++ runtime together:

```sh
brew install protobuf
bash scripts/setup.sh
```

Setup downloads Zenoh 1.10.1 and builds OpenCV 4.13.0 locally with only `core` and
`imgproc`. The first build takes a few minutes. Dependencies live in `.zenoh/`,
`python/.venv/`, and `ts/node_modules/`. Protobuf is discovered through CMake;
if using a nonstandard installation, add its `bin` directory to `PATH` and set
`CMAKE_PREFIX_PATH` to its installation prefix.

C++ requires C++26 mode, tested with AppleClang 21 and CMake 3.30. The existing
fallback checks `-std=c++26` when CMake does not model the compiler's C++26 support.
The starter Conan recipe is not used.

Rust generates protobuf bindings with `prost-build` during compilation; CMake
generates the C++ bindings. Setup generates Python bindings with `protoc` before
installing the Python package. Node loads the same `.proto` using `protobufjs`.
After changing the schema, rerun setup to regenerate and rebuild all clients.
The Python protobuf runtime must be at least as new as the compiler's generated
Python version (tested with protoc 36.1 / Python protobuf 7.36.1).

## Run

From the repository root:

```sh
cargo run --locked --manifest-path rust/Cargo.toml -- --seconds 10
```

Rust starts the bridge and all clients, waits for every `READY` marker, releases
all publishers together, verifies all **12 cross-language protobuf routes**, and
checks the numerical results received by the Rust client. Failures return a
nonzero status. Child processes are stopped on completion, failure, or Ctrl-C.
Logs are in `.logs/`. `--seconds` must be at least 3 and defaults to 10.

Ports **7447** (Zenoh TCP) and **10000** (WebSocket) must be free; both bind to
`127.0.0.1`. Multicast discovery is disabled. TypeScript uses the remote-API
WebSocket bridge; native clients use `config/client.json5`.

The old `python3 scripts/run.py --seconds 10` command remains a compatibility
entry point that executes the Rust orchestrator.

## Run clients individually

Start the bridge in a separate terminal:

```sh
.zenoh/bridge/zenoh-bridge-remote-api --listen tcp/127.0.0.1:7447 --ws-port 127.0.0.1:10000 --no-multicast-scouting
```

Then run each client in its own terminal:

```sh
cargo run --locked --manifest-path rust/Cargo.toml -- --client
python/.venv/bin/zenoh-python
cpp/build/zenoh-cpp
(cd ts && vp run start)
```

Clients run for 30 seconds by default; set `DEMO_SECONDS` to change that. Each
prints `READY <language>` and `RECV <key> <language> <operation>`. Rust, Python,
and Node also print decoded values. `DEMO_START_FILE` is the internal startup
barrier managed by the orchestrator; leave it unset for standalone clients.

## Validation

```sh
cargo fmt --check --manifest-path rust/Cargo.toml
cargo build --locked --manifest-path rust/Cargo.toml
cmake --build cpp/build
(cd ts && vp check && vp test && vp run build)
cargo run --locked --manifest-path rust/Cargo.toml -- --seconds 10
```

The integration runner verifies real binary interoperability and computed values;
the starter TypeScript unit test covers only the starter library function.
Node may print an experimental WebAssembly warning from the Zenoh dependency.

VS Code uses `cpp/build/compile_commands.json` for compiler and dependency paths.
Setup generates it; regenerate with `cmake -S cpp -B cpp/build`. If diagnostics
are stale, run **C/C++: Reset IntelliSense Database** then **Developer: Reload
Window**.

API references: [protobuf](https://protobuf.dev/getting-started/),
[PyTorch matmul](https://docs.pytorch.org/docs/stable/generated/torch.matmul.html),
[OpenCV thresholding](https://docs.opencv.org/4.13.0/d7/d4d/tutorial_py_thresholding.html),
[Zenoh Rust](https://github.com/eclipse-zenoh/zenoh),
[Python](https://github.com/eclipse-zenoh/zenoh-python),
[C++](https://github.com/eclipse-zenoh/zenoh-cpp), and
[TypeScript](https://github.com/eclipse-zenoh/zenoh-ts).

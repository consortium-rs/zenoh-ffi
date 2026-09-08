# Zenoh in four languages

Rust, Python, C++, and TypeScript each subscribe to `demo/example/**` and publish
one greeting per second on `demo/example/<language>` using Zenoh 1.10.1.

## Run everything

After completing the setup below, run this from the repository root:

```sh
python3 scripts/run.py
```

This starts a local Zenoh remote-API bridge and all four clients, runs them for
10 seconds, checks all **12 cross-language message routes**, and stops its processes.
It exits nonzero if a client fails or any route is missing. Logs are in `.logs/`.
For a longer run (Ctrl-C also stops the processes):

```sh
python3 scripts/run.py --seconds 60
```

Ports **7447** (Zenoh TCP) and **10000** (WebSocket) must be free. Both listen only
on `127.0.0.1`. No multicast discovery or separate router is needed.

## Set up or rebuild

The setup script supports macOS (Apple Silicon or Intel). Prerequisites: Rust/Cargo,
Python 3.10+, uv, Vite+ (`vp`), Node 24.20+ (the tested version), CMake, and Xcode
Command Line Tools. Node runs TypeScript directly and supplies the WebSocket API.

```sh
bash scripts/setup.sh
```

Dependencies live in `.zenoh/`, `python/.venv/`, and `ts/node_modules/`; no Zenoh
system installation is needed. The setup script also repairs the macOS install
name in the released C library and signs the modified library locally.
The starter Conan recipe is not used; CMake consumes the official Zenoh C release
and C++ headers directly.

## Run each language separately

Run these commands from the repository root, each in a separate terminal.
Start the bridge first:

```sh
.zenoh/bridge/zenoh-bridge-remote-api --listen tcp/127.0.0.1:7447 --ws-port 127.0.0.1:10000 --no-multicast-scouting
```

Then start the four clients while the bridge is running:

```sh
cargo run --locked --manifest-path rust/Cargo.toml
```

```sh
python/.venv/bin/zenoh-python
```

```sh
cpp/build/zenoh-cpp
```

```sh
cd ts
vp run start
```

Clients run for 30 seconds by default; set `DEMO_SECONDS=60` before a command to
change that. Each prints `READY <language>` and `RECV <key> <payload>`.
The native clients share `config/client.json5`; TypeScript connects to the bridge
at `ws://127.0.0.1:10000`.

## Validation

```sh
cargo fmt --check --manifest-path rust/Cargo.toml
cargo build --locked --manifest-path rust/Cargo.toml
cmake --build cpp/build
(cd ts && vp check && vp test && vp run build)
python3 scripts/run.py
```

The integration runner is the messaging test; the existing TypeScript unit test
only covers the starter library function. Node may print an experimental
WebAssembly warning from the Zenoh package's dependency; message exchange works
on the tested Node 24.20.0 runtime.

Official APIs: [Rust](https://github.com/eclipse-zenoh/zenoh),
[Python](https://github.com/eclipse-zenoh/zenoh-python),
[C++](https://github.com/eclipse-zenoh/zenoh-cpp), and
[TypeScript and its remote-API bridge](https://github.com/eclipse-zenoh/zenoh-ts).

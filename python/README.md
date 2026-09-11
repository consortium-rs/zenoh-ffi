# Python PyTorch client

Run `bash scripts/setup.sh` from the repository root to generate protobuf bindings
and install dependencies, then run `python/.venv/bin/zenoh-python` with the bridge
running. The client performs a tiny CPU matrix multiplication yielding `[4, 5]`,
publishes the result as protobuf, and decodes results from all four languages.
See the root README for Rust orchestration and validation.

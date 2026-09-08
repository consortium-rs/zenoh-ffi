import os
from pathlib import Path
import time

import zenoh


def main() -> None:
    config = zenoh.Config.from_file(str(Path(__file__).resolve().parents[3] / "config/client.json5"))
    with zenoh.open(config) as session:
        with session.declare_subscriber(
            "demo/example/**",
            lambda sample: print(f"RECV {sample.key_expr} {sample.payload.to_string()}", flush=True),
        ):
            print("READY python", flush=True)
            for i in range(int(os.environ.get("DEMO_SECONDS", "30"))):
                time.sleep(1)
                session.put("demo/example/python", f"Hello from Python! {i}")

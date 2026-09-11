import os
from pathlib import Path
import time

import torch
import zenoh

from .demo_pb2 import Result


def main() -> None:
    seconds = int(os.environ.get("DEMO_SECONDS", "30"))
    if seconds < 1:
        raise ValueError("DEMO_SECONDS must be positive")
    # A tiny CPU inference: [1, 2, 3] @ [[1, 0], [0, 1], [1, 1]] = [4, 5].
    with torch.inference_mode():
        values = (torch.tensor([[1., 2., 3.]]) @
                  torch.tensor([[1., 0.], [0., 1.], [1., 1.]])).flatten().tolist()

    def receive(sample):
        result = Result.FromString(bytes(sample.payload))
        print(f"RECV {sample.key_expr} {result.language} {result.operation} "
              f"{list(result.values)}", flush=True)

    config = zenoh.Config.from_file(str(Path(__file__).resolve().parents[3] / "config/client.json5"))
    with zenoh.open(config) as session:
        with session.declare_subscriber("demo/example/**", receive):
            print("READY python", flush=True)
            if gate := os.environ.get("DEMO_START_FILE"):
                deadline = time.monotonic() + 60
                while not Path(gate).exists():
                    if time.monotonic() > deadline:
                        raise TimeoutError("Startup barrier timed out")
                    time.sleep(0.05)
            for i in range(seconds):
                time.sleep(1)
                result = Result(language="python", sequence=i, operation="torch.matmul", values=values)
                session.put("demo/example/python", result.SerializeToString())

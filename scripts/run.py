#!/usr/bin/env python3
"""Run the bridge and four clients, then verify every cross-language route."""
import argparse
import os
from pathlib import Path
import shutil
import socket
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--seconds', type=int, default=10)
    args = parser.parse_args()
    if args.seconds < 3:
        parser.error('--seconds must be at least 3')
    for port in (7447, 10000):
        with socket.socket() as sock:
            if sock.connect_ex(('127.0.0.1', port)) == 0:
                raise RuntimeError(f'Port {port} is in use; stop the existing service first.')
    logs = ROOT / '.logs'
    logs.mkdir(exist_ok=True)
    commands = {
        'bridge': [str(ROOT / '.zenoh/bridge/zenoh-bridge-remote-api'), '--listen',
                   'tcp/127.0.0.1:7447', '--ws-port', '127.0.0.1:10000', '--no-multicast-scouting'],
        'rust': [str(ROOT / 'rust/target/debug/rust')],
        'python': [str(ROOT / 'python/.venv/bin/zenoh-python')],
        'cpp': [str(ROOT / 'cpp/build/zenoh-cpp')],
        'typescript': [shutil.which('node') or 'node', str(ROOT / 'ts/src/demo.ts')],
    }
    processes, handles = {}, []
    env = dict(os.environ, DEMO_SECONDS=str(args.seconds))

    def start(name):
        handle = (logs / f'{name}.log').open('w')
        handles.append(handle)
        processes[name] = subprocess.Popen(commands[name], cwd=ROOT, env=env,
                                           stdout=handle, stderr=subprocess.STDOUT)

    try:
        start('bridge')
        deadline = time.monotonic() + 15
        while True:
            if processes['bridge'].poll() is not None:
                raise RuntimeError('Bridge failed; see .logs/bridge.log')
            try:
                with socket.create_connection(('127.0.0.1', 10000), timeout=0.2):
                    break
            except OSError:
                if time.monotonic() > deadline:
                    raise RuntimeError('Bridge startup timed out')
                time.sleep(0.1)
        for name in commands:
            if name != 'bridge':
                start(name)
        print(f'Running four clients for {args.seconds}s. Logs: {logs}', flush=True)
        deadline = time.monotonic() + args.seconds + 25
        for name, process in processes.items():
            if name != 'bridge':
                code = process.wait(timeout=max(1, deadline - time.monotonic()))
                if code:
                    raise RuntimeError(f'{name} exited with {code}; see .logs/{name}.log')
        languages = [name for name in commands if name != 'bridge']
        for receiver in languages:
            output = (logs / f'{receiver}.log').read_text()
            if f'READY {receiver}' not in output:
                raise RuntimeError(f'{receiver} did not become ready')
            for sender in languages:
                if sender != receiver and f'RECV demo/example/{sender} Hello from ' not in output:
                    raise RuntimeError(f'Missing {sender} -> {receiver}; see .logs/{receiver}.log')
            print(f'PASS {receiver}: received from all three other languages')
        print('PASS: all 12 cross-language routes verified.')
    finally:
        for process in processes.values():
            if process.poll() is None:
                process.terminate()
        for process in processes.values():
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait()
        for handle in handles:
            handle.close()


if __name__ == '__main__':
    try:
        main()
    except (RuntimeError, OSError, subprocess.TimeoutExpired) as error:
        print(f'FAIL: {error}', file=sys.stderr)
        sys.exit(1)
    except KeyboardInterrupt:
        sys.exit(130)

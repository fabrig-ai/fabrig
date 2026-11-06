#!/usr/bin/env python3
"""Start the figures' private deployment; retain startup time, memory and closedown.

PROGRAM BUNDLE OUT [--embedded] [--box]. Send SIGTERM to this driver to stop.
Only the generated deployment documents are read, never its secret files.
"""
import argparse
import json
import os
from pathlib import Path
import select
import signal
import socket
import subprocess
import time


def main():
    os.umask(0o077)
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("program", type=Path)
    parser.add_argument("bundle", type=Path)
    parser.add_argument("out", type=Path)
    parser.add_argument("--embedded", action="store_true")
    parser.add_argument("--box", action="store_true")
    args = parser.parse_args()
    args.out.mkdir(mode=0o700, parents=True, exist_ok=False)
    program, directory = args.program.resolve(), args.out.resolve()
    flags = [word for flag in ("--config", "--data", "--cache") for word in (flag, str(directory))]
    subprocess.run([program, "init", "--box", *flags], check=True,
                   stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    path = directory / "temper.json"
    config = json.loads(path.read_text())
    old = [config["application"]["clients"]["listener"]["socket"],
           config["application"]["machine"]["fake"]["listen"]["socket"],
           config["application"]["demo"]["llm"]["listen"]["socket"],
           config["application"]["webhooks"]["listener"]["socket"]]
    reservations = [socket.socket() for _ in old]
    for listener in reservations:
        listener.bind(("127.0.0.1", 0))
    replacements = dict(zip(old, [f"127.0.0.1:{listener.getsockname()[1]}" for listener in reservations]))

    def replace(value):
        if isinstance(value, str):
            return replacements.get(value, "http://" + replacements[value[7:]]
                                    if value.startswith("http://") and value[7:] in replacements else value)
        if isinstance(value, list):
            return [replace(item) for item in value]
        if isinstance(value, dict):
            return {key: replace(item) for key, item in value.items()}
        return value

    config = replace(config)
    config["application"]["clients"]["bundle"] = None if args.embedded else str(args.bundle.resolve())
    path.write_text(json.dumps(config))
    path = directory / "agent.json"
    path.write_text(json.dumps(replace(json.loads(path.read_text()))))
    for listener in reservations:
        listener.close()
    stopping = False

    def stop(_signum, _frame):
        nonlocal stopping
        stopping = True

    signal.signal(signal.SIGTERM, stop)
    signal.signal(signal.SIGINT, stop)
    facts = {"url": f"http://{config['application']['clients']['listener']['socket']}/",
             "embedded": args.embedded, "mode": "box" if args.box else "serve"}
    (directory / "start.json").write_text(json.dumps(facts, indent=2) + "\n")

    def memory(process):
        for line in Path(f"/proc/{process.pid}/status").read_text().splitlines():
            if line.startswith("VmHWM:"):
                return int(line.split()[1])
        return None

    with (directory / "server.log").open("wb") as log:
        started = time.monotonic()
        command = [program, facts["mode"], *flags]
        process = subprocess.Popen(command, stdin=subprocess.DEVNULL,
                                   stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)
        pending = b""
        try:
            while not stopping and process.poll() is None:
                if not select.select([process.stderr], [], [], 0.1)[0]:
                    continue
                chunk = os.read(process.stderr.fileno(), 65536)
                log.write(chunk)
                log.flush()
                pending += chunk
                if "listening_ms" not in facts and b"listening clients" in pending:
                    facts["listening_ms"] = (time.monotonic() - started) * 1000
                    facts["listening_hwm_kib"] = memory(process)
                    (directory / "start.json").write_text(json.dumps(facts, indent=2) + "\n")
                    print(facts["url"], flush=True)
                pending = pending[-4096:]
            if process.poll() is None:
                facts["final_hwm_kib"] = memory(process)
                process.terminate()
            process.wait(timeout=15)
            facts["exit_status"] = process.returncode
        finally:
            if process.poll() is None:
                process.kill()
                process.wait()
            (directory / "start.json").write_text(json.dumps(facts, indent=2) + "\n")
    if facts["exit_status"] != 0:
        raise SystemExit(facts["exit_status"])


if __name__ == "__main__":
    main()

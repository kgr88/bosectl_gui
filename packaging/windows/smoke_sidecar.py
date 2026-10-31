"""Smoke-test a frozen backend without requiring Bluetooth hardware."""

import json
import subprocess
import sys


PROTOCOL_VERSION = 3


def main():
    if len(sys.argv) != 2:
        raise SystemExit("usage: smoke_sidecar.py <bosectl-backend.exe>")

    backend = os.path.realpath(sys.argv[1])
    if not os.path.isfile(backend):
        raise SystemExit("backend executable not found: %s" % backend)

    requests = (
        {"v": PROTOCOL_VERSION, "id": "hello", "method": "hello", "params": {}},
        {"v": PROTOCOL_VERSION, "id": "stop", "method": "shutdown", "params": {}},
    )
    payload = "".join(json.dumps(item, separators=(",", ":")) + "\n" for item in requests)
    completed = subprocess.run(
        [backend],
        input=payload,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        universal_newlines=True,
        timeout=20,
    )
    if completed.returncode != 0:
        raise SystemExit("frozen backend exited with code %d: %s" % (
            completed.returncode, completed.stderr.strip()
        ))

    try:
        messages = [json.loads(line) for line in completed.stdout.splitlines()]
    except ValueError as exc:
        raise SystemExit("frozen backend emitted invalid JSON: %s" % exc)
    if len(messages) != 2:
        raise SystemExit("frozen backend returned %d messages instead of 2" % len(messages))
    hello, shutdown = messages
    if (hello.get("id") != "hello"
            or hello.get("result", {}).get("protocol_version") != PROTOCOL_VERSION):
        raise SystemExit("frozen backend handshake failed")
    if shutdown != {
        "v": PROTOCOL_VERSION, "id": "stop", "result": {"ok": True}
    }:
        raise SystemExit("frozen backend shutdown failed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

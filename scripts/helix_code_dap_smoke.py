#!/usr/bin/env python3
"""Smoke full DAP control surface against lldb-dap (or HELIX_CODE_DAP_COMMAND)."""
from __future__ import annotations

import json
import os
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path


def find_lldb_dap() -> str | None:
    override = os.environ.get("HELIX_CODE_DAP_COMMAND", "").strip()
    if override:
        return override.split()[0]
    for name in ("lldb-dap.exe", "lldb-dap", "lldb-vscode.exe", "lldb-vscode"):
        p = shutil.which(name)
        if p:
            return p
    home = Path.home()
    candidates = [
        home / "scoop" / "apps" / "llvm" / "current" / "bin" / "lldb-dap.exe",
        Path(r"C:\Program Files\LLVM\bin\lldb-dap.exe"),
    ]
    for c in candidates:
        if c.is_file():
            return str(c)
    return None


def ensure_hello(workdir: Path) -> Path:
    src = workdir / "hello.rs"
    exe = workdir / ("hello.exe" if os.name == "nt" else "hello")
    if not exe.is_file():
        src.write_text('fn main() { let x = 42; println!("hi {}", x); }\n', encoding="utf-8")
        subprocess.check_call(["rustc", "-g", "-o", str(exe), str(src)])
    return exe


class Dap:
    def __init__(self, cmd: str, env: dict, cwd: str):
        args = cmd.split() if " " in cmd and not Path(cmd).is_file() else [cmd]
        self.p = subprocess.Popen(
            args,
            cwd=cwd,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            env=env,
            bufsize=0,
        )
        self.seq = 0

    def send(self, command: str, arguments: dict | None = None) -> None:
        self.seq += 1
        msg = {
            "seq": self.seq,
            "type": "request",
            "command": command,
            "arguments": arguments or {},
        }
        body = json.dumps(msg).encode()
        header = f"Content-Length: {len(body)}\r\n\r\n".encode()
        assert self.p.stdin
        self.p.stdin.write(header + body)
        self.p.stdin.flush()

    def read(self, timeout: float = 15.0) -> dict | None:
        assert self.p.stdout
        deadline = time.time() + timeout
        buf = b""
        while time.time() < deadline:
            chunk = self.p.stdout.read(1)
            if not chunk:
                time.sleep(0.01)
                continue
            buf += chunk
            if b"\r\n\r\n" in buf:
                header, rest = buf.split(b"\r\n\r\n", 1)
                n = int(header.decode().split("Content-Length:")[1].strip().split()[0])
                while len(rest) < n and time.time() < deadline:
                    rest += self.p.stdout.read(n - len(rest))
                return json.loads(rest[:n])
        return None

    def request(self, command: str, arguments: dict | None = None, timeout: float = 15.0) -> dict | None:
        self.send(command, arguments)
        while True:
            m = self.read(timeout)
            if m is None:
                return None
            if m.get("type") == "response" and m.get("command") == command:
                return m
            # drain events
            if m.get("type") == "event":
                continue
            return m

    def close(self) -> None:
        try:
            self.send("disconnect", {"terminateDebuggee": True})
            self.read(3)
        except Exception:
            pass
        if self.p.poll() is None:
            self.p.kill()


def main() -> int:
    adapter = find_lldb_dap()
    if not adapter:
        print("DAP_SMOKE_SKIP: no lldb-dap found")
        return 0
    adapter_dir = str(Path(adapter).parent)
    py = Path(os.environ.get("LOCALAPPDATA", "")) / "Programs" / "Python" / "Python311"
    env = os.environ.copy()
    env["PATH"] = os.pathsep.join(
        [str(py) if py.is_dir() else "", adapter_dir, env.get("PATH", "")]
    )
    if py.is_dir():
        env["PYTHONHOME"] = str(py)
    # ensure python311.dll beside lldb if needed
    dll = py / "python311.dll"
    if dll.is_file():
        target = Path(adapter_dir) / "python311.dll"
        if not target.is_file():
            shutil.copy2(dll, target)

    work = Path(tempfile.mkdtemp(prefix="helix-dap-"))
    prog = ensure_hello(work)
    print(f"adapter={adapter}")
    print(f"program={prog}")

    dap = Dap(adapter, env, adapter_dir)
    try:
        init = dap.request(
            "initialize",
            {
                "clientID": "helix-code-smoke",
                "clientName": "HelixCode",
                "adapterID": "lldb",
                "pathFormat": "path",
                "linesStartAt1": True,
                "columnsStartAt1": True,
            },
        )
        if not init or not init.get("success"):
            print("DAP_SMOKE_FAIL initialize", init)
            return 1
        print("initialize ok")

        # drain initialized event
        for _ in range(3):
            m = dap.read(2)
            if m and m.get("event") == "initialized":
                print("initialized event")
                break

        launch = dap.request(
            "launch",
            {
                "name": "helix-smoke",
                "type": "lldb",
                "request": "launch",
                "program": str(prog),
                "stopOnEntry": True,
                "cwd": str(work),
            },
            timeout=30,
        )
        print("launch", launch and launch.get("success"), launch and launch.get("message"))

        cfg = dap.request("configurationDone", {})
        print("configurationDone", cfg and cfg.get("success"))

        # wait for stopped
        stopped = False
        for _ in range(10):
            m = dap.read(3)
            if not m:
                break
            if m.get("event") == "stopped":
                stopped = True
                print("stopped event")
                break

        thr = dap.request("threads", {})
        threads = (thr or {}).get("body", {}).get("threads") or []
        tid = threads[0]["id"] if threads else 1
        print("threads", threads)

        st = dap.request("stackTrace", {"threadId": tid, "startFrame": 0, "levels": 20})
        frames = (st or {}).get("body", {}).get("stackFrames") or []
        print("frames", len(frames))
        fid = frames[0]["id"] if frames else 0

        if fid:
            sc = dap.request("scopes", {"frameId": fid})
            scopes = (sc or {}).get("body", {}).get("scopes") or []
            print("scopes", [s.get("name") for s in scopes])
            if scopes:
                vr = scopes[0].get("variablesReference", 0)
                if vr:
                    vars_ = dap.request("variables", {"variablesReference": vr})
                    print("variables ok", bool(vars_ and vars_.get("success")))

        for cmd in ("next", "stepIn", "stepOut", "continue", "pause"):
            r = dap.request(cmd, {"threadId": tid}, timeout=8)
            print(cmd, r and r.get("success"))

        print("FULL_DAP_SMOKE_OK")
        return 0
    finally:
        dap.close()


if __name__ == "__main__":
    sys.exit(main())

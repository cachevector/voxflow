#!/usr/bin/env python3
"""Render the three VoxFlow feature graphics with Chrome."""

from __future__ import annotations

import base64
import http.server
import json
import os
import socketserver
import subprocess
import threading
import time
import urllib.request
from pathlib import Path

import websocket

ROOT = Path(__file__).resolve().parent
CHROME = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
PORT = 8765
CDP = 9335

SHOTS = [
    ("render/hero.html", "feature-hero.png", 1920, 1080, 2),
    ("render/ondevice.html", "feature-ondevice.png", 1920, 1080, 2),
    ("render/square.html", "feature-square.png", 1080, 1080, 2),
]


def serve() -> socketserver.TCPServer:
    os.chdir(ROOT)
    handler = http.server.SimpleHTTPRequestHandler
    socketserver.TCPServer.allow_reuse_address = True
    httpd = socketserver.TCPServer(("127.0.0.1", PORT), handler)
    threading.Thread(target=httpd.serve_forever, daemon=True).start()
    return httpd


def wait_port(port: int) -> None:
    for _ in range(50):
        try:
            urllib.request.urlopen(f"http://127.0.0.1:{port}/json/version", timeout=0.2)
            return
        except Exception:
            time.sleep(0.15)
    raise SystemExit(f"Chrome CDP did not come up on {port}")


def cdp(ws, mid: list[int], method: str, params: dict | None = None) -> dict:
    mid[0] += 1
    i = mid[0]
    ws.send(json.dumps({"id": i, "method": method, "params": params or {}}))
    while True:
        data = json.loads(ws.recv())
        if data.get("id") == i:
            if "error" in data:
                raise RuntimeError(data["error"])
            return data


def main() -> None:
    httpd = serve()
    subprocess.run(
        ["pkill", "-f", f"remote-debugging-port={CDP}"],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    time.sleep(0.2)
    proc = subprocess.Popen(
        [
            CHROME,
            "--headless=new",
            "--disable-gpu",
            f"--remote-debugging-port={CDP}",
            "--remote-allow-origins=*",
            f"--user-data-dir=/tmp/voxflow-marketing-chrome-{CDP}",
            "--no-first-run",
            "--no-default-browser-check",
            "about:blank",
        ],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    try:
        wait_port(CDP)
        tabs = json.loads(urllib.request.urlopen(f"http://127.0.0.1:{CDP}/json/list").read())
        page = next(t for t in tabs if t.get("type") == "page")
        ws = websocket.create_connection(page["webSocketDebuggerUrl"], timeout=15)
        mid = [0]
        cdp(ws, mid, "Page.enable")
        cdp(ws, mid, "Runtime.enable")

        for src, dest, w, h, dpr in SHOTS:
            cdp(
                ws,
                mid,
                "Emulation.setDeviceMetricsOverride",
                {
                    "width": w,
                    "height": h,
                    "deviceScaleFactor": dpr,
                    "mobile": False,
                },
            )
            cdp(ws, mid, "Page.navigate", {"url": f"http://127.0.0.1:{PORT}/{src}"})
            time.sleep(0.9)
            # Wait for fonts
            cdp(
                ws,
                mid,
                "Runtime.evaluate",
                {"expression": "document.fonts.ready.then(() => true)", "awaitPromise": True},
            )
            time.sleep(0.2)
            shot = cdp(ws, mid, "Page.captureScreenshot", {"format": "png", "fromSurface": True})
            out = ROOT / dest
            out.write_bytes(base64.b64decode(shot["result"]["data"]))
            print(f"wrote {out} ({out.stat().st_size // 1024} KB)")

        ws.close()
    finally:
        proc.terminate()
        httpd.shutdown()


if __name__ == "__main__":
    main()

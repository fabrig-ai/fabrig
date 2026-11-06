#!/usr/bin/env python3
"""Render the request beside a release page under its unchanged policy.

URL OUT. The TCP/HTTP proxy forwards the policy and compressed bytes, with
explicit fault fixtures for stalled/dropped/silent answers and faulty glue.
Each JSON retains every element's computed style and box, including the SVG.
"""
import argparse
import base64
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import runpy
import socket
import subprocess
import tempfile
import threading
import time
import urllib.error
import urllib.request

Pipe = runpy.run_path(str(Path(__file__).with_name("load.py")))["Pipe"]

SNAPSHOT = """JSON.stringify((() => {
 const b=document.getElementById('boot');
 const elements=[document.documentElement,document.body,...(b?[b,...b.querySelectorAll('*')]:[])];
 return {time:performance.now(),title:document.title,boot:!!b,
  state:b?.dataset.state||'',out:!!b?.classList.contains('out'),inert:!!b?.inert,
  line:document.getElementById('boot-line')?.textContent??null,
  note:document.getElementById('boot-note')?.textContent??null,
  value:b?.querySelector('progress')?.value??null,
  app:document.getElementById('app')?.firstChild?.nodeName??null,
  elements:elements.filter(Boolean).map(e=>{const s=getComputedStyle(e),r=e.getBoundingClientRect();
   return {tag:e.tagName,id:e.id,text:e.tagName==='svg'?'':e.textContent,
    box:{x:r.x,y:r.y,width:r.width,height:r.height},
    style:Object.fromEntries([...s].map(k=>[k,s.getPropertyValue(k)]))};})};
})())"""


class Proxy(BaseHTTPRequestHandler):
    def log_message(self, *_args):
        pass

    def do_GET(self):
        mode = self.server.mode
        module = self.path.endswith("_bg.wasm")
        glue = self.path.endswith(".js") and not self.path.endswith("/boot.js")
        if mode == "silent" and (module or glue or
                                 self.path.endswith(".css") and not self.path.endswith("/loading.css")):
            time.sleep(0.5)
            self.connection.close()
            return
        try:
            request = urllib.request.Request(self.server.upstream.rstrip("/") + self.path,
                                             headers={"Accept-Encoding": self.headers.get("Accept-Encoding", "identity")})
            try:
                response = urllib.request.urlopen(request, timeout=10)
            except urllib.error.HTTPError as error:
                response = error
            replacement = None
            if glue and mode in ("starting", "init-threw"):
                replacement = (b"export default async function({module_or_path}){"
                               b"await module_or_path.arrayBuffer();"
                               + (b"await new Promise(()=>{});" if mode == "starting" else
                                  b"await new Promise(r=>setTimeout(r,100));throw Error('fixture init');") + b"}")
            self.send_response(response.status)
            for name, value in response.headers.items():
                if name.lower() in ("connection", "transfer-encoding"):
                    continue
                if replacement is not None and name.lower() in ("content-length", "content-encoding"):
                    continue
                self.send_header(name, value)
            if replacement is not None:
                self.send_header("Content-Length", str(len(replacement)))
            self.end_headers()
            if replacement is not None:
                self.wfile.write(replacement)
                return
            sent = 0
            while data := response.read(16384):
                self.wfile.write(data)
                self.wfile.flush()
                sent += len(data)
                if module and mode in ("stall", "dropped") and sent >= 131072:
                    if mode == "stall":
                        self.server.release.wait(35)
                    else:
                        time.sleep(1)
                        self.connection.shutdown(socket.SHUT_RDWR)
                        return
            response.close()
        except (BrokenPipeError, ConnectionError, TimeoutError, OSError):
            self.close_connection = True


def evaluate(pipe, session, expression):
    result = pipe.call("Runtime.evaluate", {"expression": expression, "returnByValue": True}, session)
    if "exceptionDetails" in result:
        raise RuntimeError(result["exceptionDetails"])
    return result.get("result", {}).get("value")


def main():
    os.umask(0o077)
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("url")
    parser.add_argument("out", type=Path)
    parser.add_argument("--request", default="0c1b78f3e1")
    parser.add_argument("--not-found-url", required=True)
    args = parser.parse_args()
    args.out.mkdir(mode=0o700, parents=True, exist_ok=False)
    reference = args.out / "request"
    reference.mkdir()
    for name in ("01.html", "02.html", "03.html", "04.html", "loading.css"):
        (reference / name).write_bytes(subprocess.check_output([
            "git", "show", f"{args.request}:temper/docs/design/client/journeys/bundle-loading/{name}"]))
    # A separate static server gives the untouched request its own stylesheet.
    static = subprocess.Popen(["python3", "-I", "-B", "-u", "-m", "http.server", "0", "--bind", "127.0.0.1",
                               "--directory", str(reference)], stdout=subprocess.PIPE, stderr=subprocess.DEVNULL,
                              env={**os.environ, "PYTHONUNBUFFERED": "1"}, stdin=subprocess.DEVNULL)
    banner = static.stdout.readline().decode()
    static_port = banner.split("port ")[1].split()[0]
    proxy = ThreadingHTTPServer(("127.0.0.1", 0), Proxy)
    proxy.upstream = args.url
    proxy.mode = "normal"
    proxy.release = threading.Event()
    threading.Thread(target=proxy.serve_forever, daemon=True).start()
    proxy_url = f"http://127.0.0.1:{proxy.server_port}/"
    results = []
    with tempfile.TemporaryDirectory(prefix="bundle-render-") as profile, (args.out / "chromium.log").open("wb") as stderr:
        pipe = Pipe(profile, stderr)
        try:
            for width in (1440, 1000, 420):
                target = pipe.call("Target.createTarget", {"url": "about:blank"})["targetId"]
                session = pipe.call("Target.attachToTarget", {"targetId": target, "flatten": True})["sessionId"]
                for domain in ("Page", "Runtime", "Network", "Log"):
                    pipe.call(f"{domain}.enable", session=session)
                pipe.call("Emulation.setDeviceMetricsOverride", {"width": width, "height": 900,
                          "deviceScaleFactor": 1, "mobile": False}, session)
                pipe.call("Network.setCacheDisabled", {"cacheDisabled": True}, session)
                errors = []

                def event(message):
                    if message.get("sessionId") != session:
                        return
                    method, params = message["method"], message.get("params", {})
                    if method == "Page.javascriptDialogOpening":
                        pipe.send("Page.handleJavaScriptDialog", {"accept": True}, session)
                    elif method == "Runtime.exceptionThrown" or (method == "Log.entryAdded" and params["entry"]["source"] == "security"):
                        errors.append({"method": method, "params": params})

                pipe.on_event = event

                def navigate(mode="normal", rate=250000, url=None):
                    proxy.release.set()
                    pipe.call("Page.navigate", {"url": "about:blank"}, session)
                    proxy.release = threading.Event()
                    proxy.mode = mode
                    pipe.call("Network.emulateNetworkConditions", {"offline": False, "latency": 0,
                              "downloadThroughput": rate, "uploadThroughput": rate}, session)
                    errors.clear()
                    pipe.call("Page.navigate", {"url": url or proxy_url}, session)

                def wait(predicate, limit=35):
                    deadline = time.monotonic() + limit
                    while time.monotonic() < deadline:
                        pipe.pump(0.02)
                        snap = json.loads(evaluate(pipe, session, SNAPSHOT))
                        if predicate(snap):
                            return snap
                    raise TimeoutError(f"state at {width}: {snap}")

                def capture(name, snap):
                    if not name.startswith("request-") and snap["state"] == "failed":
                        # Unhiding Try again starts its own 250 ms CSS delay.
                        failed_at = snap["time"]
                        snap = wait(lambda s: s["time"] > failed_at + 500)
                    shot = pipe.call("Page.captureScreenshot", {"format": "png"}, session)
                    filename = f"{width}-{name}.png"
                    (args.out / filename).write_bytes(base64.b64decode(shot["data"]))
                    facts = {"width": width, "name": name, "screenshot": filename,
                             "snapshot": snap, "errors": list(errors)}
                    (args.out / f"{width}-{name}.json").write_text(json.dumps(facts, indent=2, ensure_ascii=False) + "\n")
                    results.append(facts)
                    print(f"{width} {name}: {snap.get('line')} / {snap.get('note')}", flush=True)

                def button_states(prefix):
                    point = evaluate(pipe, session, "(() => {const r=document.querySelector('#boot button').getBoundingClientRect();return {x:r.x+r.width/2,y:r.y+r.height/2};})()")
                    pipe.call("Input.dispatchMouseEvent", {"type": "mouseMoved", **point}, session)
                    capture(prefix + "-hover", json.loads(evaluate(pipe, session, SNAPSHOT)))
                    pipe.call("Input.dispatchMouseEvent", {"type": "mouseMoved", "x": 0, "y": 0}, session)
                    pipe.call("Input.dispatchKeyEvent", {"type": "keyDown", "key": "Tab", "code": "Tab"}, session)
                    pipe.call("Input.dispatchKeyEvent", {"type": "keyUp", "key": "Tab", "code": "Tab"}, session)
                    capture(prefix + "-focus", json.loads(evaluate(pipe, session, SNAPSHOT)))

                for number in range(1, 5):
                    navigate(url=f"http://127.0.0.1:{static_port}/{number:02}.html")
                    capture(f"request-{number:02}", wait(lambda s: s["boot"] and s["time"] > 600))
                button_states("request-04")

                navigate(rate=62500)
                capture("paper", wait(lambda s: s["boot"] and 100 < s["time"] < 240))
                capture("downloading", wait(lambda s: s["boot"] and "about" in (s["line"] or "")))
                capture("eight-seconds", wait(lambda s: s["boot"] and "keeps" in (s["note"] or "")))
                navigate("stall")
                capture("waiting", wait(lambda s: s["line"] == "Waiting for the server…"))
                navigate("starting")
                capture("starting", wait(lambda s: s["state"] == "start"))
                for mode in ("dropped", "silent", "init-threw"):
                    navigate(mode)
                    capture(mode, wait(lambda s: s["state"] == "failed" and s["time"] > 650))
                    if mode == "dropped":
                        button_states("dropped")
                navigate(url=args.not_found_url)
                capture("not-found", wait(lambda s: s["state"] == "failed" and s["time"] > 650))
                script = pipe.call("Page.addScriptToEvaluateOnNewDocument", {"source": "delete window.WebAssembly;"}, session)["identifier"]
                navigate()
                capture("no-webassembly", wait(lambda s: s["state"] == "failed" and s["time"] > 600))
                pipe.call("Page.removeScriptToEvaluateOnNewDocument", {"identifier": script}, session)
                evaluate(pipe, session, "document.querySelector('#boot button').click()")
                capture("retry", wait(lambda s: s["boot"] and s["state"] != "failed" and s["time"] > 650))
                pipe.call("Emulation.setScriptExecutionDisabled", {"value": True}, session)
                navigate()
                capture("scripts-off", wait(lambda s: s["boot"] and s["time"] > 600))
                pipe.call("Emulation.setScriptExecutionDisabled", {"value": False}, session)
                pipe.call("Emulation.setEmulatedMedia", {"features": [{"name": "prefers-reduced-motion", "value": "reduce"}]}, session)
                navigate(rate=62500)
                capture("reduced-motion", wait(lambda s: s["boot"] and s["time"] > 1000))
                pipe.call("Emulation.setEmulatedMedia", {"features": []}, session)
                navigate()
                capture("handing-over", wait(lambda s: s["out"]))
                capture("client", wait(lambda s: not s["boot"] and bool(s["app"])))
                pipe.call("Target.closeTarget", {"targetId": target})
        finally:
            pipe.close()
            proxy.release.set()
            proxy.shutdown()
            proxy.server_close()
            static.terminate()
            static.wait(timeout=5)
    (args.out / "render.json").write_text(json.dumps(results, indent=2, ensure_ascii=False) + "\n")


if __name__ == "__main__":
    main()

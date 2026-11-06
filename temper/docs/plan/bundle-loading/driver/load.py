#!/usr/bin/env python3
"""Measure a bundle through CDP's pipe, without changing the page's policy.

Run with python3 -I -B load.py URL WIDTH OUT --mark first-paint --mark first-text.
--cached adds a second navigation in the same browser/profile after the fresh load.
OUT must be new; JSON, Chromium stderr and requested PNGs are retained there.
"""

import argparse
import base64
import json
import os
from pathlib import Path
import select
import subprocess
import tempfile
import time


MARKS = ("first-paint", "first-contentful-paint", "first-text", "first-node", "loaded")


class Pipe:
    """Null-delimited CDP JSON on the browser's descriptors 3 and 4."""

    def __init__(self, profile, stderr):
        command_read, self.command_write = os.pipe()
        self.result_read, result_write = os.pipe()

        def child_descriptors():
            os.dup2(command_read, 3)
            os.dup2(result_write, 4)

        self.process = subprocess.Popen(
            [os.environ["SKEIN_TEST_CHROMIUM"], "--headless=new",
             "--remote-debugging-pipe", f"--user-data-dir={profile}",
             "--disable-background-networking", "--disable-component-update",
             "--disable-sync", "--disable-default-apps", "--metrics-recording-only",
             "--safebrowsing-disable-auto-update",
             "--no-first-run", "--no-default-browser-check", "about:blank"],
            stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=stderr,
            pass_fds=tuple(sorted({3, 4, command_read, result_write})),
            preexec_fn=child_descriptors,
        )
        os.close(command_read)
        os.close(result_write)
        self.sequence = 0
        self.buffer = b""
        self.answers = {}
        self.on_event = lambda message: None

    def send(self, method, params=None, session=None):
        self.sequence += 1
        message = {"id": self.sequence, "method": method, "params": params or {}}
        if session:
            message["sessionId"] = session
        data = json.dumps(message).encode() + b"\0"
        while data:
            data = data[os.write(self.command_write, data):]
        return self.sequence

    def pump(self, timeout):
        if b"\0" not in self.buffer:
            if not select.select([self.result_read], [], [], max(0, timeout))[0]:
                return
            data = os.read(self.result_read, 1024 * 1024)
            if not data:
                raise RuntimeError("Chromium closed its CDP pipe")
            self.buffer += data
        while b"\0" in self.buffer:
            raw, self.buffer = self.buffer.split(b"\0", 1)
            message = json.loads(raw)
            if "id" in message:
                self.answers[message["id"]] = message
            else:
                self.on_event(message)

    def call(self, method, params=None, session=None):
        sequence = self.send(method, params, session)
        deadline = time.monotonic() + 15
        while sequence not in self.answers:
            if time.monotonic() >= deadline:
                raise TimeoutError(method)
            self.pump(min(0.05, deadline - time.monotonic()))
        answer = self.answers.pop(sequence)
        if "error" in answer:
            raise RuntimeError(f"{method}: {answer['error']}")
        return answer.get("result", {})

    def close(self):
        if self.process.poll() is None:
            self.send("Browser.close")
            try:
                self.process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                self.process.terminate()
                self.process.wait(timeout=10)
        os.close(self.command_write)
        os.close(self.result_read)


OBSERVE = """(() => {
  const facts = window.__bundleMeasurement = {node: null};
  const observer = new MutationObserver(() => {
    const app = document.getElementById('app');
    if (app && app.firstChild && !facts.node) {
      facts.node = {time: performance.now(), html:
        app.firstChild.outerHTML || app.firstChild.textContent};
      observer.disconnect();
    }
  });
  observer.observe(document, {childList: true, subtree: true});
})();"""

POLL = """JSON.stringify({time: performance.now(),
  paints: performance.getEntriesByType('paint').map(p => ({name:p.name,time:p.startTime})),
  text: document.body ? document.body.innerText.trim() : '',
  node: window.__bundleMeasurement ? window.__bundleMeasurement.node : null,
  title: document.title,
  boot: (() => { const b = document.getElementById('boot');
    return b ? {state: b.dataset.state || '', out: b.classList.contains('out'), inert: b.inert,
      opacity: getComputedStyle(b).opacity,
      children: [...b.children].map(e => ({tag:e.tagName, text:e.textContent,
        display:getComputedStyle(e).display, opacity:getComputedStyle(e).opacity}))} : null; })(),
  background: document.body ? getComputedStyle(document.body).backgroundColor : null,
  ready: document.readyState})"""


def measure(pipe, session, args, label):
    events, transfers, requests, milestones, samples = [], [], {}, {}, []
    origin = None
    started = time.monotonic()
    loaded = False

    def event(message):
        nonlocal origin, loaded
        if message.get("sessionId") != session:
            return
        method, params = message["method"], message.get("params", {})
        stamp = params.get("timestamp")
        if stamp is None:
            stamp = (origin or started) + time.monotonic() - started
        if method == "Page.javascriptDialogOpening":
            pipe.send("Page.handleJavaScriptDialog", {"accept": True}, session)
        elif method == "Network.requestWillBeSent":
            if params.get("type") == "Document" and origin is None:
                origin = params["timestamp"]
            requests[params["requestId"]] = {"url": params["request"]["url"], "counted": 0}
        elif method == "Network.responseReceived":
            response = params["response"]
            request = requests.setdefault(params["requestId"], {"counted": 0})
            request["url"] = response["url"]
            request["cached"] = response.get("fromDiskCache", False)
            size = response.get("encodedDataLength", 0)
            request["counted"] += size
            transfers.append((stamp, size))
            headers = {key.lower(): value for key, value in response["headers"].items()}
            events.append({"event": "response", "timestamp": stamp,
                           "request_id": params["requestId"], "url": response["url"],
                           "status": response["status"], "content_encoding": headers.get("content-encoding"),
                           "from_disk_cache": request["cached"], "headers": response["headers"]})
        elif method == "Network.dataReceived":
            size = params["encodedDataLength"]
            requests[params["requestId"]]["counted"] += size
            transfers.append((stamp, size))
        elif method == "Network.loadingFinished":
            request = requests[params["requestId"]]
            size = params["encodedDataLength"]
            transfers.append((stamp, size - request["counted"]))
            request["counted"] = size
            events.append({"event": "finished", "timestamp": stamp,
                           "request_id": params["requestId"], "url": request["url"],
                           "encoded_data_length": size})
        elif method == "Network.loadingFailed":
            events.append({"event": "failed", "timestamp": stamp, "details": params})
        elif method == "Runtime.exceptionThrown":
            # exceptionDetails.timestamp is epoch time, unlike Network timestamps.
            events.append({"event": "exception", "timestamp": (origin or started) + time.monotonic() - started,
                           "details": params})
        elif method == "Log.entryAdded" and params["entry"]["source"] == "security":
            events.append({"event": "security", "timestamp": stamp, "details": params["entry"]})
        elif method == "Page.loadEventFired":
            loaded = True

    pipe.on_event = event
    navigation = pipe.call("Page.navigate", {"url": args.url}, session)
    if navigation.get("errorText"):
        raise RuntimeError(navigation["errorText"])
    metrics = pipe.call("Performance.getMetrics", session=session)["metrics"]
    origin = next(metric["value"] for metric in metrics if metric["name"] == "NavigationStart")
    next_poll = time.monotonic()

    def mark(name, milliseconds, **details):
        if name in milestones:
            return
        milestones[name] = {"time_ms": milliseconds, **details}
        if name in args.mark:
            shot = pipe.call("Page.captureScreenshot", {"format": "png"}, session)
            path = args.out / f"{label}-{name}.png"
            path.write_bytes(base64.b64decode(shot["data"]))
            milestones[name]["screenshot"] = path.name
            milestones[name]["screenshot_observed_ms"] = (time.monotonic() - started) * 1000

    while time.monotonic() - started < args.timeout:
        pipe.pump(max(0, min(0.05, next_poll - time.monotonic())))
        if time.monotonic() < next_poll:
            continue
        value = pipe.call("Runtime.evaluate", {"expression": POLL, "returnByValue": True}, session)
        if "exceptionDetails" not in value and "value" in value.get("result", {}):
            snapshot = json.loads(value["result"]["value"])
            samples.append({key: snapshot[key] for key in ("time", "title", "boot")})
            for paint in snapshot["paints"]:
                mark(paint["name"], paint["time"], background=snapshot["background"])
            if snapshot["text"]:
                mark("first-text", snapshot["time"], text=snapshot["text"])
            if snapshot["node"]:
                mark("first-node", snapshot["node"]["time"], html=snapshot["node"]["html"])
            if loaded:
                mark("loaded", snapshot["time"])
            if all(name in milestones for name in MARKS) and time.monotonic() - started > milestones["first-text"]["time_ms"] / 1000 + 1:
                break
        next_poll += 0.05
        # A busy renderer or screenshot may have taken more than one polling interval.
        next_poll = max(next_poll, time.monotonic())

    for fact in milestones.values():
        fact["bytes_received"] = sum(size for stamp, size in transfers if stamp <= origin + fact["time_ms"] / 1000)
    for item in events:
        item["time_ms"] = (item.pop("timestamp") - origin) * 1000
        item["bytes_received"] = sum(size for stamp, size in transfers if stamp <= origin + item["time_ms"] / 1000)
    return {"label": label, "milestones": milestones, "events": events,
            "missing": [name for name in MARKS if name not in milestones],
            "bytes_received": sum(size for _, size in transfers),
            "transfers": [{"time_ms": (stamp - origin) * 1000, "bytes": size} for stamp, size in transfers],
            "samples": samples}


def main():
    os.umask(0o077)
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("url")
    parser.add_argument("width", type=int)
    parser.add_argument("out", type=Path)
    parser.add_argument("--mark", action="append", choices=MARKS, default=[])
    parser.add_argument("--cached", action="store_true")
    parser.add_argument("--timeout", type=float, default=120)
    args = parser.parse_args()
    if args.width <= 0 or args.timeout <= 0:
        parser.error("width and timeout must be positive")
    args.out.mkdir(mode=0o700, parents=True, exist_ok=False)
    result = {"url": args.url, "width": args.width, "height": 900,
              "network": {"offline": False, "latency": 0,
                          "downloadThroughput": 250000, "uploadThroughput": 250000},
              "poll_interval_ms": 50, "cache_disabled": False, "runs": []}
    with tempfile.TemporaryDirectory(prefix="bundle-load-") as profile, (args.out / "chromium.log").open("wb") as stderr:
        pipe = Pipe(profile, stderr)
        try:
            result["browser"] = pipe.call("Browser.getVersion")
            target = pipe.call("Target.createTarget", {"url": "about:blank"})["targetId"]
            session = pipe.call("Target.attachToTarget", {"targetId": target, "flatten": True})["sessionId"]
            for domain in ("Page", "Runtime", "Network", "Log", "Performance"):
                pipe.call(f"{domain}.enable", session=session)
            pipe.call("Emulation.setDeviceMetricsOverride", {"width": args.width, "height": 900,
                      "deviceScaleFactor": 1, "mobile": False}, session)
            pipe.call("Network.emulateNetworkConditions", result["network"], session)
            pipe.call("Network.setCacheDisabled", {"cacheDisabled": False}, session)
            pipe.call("Page.addScriptToEvaluateOnNewDocument", {"source": OBSERVE}, session)
            result["runs"].append(measure(pipe, session, args, "fresh"))
            if args.cached:
                # Keep Chromium and its HTTP cache; leave the document before navigating again.
                pipe.call("Page.navigate", {"url": "about:blank"}, session)
                pipe.on_event = lambda message: None
                pipe.call("Runtime.evaluate", {"expression": "document.readyState"}, session)
                result["runs"].append(measure(pipe, session, args, "cached"))
        finally:
            pipe.close()
    document = json.dumps(result, indent=2, ensure_ascii=False) + "\n"
    (args.out / "load.json").write_text(document)
    print(document, end="")


if __name__ == "__main__":
    main()

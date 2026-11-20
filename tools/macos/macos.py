#!/usr/bin/env python3
"""Dispatch and report macOS parts (temper/docs/design/testing/macos.md, 1–8)."""

import argparse
import copy
import json
import os
from pathlib import Path
import re
import shutil
import signal
import subprocess
import sys
import time
import tomllib
import uuid
import xml.etree.ElementTree as ET


SETS = {
    "e2e": ("process", "browser", "browser-temper"),
    "suites": ("skein", "smith", "jig", "temper"),
    "live": ("live",),
}
SETS["all"] = SETS["e2e"] + SETS["suites"]
PAIRS = {
    "process": ("process-temper",),
    "browser": ("browser-jig", "browser-skein"),
    "browser-temper": ("browser-temper",),
    "live": ("live-temper", "live-skein"),
}
FAULT = 2
BOUNDS = {"process": 150, "browser": 150, "browser-temper": 150,
          "skein": 300, "smith": 300, "jig": 300, "temper": 300, "live": 60}
RESERVE = 15 * 60
STOP_GRACE = 60


def ids(text):
    return list(dict.fromkeys(line.strip() for line in text.splitlines() if line.strip()))


def identity(identifier):
    binary, separator, test = identifier.partition(".")
    if not separator or not binary or not test:
        raise ValueError(f"invalid test id: {identifier}")
    return binary, test


def tree_holds(tree, package):
    if tree == "skein":
        return package.startswith("skein-") or package in ("forgejo-conformance", "github-conformance")
    return package == tree or package.startswith(tree + "-")


def holds(part, identifier, packages):
    binary, test = identity(identifier)
    if binary == "build":
        return test == part
    package = binary.split("::", 1)[0]
    if part in PAIRS:
        return any(package in packages[pair] for pair in PAIRS[part])
    return tree_holds(part, package)


def pair_ids(pair, selected, packages):
    return [item for item in selected if identity(item)[0].split("::", 1)[0] in packages[pair]]


def place(test_set, text, packages):
    """Place by package, without a build or a listing (4.1–4.2)."""
    selected = ids(text)
    parts = {}
    for part in SETS[test_set]:
        owned = [item for item in selected if holds(part, item, packages)]
        parts[part] = dict(run=not selected or bool(owned), ids=owned)
        if part in PAIRS:
            parts[part]["pairs"] = [pair for pair in PAIRS[part]
                                    if not selected or pair_ids(pair, owned, packages)]
    unheld = [item for item in selected if not any(item in value["ids"] for value in parts.values())]
    return parts, unheld


def filterset(selected):
    return " | ".join(f"binary_id(={binary}) & test(={test})"
                      for binary, test in map(identity, selected) if binary != "build")


def annotation(kind, title, text):
    def escape(value):
        return str(value).replace("%", "%25").replace("\r", "%0D").replace("\n", "%0A")
    title = escape(title).replace(":", "%3A").replace(",", "%2C")
    print(f"::{kind} title={title}::{escape(text)}", flush=True)


def output(name, value):
    # A fresh delimiter also handles input text containing delimiter-like lines.
    delimiter = "macos_" + uuid.uuid4().hex
    with Path(os.environ["GITHUB_OUTPUT"]).open("a", encoding="utf-8") as stream:
        stream.write(f"{name}<<{delimiter}\n{value}\n{delimiter}\n")


def write_xml(path, root):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    ET.ElementTree(root).write(path, encoding="utf-8", xml_declaration=True)


def suites(path):
    root = ET.parse(path).getroot()
    if root.tag == "testsuite":
        return [root]
    if root.tag != "testsuites":
        raise ValueError(f"not JUnit: {path}")
    return list(root.iter("testsuite"))


def merge(paths, destination):
    """Keep suites, cases and times unchanged (5.1)."""
    root = ET.Element("testsuites")
    for path in paths:
        root.extend(copy.deepcopy(suite) for suite in suites(path))
    write_xml(destination, root)


def build_case(part, log, destination):
    """Only cargo's compiler refusal is a red (5.2)."""
    text = Path(log).read_text(encoding="utf-8", errors="replace")
    if not re.search(r"^error(?:\[[^\]]+\])?: could not compile\b", text, re.M):
        raise ValueError(f"{part}: build fault, no cargo 'could not compile' line in {log}")
    blocks = []
    size = 0
    for match in re.finditer(r"^error(?:\[[^\]]+\])?:[^\n]*(?:\n(?![ \t]*\n)[^\n]*)*(?:\n[ \t]*\n|\Z)", text, re.M):
        block = match.group(0)
        if size + len(block.encode("utf-8")) > 16 * 1024:
            break
        blocks.append(block)
        size += len(block.encode("utf-8"))
    root = ET.Element("testsuites")
    suite = ET.SubElement(root, "testsuite", name=part, tests="1", failures="1", errors="0")
    case = ET.SubElement(suite, "testcase", classname="build", name=part)
    ET.SubElement(case, "failure", type="build failure").text = "".join(blocks)
    write_xml(destination, root)


def check_status(status):
    if status not in (0, 100):
        raise ValueError(f"run fault: exit status {status} (expected 0 or 100)")


def check_one_process(directory, status):
    """A run cut short is a fault, not a partial report (5.4)."""
    check_status(status)
    directory = Path(directory)
    run = json.loads((directory / "run.json").read_text())
    result = json.loads((directory / "results.json").read_text())
    if run["starved"] or not run["finished"] or result["unobserved"]:
        raise ValueError(f"{directory}: cut short: starved={run['starved']}, unobserved={result['unobserved']}")
    if any(test["outcome"] in ("starved", "cancelled") for test in result["tests"]):
        raise ValueError(f"{directory}: cut short: starved or cancelled test")
    suites(directory / "junit.xml")


def check_nextest(listing, junit, status, run_ignored="default"):
    """Compare selection to observation, including ignored-test policy (5.4)."""
    check_status(status)
    selected = set()
    for binary, suite in json_document(listing, "rust-suites")["rust-suites"].items():
        if suite["status"] != "listed":
            continue
        for name, case in suite["testcases"].items():
            ignored = case["ignored"]
            if case["filter-match"]["status"] == "matches" and (
                run_ignored == "all" or ignored == (run_ignored == "only")
            ):
                selected.add((binary, name))
    observed = {(case.get("classname"), case.get("name"))
                for suite in suites(junit) for case in suite.iter("testcase")
                if case.find("skipped") is None}
    missing = selected - observed
    if missing:
        raise ValueError("nextest cut short, unobserved: " + ", ".join(f"{b}.{t}" for b, t in sorted(missing)))


def report(parts, directory, destination):
    """Validate every part before writing the one report (5.3–5.4)."""
    destination = Path(destination)
    destination.unlink(missing_ok=True)
    files = {part: Path(directory) / f"{part}.xml" for part in parts}
    for part, path in files.items():
        if not path.is_file():
            files[part] = Path(directory) / f"part-{part}" / f"{part}.xml"
    missing = [part for part, path in files.items() if not path.is_file()]
    if missing:
        for part in missing:
            annotation("error", "missing part", part)
        raise ValueError("missing part files: " + ", ".join(missing))
    root = ET.Element("testsuites")
    for part in parts:
        suite = ET.SubElement(root, "testsuite", name=part)
        for source in suites(files[part]):
            for case in source.iter("testcase"):
                if case.find("failure") is not None or case.find("error") is not None:
                    suite.append(copy.deepcopy(case))
        suite.set("tests", str(len(suite)))
        suite.set("failures", str(sum(c.find("failure") is not None for c in suite)))
        suite.set("errors", str(sum(c.find("error") is not None for c in suite)))
    destination.parent.mkdir(parents=True, exist_ok=True)
    for part, path in files.items():
        copy_to = destination.parent / f"{part}.xml"
        if path.resolve() != copy_to.resolve():
            shutil.copyfile(path, copy_to)
    write_xml(destination, root)


def annotate(name, path):
    path = Path(path)
    lines = path.read_text(encoding="utf-8", errors="replace").splitlines() if path.exists() else []
    failed = [line.strip() for line in lines
              if re.match(r"\s+(FAIL|SIGSEGV|SIGABRT|SIGKILL|TIMEOUT|LEAK-FAIL)\s+\[", line)]
    headlines = [line for line in lines if re.match(r"(error(\[E\d+\])?|warning): ", line)]
    found = failed or headlines
    annotation("error", f"{name}: {len(found)} failed",
               "\n".join(dict.fromkeys(found))[-30000:] or "no test or error line found")
    tail = "\n".join(lines[-1200:])
    chunks = [tail[i:i + 30000] for i in range(0, len(tail), 30000)][-4:]
    for index, chunk in enumerate(chunks, 1):
        annotation("error", f"{name}: output's tail {index}/{len(chunks)}", chunk)


def setup_runner():
    runner = Path(os.environ["RUNNER_TEMP"]) / "run"
    (runner.parent / "logs").mkdir(exist_ok=True)
    helper = Path(__file__).resolve()
    runner.write_text('''#!/bin/bash
name=$1
shift
log="$RUNNER_TEMP/logs/$name.log"
"$@" 2>&1 | tee "$log"
status=${PIPESTATUS[0]}
if [ "$status" -ne 0 ]; then
    python3 -I -B "$MACOS_HELPER" annotate "$name" "$log"
fi
exit "$status"
''')
    runner.chmod(0o700)
    with Path(os.environ["GITHUB_ENV"]).open("a") as stream:
        stream.write(f"RUN={runner}\nMACOS_HELPER={helper}\n")


def stop_group(child):
    """Stop the runner and its descendants, even if the runner has already exited."""
    for sig in (signal.SIGINT, signal.SIGTERM):
        signal.signal(sig, signal.SIG_IGN)

    def send(sig):
        try:
            os.killpg(child.pid, sig)
            return True
        except ProcessLookupError:
            return False

    send(signal.SIGTERM)
    until = time.monotonic() + STOP_GRACE
    while True:
        child.poll()
        if not send(0):
            break
        if time.monotonic() >= until:
            send(signal.SIGKILL)
            break
        time.sleep(min(0.1, max(0, until - time.monotonic())))
    child.wait()


def checked(name, command):
    """Keep the saves' reserve and propagate cancels to the command's group (4.4)."""
    timeout = None
    part = os.environ.get("PART")
    if "MACOS_STARTED" in os.environ:
        deadline = int(os.environ["MACOS_STARTED"]) + BOUNDS[part] * 60 - RESERVE
        timeout = max(0, deadline - time.time())
    child = subprocess.Popen([os.environ["RUN"], name, *map(str, command)],
                             stdin=subprocess.DEVNULL, start_new_session=True)
    previous = {sig: signal.getsignal(sig) for sig in (signal.SIGINT, signal.SIGTERM)}

    def cut_short(reason):
        stop_group(child)
        annotate(name, Path(os.environ["RUNNER_TEMP"]) / f"logs/{name}.log")
        raise ValueError(reason)

    def cancelled(sig, _frame):
        # Unwind Popen.wait's lock before reaping the stopped group.
        raise InterruptedError(f"{part}: cut short by {signal.Signals(sig).name}")

    try:
        for sig in previous:
            signal.signal(sig, cancelled)
        try:
            return child.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            cut_short(f"{part}: cut short at its deadline, {RESERVE // 60} minutes "
                      f"before its {BOUNDS[part]}-minute timeout")
        except InterruptedError as error:
            cut_short(str(error))
    finally:
        for sig, handler in previous.items():
            signal.signal(sig, handler)


def require(name, command):
    status = checked(name, command)
    if status:
        raise ValueError(f"{name}: fault, exit status {status}")


def json_document(path, key):
    text = Path(path).read_text()
    for match in re.finditer(r"^\{", text, re.M):
        try:
            value, _ = json.JSONDecoder().raw_decode(text, match.start())
        except ValueError:
            continue
        if isinstance(value, dict) and key in value:
            return value
    raise ValueError(f"{path}: missing JSON with {key}")


def metadata():
    require("metadata", ["cargo", "metadata", "--no-deps", "--format-version", "1"])
    return json_document(Path(os.environ["RUNNER_TEMP"]) / "logs/metadata.log", "packages")


def package_map():
    return tomllib.loads(Path("tools/tiers/packages.toml").read_text())


def package_args(part, meta):
    packages = sorted(p["name"] for p in meta["packages"] if tree_holds(part, p["name"]))
    if not packages:
        raise ValueError(f"{part}: no workspace packages")
    return [arg for package in packages for arg in ("-p", package)]


def build_commands(part, packages, meta):
    if part in PAIRS:
        # A probe still builds the whole part (4.2), including unselected pairs.
        commands = [["tools/tiers/run", *reversed(pair.split("-", 1)), "--", "--no-run"]
                    for pair in PAIRS[part]]
        if part in ("process", "live"):
            commands.append(["cargo", "build", "-p", "temper-box", "--bin", "temper-box"])
        return commands
    args = package_args(part, meta)
    owned = set(args[1::2])
    binaries = {(item["package"], item["bin"]) for package in owned
                for item in packages.get("test-binaries", {}).get(package, [])}
    return [["cargo", "build", "-p", package, "--bin", binary] for package, binary in sorted(binaries)] + [
        ["cargo", "nextest", "run", "--no-run", "--profile", "default", *args]]


def build(part, destination):
    destination = Path(destination)
    destination.unlink(missing_ok=True)
    packages = package_map()
    meta = metadata() if part not in PAIRS else None
    for index, command in enumerate(build_commands(part, packages, meta)):
        name = f"build-{part}-{index}"
        log = Path(os.environ["RUNNER_TEMP"]) / f"logs/{name}.log"
        if checked(name, command):
            build_case(part, log, destination)
            output("build_red", "true")
            output("run_tests", "false")
            return
        compiled = compilations(log)
        workspace = sum(source.startswith("/") for _, source in compiled)
        annotation("notice", name, f"compiled {len(compiled) - workspace} dependencies "
                   f"and {workspace} workspace crates")
    with Path(os.environ["GITHUB_ENV"]).open("a") as stream:
        stream.write("MACOS_BUILT=true\n")
    output("build_red", "false")
    output("run_tests", str(not ids(os.environ.get("IDS", "")) or bool(filterset(ids(os.environ["IDS"])))).lower())


def compilations(log):
    return [(match[1], match[2] or "") for match in re.finditer(
        r"^\s+Compiling (\S+) v\S+(?: \((.*)\))?$", Path(log).read_text(encoding="utf-8", errors="replace"), re.M)]


def run_arguments(part, selected, packages):
    """Commands and selections are shared with the pre-run listings (4.2, 8)."""
    given = os.environ.get("GIVEN") == "true"
    if given and not filterset(selected):
        return []
    if part not in PAIRS:
        args = package_args(part, metadata())
        extra = ["-E", filterset(selected), "--no-tests=warn"] if given else []
        return [(profile, args + extra, "default") for profile in ("default", "fuzzy")]
    result = []
    for pair in PAIRS[part]:
        owned = pair_ids(pair, selected, packages)
        if given and not owned:
            continue
        args = [arg for package in packages[pair] for arg in ("-p", package)]
        extra = []
        expression = filterset(owned)
        ignored = "default"
        if part == "live":
            ignored = "only"
            extra = ["--run-ignored", "only"]
            if pair == "live-temper":
                extra += ["--ignore-default-filter"]
                story = "binary(=live) & test(=a_person_asks_claude_on_an_api_key_to_land_a_change_on_the_owned_forge)"
            else:
                story = "test(/^anthropic_/)"
            expression = f"({story}) & ({expression})" if expression else story
        if expression:
            extra += ["-E", expression]
        if given:
            extra += ["--no-tests=warn"]
        result.append((pair, args + extra, ignored))
    return result


def run_part(part, destination):
    destination = Path(destination)
    destination.unlink(missing_ok=True)
    root = Path(os.environ["RUNNER_TEMP"]) / "macos-runs"
    root.mkdir()
    paths = []
    packages = package_map()
    for profile, args, ignored in run_arguments(part, ids(os.environ.get("IDS", "")), packages):
        if part in PAIRS:
            listing = root / f"{profile}.json"
            # list and run receive identical profile, package, filter and ignored flags.
            require(f"list-{profile}", ["cargo", "nextest", "list", "--message-format", "json",
                                      "--profile", profile, *[arg for arg in args if not arg.startswith("--no-tests=")]])
            shutil.copyfile(Path(os.environ["RUNNER_TEMP"]) / f"logs/list-{profile}.log", listing)
            path = Path("target/nextest") / profile / "junit.xml"
            path.unlink(missing_ok=True)
            tier, tree = profile.split("-", 1)
            # tools/tiers/run owns its package selection and prerequisites.
            extra = args[2 * len(packages[profile]):]
            status = checked(f"run-{profile}", ["tools/tiers/run", tree, tier, "--", *extra])
            if profile.startswith("browser-"):
                compiled = compilations(Path(os.environ["RUNNER_TEMP"]) / f"logs/run-{profile}.log")
                if compiled:
                    annotation("warning", f"run-{profile}: compiled during run",
                               ", ".join(dict.fromkeys(crate for crate, _ in compiled)))
            check_nextest(listing, path, status, ignored)
        else:
            directory = root / profile
            empty = root / "empty-budgets.toml"
            empty.write_text("")
            status = checked(f"run-{profile}", ["tools/tiers/one-process", "--out", directory,
                                               "--budgets", empty, profile, *args])
            check_one_process(directory, status)
            path = directory / "junit.xml"
        paths.append(path)
    merge(paths, destination)


def install():
    """Use the checkout's toolchain and the workflow's checked nextest pin (6)."""
    require("rustup-version", ["rustup", "--version"])
    require("toolchain", ["rustup", "toolchain", "install"])
    root = Path(os.environ["RUNNER_TEMP"])
    version = os.environ["NEXTEST_VERSION"]
    archive = root / "nextest.tar.gz"
    require("nextest-download", ["curl", "-sSfL", "-o", archive,
                                f"https://github.com/nextest-rs/nextest/releases/download/cargo-nextest-{version}/cargo-nextest-{version}-universal-apple-darwin.tar.gz"])
    checksum = root / "nextest.sha256"
    checksum.write_text(f"{os.environ['NEXTEST_SHA256']}  {archive}\n")
    require("nextest-digest", ["shasum", "-a", "256", "-c", checksum])
    bindir = Path.home() / ".cargo/bin"
    bindir.mkdir(parents=True, exist_ok=True)
    require("nextest-install", ["tar", "-xzf", archive, "-C", bindir])
    require("rust-version", ["rustc", "--version"])
    require("nextest-version", ["cargo", "nextest", "--version"])
    require("platform", ["uname", "-a"])


def install_ripgrep():
    """Install smith's checked ripgrep pin (temper/docs/design/testing/macos.md, 6)."""
    root = Path(os.environ["RUNNER_TEMP"])
    version = os.environ["RIPGREP_VERSION"]
    release = f"ripgrep-{version}-aarch64-apple-darwin"
    archive = root / f"{release}.tar.gz"
    require("rg-download", ["curl", "-sSfL", "-o", archive,
                            f"https://github.com/BurntSushi/ripgrep/releases/download/{version}/{release}.tar.gz"])
    checksum = root / "ripgrep.sha256"
    checksum.write_text(f"{os.environ['RIPGREP_SHA256']}  {archive}\n")
    require("rg-digest", ["shasum", "-a", "256", "-c", checksum])
    destination = root / "ripgrep"
    destination.mkdir()
    require("rg-unpack", ["tar", "-xzf", archive, "-C", destination])
    binary = destination / release / "rg"
    if not binary.is_file():
        raise ValueError(f"ripgrep: missing {binary}")
    require("rg-version", [binary, "--version"])
    with Path(os.environ["GITHUB_PATH"]).open("a") as stream:
        stream.write(f"{binary.parent}\n")


def wasm_compiler():
    """Select wasm C tools (temper/docs/design/testing/macos.md, 6)."""
    def query(name, command):
        require(name, command)
        return (Path(os.environ["RUNNER_TEMP"]) / "logs" / (name + ".log")).read_text()

    installed = query("wasm-llvm-installed", ["brew", "list", "--formula", "--versions"])
    formulae = []
    for line in installed.splitlines():
        match = re.fullmatch(r"(llvm(?:@\d+)?)\s+(\d+)[^\n]*", line)
        if match:
            formulae.append((int(match[2]), match[1]))
    if formulae:
        _, formula = max(formulae)
    else:
        require("wasm-llvm-install", ["env", "HOMEBREW_NO_AUTO_UPDATE=1", "brew", "install", "llvm"])
        formula = "llvm"
    prefix = Path(query("wasm-llvm-prefix", ["brew", "--prefix", formula]).strip())
    clang = prefix / "bin/clang"
    archive = prefix / "bin/llvm-ar"
    for tool in (clang, archive):
        if not tool.is_file():
            raise ValueError(f"wasm compiler: missing {tool}")
    targets = query("wasm-cc-targets", [clang, "--print-targets"])
    if not any(line.split()[:1] == ["wasm32"] for line in targets.splitlines()):
        raise ValueError(f"{clang}: no wasm32 target")
    require("wasm-cc-version", [clang, "--version"])
    with Path(os.environ["GITHUB_ENV"]).open("a") as stream:
        stream.write(f"CC_wasm32_unknown_unknown={clang}\nAR_wasm32_unknown_unknown={archive}\n")


def fetch_forge():
    # The first prepare's expected refusal is not a failed check (2).
    root = Path(os.environ["RUNNER_TEMP"]) / "fetch"
    root.mkdir(mode=0o700, exist_ok=True)
    root.chmod(0o700)
    flags = []
    for name in ("config", "data", "cache"):
        directory = root / name
        directory.mkdir(mode=0o700, exist_ok=name == "cache")
        directory.chmod(0o700)
        flags += ["--" + name, str(directory)]
    binary = "target/debug/temper-box"
    require("init", [binary, "init", *flags, "--box"])
    log = Path(os.environ["RUNNER_TEMP"]) / "logs/prepare.log"
    with log.open("w") as stream:
        subprocess.run([binary, "forge", "prepare", *flags, "1", "--listen", "127.0.0.1:3000",
                        "--person", "ada", "ada@example.invalid", "--runner-label", "temper-host:host"],
                       stdin=subprocess.DEVNULL, stdout=stream, stderr=subprocess.STDOUT, check=False)
    print(log.read_text(), flush=True)
    if not (root / "data/forge-1/setup").is_file():
        annotate("prepare", log)
        raise ValueError("prepare: no forge-1/setup")
    require("fetch", [binary, "forge", "fetch", *flags, "1"])
    with Path(os.environ["GITHUB_ENV"]).open("a") as stream:
        for directory, filename, variable in (
            ("forgejo", "forgejo", "TEMPER_TEST_FORGEJO"),
            ("forgejo-runner", "forgejo-runner", "TEMPER_TEST_FORGEJO_RUNNER"),
        ):
            matches = list((root / "cache" / directory).glob(f"*/{filename}"))
            if len(matches) != 1:
                raise ValueError(f"fetch: expected one {filename}, got {len(matches)}")
            require(filename + "-runs", [matches[0], "--version"])
            stream.write(f"{variable}={matches[0]}\n")


def prune(target, meta, dependencies):
    """Keep dependencies alone, including when a target cache was restored (6)."""
    target = Path(target)
    names = {package["name"] for package in meta["packages"]}
    names |= {item["name"] for package in meta["packages"] for item in package["targets"]}

    def spellings(values):
        return {form for name in values for form in (name, name.replace("-", "_"), name.replace("_", "-"))}

    names = spellings(names)
    shared = names & spellings(dependencies)
    for name in sorted({name.replace("_", "-") for name in shared}):
        annotation("warning", "prune: dependency shares a workspace name", name)
    names -= shared
    patterns = [re.compile(rf"{re.escape(name)}(-[0-9a-f]{{16}})?(\..+)?") for name in names]
    profiles = {path.parent for path in target.rglob(".fingerprint") if path.is_dir()}
    ancestors = {parent for profile in profiles for parent in profile.parents
                 if parent == target or target in parent.parents}

    def clean(directory):
        for path in directory.iterdir():
            if directory in profiles and path.name in ("deps", "build", ".fingerprint"):
                for entry in path.iterdir():
                    if any(pattern.fullmatch(entry.name.removeprefix("lib")) for pattern in patterns):
                        remove_path(entry)
            elif path in profiles or path in ancestors:
                clean(path)
            elif directory == target and path.name in (".rustc_info.json", "CACHEDIR.TAG"):
                continue
            else:
                remove_path(path)

    if target.is_dir():
        clean(target)


def prune_cache():
    """Prune in the saves' reserve, retaining all dependency areas if names are unavailable."""
    target = Path("target")
    try:
        started = os.environ.pop("MACOS_STARTED", None)
        try:
            meta = metadata()
        finally:
            if started is not None:
                os.environ["MACOS_STARTED"] = started
        target = Path(meta["target_directory"])
        lock = tomllib.loads(Path("Cargo.lock").read_text())
        dependencies = {package["name"] for package in lock["package"] if "source" in package}
    except (ValueError, KeyError, OSError) as error:
        annotation("warning", "prune: keeping dependency areas whole", error)
        meta = {"packages": []}
        dependencies = set()
    try:
        prune(target, meta, dependencies)
    except OSError as error:
        annotation("warning", "prune: could not remove every output", error)


def remove_path(path):
    if path.is_dir() and not path.is_symlink():
        shutil.rmtree(path)
    else:
        path.unlink(missing_ok=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    plan = commands.add_parser("plan")
    plan.add_argument("--packages", type=Path, required=True)
    commands.add_parser("filterset")
    case = commands.add_parser("build-case")
    case.add_argument("part", choices=(*SETS["all"], "live"))
    case.add_argument("log", type=Path)
    case.add_argument("destination", type=Path)
    merging = commands.add_parser("merge")
    merging.add_argument("destination", type=Path)
    merging.add_argument("files", nargs="*", type=Path)
    one = commands.add_parser("check-one-process")
    one.add_argument("directory", type=Path)
    one.add_argument("status", type=int)
    nxt = commands.add_parser("check-nextest")
    nxt.add_argument("listing", type=Path)
    nxt.add_argument("junit", type=Path)
    nxt.add_argument("status", type=int)
    nxt.add_argument("--run-ignored", choices=("default", "only", "all"), default="default")
    reporting = commands.add_parser("report")
    reporting.add_argument("directory", type=Path)
    reporting.add_argument("destination", type=Path)
    ann = commands.add_parser("annotate")
    ann.add_argument("name")
    ann.add_argument("log", type=Path)
    commands.add_parser("setup-runner")
    for name in ("build", "run", "verdict"):
        sub = commands.add_parser(name)
        sub.add_argument("part", choices=(*SETS["all"], "live"))
        sub.add_argument("destination", type=Path)
    commands.add_parser("install")
    commands.add_parser("install-ripgrep")
    commands.add_parser("wasm-compiler")
    commands.add_parser("fetch-forge")
    commands.add_parser("prune")
    args = parser.parse_args()
    try:
        if args.command == "plan":
            parts, unheld = place(os.environ["SET"], os.environ.get("TESTS", ""),
                                  tomllib.loads(args.packages.read_text()))
            for part in (*SETS["all"], "live"):
                value = parts.get(part, dict(run=False, ids=[]))
                output(part, str(value["run"]).lower())
                output(part + "_ids", "\n".join(value["ids"]))
            output("given", str(bool(ids(os.environ.get("TESTS", "")))).lower())
            output("parts", json.dumps([part for part, value in parts.items() if value["run"]]))
            for item in unheld:
                annotation("notice", "id held by no part", item)
        elif args.command == "filterset":
            print(filterset(ids(os.environ.get("IDS", ""))))
        elif args.command == "build-case":
            build_case(args.part, args.log, args.destination)
        elif args.command == "merge":
            merge(args.files, args.destination)
        elif args.command == "check-one-process":
            check_one_process(args.directory, args.status)
        elif args.command == "check-nextest":
            check_nextest(args.listing, args.junit, args.status, args.run_ignored)
        elif args.command == "report":
            report(json.loads(os.environ["PARTS"]), args.directory, args.destination)
        elif args.command == "annotate":
            annotate(args.name, args.log)
        elif args.command == "setup-runner":
            setup_runner()
        elif args.command == "build":
            build(args.part, args.destination)
        elif args.command == "run":
            run_part(args.part, args.destination)
        elif args.command == "install":
            install()
        elif args.command == "install-ripgrep":
            install_ripgrep()
        elif args.command == "wasm-compiler":
            wasm_compiler()
        elif args.command == "fetch-forge":
            fetch_forge()
        elif args.command == "prune":
            prune_cache()
        elif args.command == "verdict":
            if any(case.find("failure") is not None or case.find("error") is not None
                   for suite in suites(args.destination) for case in suite.iter("testcase")):
                return 1
    except (ValueError, KeyError, OSError, ET.ParseError) as error:
        annotation("error", "macOS fault", error)
        return FAULT
    return 0


if __name__ == "__main__":
    sys.exit(main())

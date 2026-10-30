#!/usr/bin/env python3
"""Opt-in synchronous LOCAL checker; tools/queue/design.md, section 3.8."""

import argparse
from contextlib import contextmanager
import fcntl
import hashlib
import importlib.machinery
import importlib.util
import json
import math
import os
from pathlib import Path
import re
import stat
import sys
import tempfile
import time


DIRECTORY = Path(__file__).resolve().parent
RUNTIME_FILES = ("worker.py", "mq", "always.txt", "read-by-tests.txt", "leads.txt",
                 "train.py", "authority.py", "recovery.py", "durable.py", "publication.py", "coordinators.txt", "restart")
FROZEN = {name: (DIRECTORY / name).read_bytes() for name in RUNTIME_FILES}
LOADER = importlib.machinery.SourceFileLoader("worker_mq", str(DIRECTORY / "mq"))
SPEC = importlib.util.spec_from_loader(LOADER.name, LOADER)
mq = importlib.util.module_from_spec(SPEC)
LOADER.exec_module(mq)
TOKEN = re.compile(r"[A-Za-z0-9][A-Za-z0-9_.-]{0,127}\Z")
OID = re.compile(r"(?:[0-9a-f]{40}|[0-9a-f]{64})\Z")
PACKAGE = re.compile(r"[A-Za-z0-9_][A-Za-z0-9_-]*\Z")


class ObservedFailure(RuntimeError):
    """A synchronous command returned; the observer refused further checking."""


class UncertainCompletion(RuntimeError):
    """A lost heavy client does not establish that its job has stopped."""


def digest(data):
    return hashlib.sha256(data).hexdigest()


def encoded(value):
    return (json.dumps(value, sort_keys=True, allow_nan=False, separators=(",", ":")) + "\n").encode()


def runtime_identity(disk_floor=45):
    mq.publication.coordinators(FROZEN["coordinators.txt"])
    return {"version": 1, "files": {name: digest(data) for name, data in FROZEN.items()},
            "python": list(sys.version_info[:3]), "disk_floor_gb": disk_floor,
            "ratchets": "check-only", "coordinators_sha256": digest(FROZEN["coordinators.txt"])}


def runtime_unchanged():
    if any((DIRECTORY / name).is_symlink() or (DIRECTORY / name).read_bytes() != data
           for name, data in FROZEN.items()):
        raise RuntimeError("running runtime/config files changed; restart from a frozen copy")


def exact_keys(value, keys):
    if not isinstance(value, dict) or set(value) != set(keys):
        raise ValueError(f"expected exactly these fields: {', '.join(keys)}")


def validate_request(request):
    exact_keys(request, ("version", "attempt", "generation", "gate", "parent", "candidate",
                         "branches", "selection", "runtime", "plan"))
    if type(request["version"]) is not int or request["version"] != 1:
        raise ValueError("unsupported request version")
    for name in ("attempt", "generation", "gate"):
        if not isinstance(request[name], str) or not TOKEN.fullmatch(request[name]):
            raise ValueError(f"invalid {name}")
    for name in ("parent", "candidate"):
        if not isinstance(request[name], str) or not OID.fullmatch(request[name]):
            raise ValueError(f"{name} must be a full commit OID")
    branches = request["branches"]
    if not isinstance(branches, list):
        raise ValueError("branches must be a list")
    names = set()
    for branch in branches:
        exact_keys(branch, ("name", "tip"))
        name, tip = branch["name"], branch["tip"]
        if (not isinstance(name, str) or len(name.split("/")) != 3 or
                not name.startswith("ready/") or name in names or
                mq.git(DIRECTORY, "check-ref-format", "refs/heads/" + name, check=False).returncode):
            raise ValueError("invalid or duplicate ready branch name")
        if not isinstance(tip, str) or not OID.fullmatch(tip):
            raise ValueError("ready tip must be a full commit OID")
        names.add(name)
    selected = request["selection"]
    if not (isinstance(selected, str) and selected in ("full", "docs")):
        if (not isinstance(selected, list) or any(not isinstance(name, str) or not PACKAGE.fullmatch(name)
                                                 for name in selected) or
                selected != sorted(set(selected))):
            raise ValueError("selection must be full, docs, or sorted unique packages")
    if not isinstance(request["runtime"], dict) or not isinstance(request["plan"], list):
        raise ValueError("runtime must be an object and plan a list")
    for step in request["plan"]:
        keys = ("name", "suites") if isinstance(step, dict) and step.get("name") == "tools" else (
            "name", "command", "cwd")
        exact_keys(step, keys)
        if "suites" in step:
            if not isinstance(step["suites"], list):
                raise ValueError("tools suites must be a list")
            for suite in step["suites"]:
                exact_keys(suite, ("name", "command", "cwd"))
                validate_command_plan(suite)
        else:
            validate_command_plan(step)
    # The locally computed plan below is authoritative; arbitrary commands are never executed.
    encoded(request)
    return request


def validate_command_plan(step):
    if not isinstance(step["name"], str) or not step["name"]:
        raise ValueError("plan step name must be a nonempty string")
    argv, cwd = step["command"], step["cwd"]
    if (not isinstance(argv, list) or not argv or
            any(not isinstance(arg, str) or "\0" in arg for arg in argv) or not argv[0]):
        raise ValueError("plan command must be a nonempty string argv")
    if (not isinstance(cwd, str) or not cwd or Path(cwd).is_absolute() or
            ".." in Path(cwd).parts or Path(cwd).as_posix() != cwd):
        raise ValueError("plan cwd must be normalized and checkout-relative")


def read_request(path):
    return validate_request(mq.authority.durable.read_json(mq.authority.durable.read_bytes(path)))


def canonical_directory(path):
    path = Path(os.path.abspath(path))
    if any(part.is_symlink() for part in (path, *path.parents)):
        raise ValueError("configured directory contains a symlink")
    if not path.is_dir():
        raise ValueError("configured directory must exist")
    return path


def sync_directory(path):
    fd = os.open(path, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(fd)
    finally:
        os.close(fd)


def atomic_json(path, value):
    mq.authority.durable.publish(path, encoded(value), immutable=path.name in {"request.json", "result.json"},
                                private_parent=path.parent.name != ".git")


@contextmanager
def checkout_ownership(repo):
    with mq.authority.checkout_lock(repo):
        yield repo / ".git/mq-worker-unresolved.json"


def snapshot(repo):
    head, tree = mq.git(repo, "rev-parse", "HEAD", "HEAD^{tree}").stdout.splitlines()
    status = mq.git(repo, "status", "--porcelain", "--untracked-files=all").stdout
    return {"head": head, "tree": tree, "clean": not status, "status": status}


def immutable(repo, expected):
    observed = snapshot(repo)
    if observed != expected:
        raise RuntimeError("candidate HEAD/tree/worktree changed during checking")
    return observed


def verify_objects(repo, request):
    for oid in [request["parent"], request["candidate"], *[b["tip"] for b in request["branches"]]]:
        if mq.git(repo, "cat-file", "-t", oid).stdout.strip() != "commit":
            raise ValueError("requested object is not a commit")
    for oid in [request["parent"], *[b["tip"] for b in request["branches"]]]:
        if mq.git(repo, "merge-base", "--is-ancestor", oid, request["candidate"], check=False).returncode:
            raise ValueError("parent/input tip is not contained in candidate")
    return {"parent": request["parent"], "candidate": request["candidate"],
            "candidate_tree": mq.git(repo, "rev-parse", request["candidate"] + "^{tree}").stdout.strip(),
            "branches": request["branches"]}


def frozen_lines(name):
    return [line.split("#", 1)[0].strip() for line in FROZEN[name].decode().splitlines()
            if line.split("#", 1)[0].strip()]


def graph_identity(repo, metadata):
    if Path(metadata["workspace_root"]) != repo:
        raise ValueError("metadata workspace root differs from candidate checkout")
    members = metadata["workspace_members"]
    packages = {p["id"]: p for p in metadata["packages"]}
    if not members or len(packages) != len(metadata["packages"]) or len(set(members)) != len(members):
        raise ValueError("incomplete/duplicate candidate metadata")
    keys = {}
    for package_id, package in packages.items():
        manifest = None
        if package_id in members:
            path = Path(package["manifest_path"])
            manifest = path.relative_to(repo).as_posix()
            if not path.is_file() or path.is_symlink():
                raise ValueError("workspace manifest is missing or symlinked")
        keys[package_id] = [package["name"], package["version"], package.get("source"), manifest]
    if set(members) - packages.keys():
        raise ValueError("workspace member missing from resolved metadata")
    nodes = metadata["resolve"]["nodes"]
    if {node["id"] for node in nodes} != packages.keys() or len(nodes) != len(packages):
        raise ValueError("incomplete resolve graph")
    graph = [{"package": keys[node["id"]], "dependencies": sorted(
        [keys[dep] for dep in node["dependencies"]], key=encoded)} for node in nodes]
    return {"sha256": digest(encoded(sorted(graph, key=encoded))),
            "packages": sorted(packages[member]["name"] for member in members)}


def choose(repo, request, paths, metadata):
    if request["selection"] == "full":
        return "full"
    locked = None
    if "Cargo.lock" in paths:
        locked = mq.lock_changes(mq.git(repo, "show", request["parent"] + ":Cargo.lock").stdout,
                                 mq.git(repo, "show", request["candidate"] + ":Cargo.lock").stdout, metadata)
    selected = mq.selection(paths, metadata, always=frozen_lines("always.txt"),
                            read_by_tests=frozen_lines("read-by-tests.txt"), locked=locked)
    selected = sorted(selected) if isinstance(selected, set) else selected
    if selected != request["selection"]:
        raise ValueError(f"selection mismatch: computed {selected!r}")
    return request["selection"]


def command_record(repo, name, argv, cwd):
    # Only the ratchet script uses an absolute checkout path in the existing checker.
    prefix = str(repo) + "/"
    return {"name": name, "command": ["./" + arg[len(prefix):] if arg.startswith(prefix) else arg
                                      for arg in argv], "cwd": Path(cwd).relative_to(repo).as_posix()}


def check_plan(repo, selected, paths, metadata):
    plan = [command_record(repo, *command) for command in mq.ratchet_commands(repo, paths, check_only=True)]
    if mq.tools_required(paths):
        plan.append({"name": "tools", "suites": [command_record(repo, *command)
                                                  for command in mq.tools_commands(repo)]})
    plan.extend(command_record(repo, *command) for command in mq.check_commands(
        repo, set(selected) if isinstance(selected, list) else selected, paths, metadata=metadata))
    return plan


def validate_steps(repo, plan, steps):
    """Accept precisely the completed prefix ending at the first red step."""
    if not isinstance(steps, list) or len(steps) > len(plan):
        raise ValueError("invalid step evidence")
    failed = False
    for expected, step in zip(plan, steps):
        if failed or step["name"] != expected["name"]:
            raise ValueError("unexpected/reordered step evidence")
        if expected["name"] == "tools":
            suites = step["suites"]
            if len(suites) != len(expected["suites"]):
                raise ValueError("incomplete tools evidence")
            for suite_plan, suite in zip(expected["suites"], suites):
                validate_step(repo, suite_plan, suite, tools=True)
            if type(step["green"]) is not bool or step["green"] != all(s["green"] for s in suites):
                raise ValueError("inconsistent tools outcome")
            duration(step)
        else:
            validate_step(repo, expected, step)
        failed = not step["green"]
    if not failed and len(steps) != len(plan):
        raise ValueError("incomplete check plan")
    return not failed


def duration(step):
    value = step["duration"]
    if type(value) not in (int, float) or not math.isfinite(value) or value < 0:
        raise ValueError("invalid step duration")


def validate_step(repo, expected, step, *, tools=False):
    if step["name"] != expected["name"] or step["command"] != expected["command"]:
        # Results use the normalized commands, so they can be checked on another host.
        raise ValueError("step command/identity mismatch")
    if step.get("cwd") != expected["cwd"]:
        raise ValueError("step working directory mismatch")
    code = step["returncode"]
    if type(code) is not int or type(step["green"]) is not bool:
        raise ValueError("invalid step outcome")
    green = code == 0 or (not tools and step["name"] in ("build", "default", "fuzzy") and code == 4)
    if step["green"] != green:
        raise ValueError("step outcome disagrees with return code")
    duration(step)
    if "launch_error" in step or "error" in step or code < 0:
        raise RuntimeError("checker infrastructure failure: " + str(step))


def run_job(repo, artifacts, request, *, gate, disk_floor=45, checker=mq.run_checks,
            metadata_loader=mq.load_metadata):
    """No transport or authority effects. Injectable checker/metadata are test seams only."""
    os.umask(0o077)
    validate_request(request)
    repo, artifacts = canonical_directory(repo), canonical_directory(artifacts)
    if repo == artifacts or repo in artifacts.parents or artifacts in repo.parents:
        raise ValueError("artifacts and checkout must be separate directories")
    if repo == DIRECTORY or repo in DIRECTORY.parents:
        raise ValueError("run the frozen worker runtime outside the candidate checkout")
    if mq.git(repo, "rev-parse", "--show-toplevel").stdout.strip() != str(repo):
        raise ValueError("repo must be the checkout root")
    if (not (repo / ".git").is_dir() or (repo / ".git").is_symlink() or
            Path(mq.git(repo, "rev-parse", "--path-format=absolute", "--git-common-dir").stdout.strip()) != repo / ".git"):
        raise ValueError("worker requires an independent clone with its own .git directory")
    if artifacts.stat().st_uid != os.getuid() or stat.S_IMODE(artifacts.stat().st_mode) != 0o700:
        raise ValueError("artifact directory must be owned by this user and mode 0700")
    with checkout_ownership(repo) as marker:
        job = artifacts / request["attempt"]
        job.mkdir(mode=0o700)  # Existing attempts are never overwritten or rerun.
        sync_directory(artifacts)
        log = job / ".checks.partial"
        with log.open("x"):
            pass
        record = {"version": 1, "requested": request, "observed": {"gate": gate,
                  "runtime": runtime_identity(disk_floor)}, "outcome": "INFRA", "error": None,
                  "start": mq.utc_now(), "steps": [], "events": [], "plan": [],
                  "before": None, "after": None, "completion_known": True,
                  "target_cleanup": {"skipped": "not admitted"}}
        admitted = False
        started_ns = time.time_ns()
        try:
            runtime_unchanged()
            if request["gate"] != gate or encoded(request["runtime"]) != encoded(record["observed"]["runtime"]):
                raise ValueError("gate/runtime policy identity mismatch")
            if ((repo / ".git/info/grafts").exists() or
                    mq.git(repo, "for-each-ref", "--format=%(refname)", "refs/replace/").stdout):
                raise ValueError("local history overrides are not permitted")
            if any(entry and not entry.startswith("H ") for entry in
                   mq.git(repo, "ls-files", "-v", "-z").stdout.split("\0")):
                raise ValueError("index flags must not hide candidate worktree changes")
            mq.ensure_clean(repo)
            record["initial"] = snapshot(repo)
            record["observed"].update(verify_objects(repo, request))
            available = mq.free_gb(repo)
            record["capacity"] = {"free_gb": available, "disk_floor_gb": disk_floor}
            if available < disk_floor:
                raise RuntimeError(f"{available:.1f} GB free, below queue disk floor {disk_floor} GB")
            atomic_json(marker, {"attempt": request["attempt"], "generation": request["generation"],
                                 "artifacts": str(job), "candidate": request["candidate"]})
            admitted = True
            mq.git(repo, "switch", "--detach", request["candidate"])
            record["before"] = snapshot(repo)
            if (record["before"]["head"] != request["candidate"] or not record["before"]["clean"] or
                    record["before"]["tree"] != record["observed"]["candidate_tree"]):
                raise RuntimeError("candidate checkout mismatch")
            paths = mq.changed_paths(repo, request["candidate"], base=request["parent"])
            record["changed_paths"] = paths
            # Metadata is loaded only after detaching, including full runs.
            metadata = {}
            if request["selection"] != "docs":
                metadata = metadata_loader(repo)
                record["metadata"] = graph_identity(repo, metadata)
            immutable(repo, record["before"])
            selected = choose(repo, request, paths, metadata)
            record["selection"] = selected
            record["plan"] = check_plan(repo, selected, paths, metadata)
            if record["plan"] != request["plan"]:
                raise ValueError("request check plan mismatch/incomplete plan")

            def observe(step):
                # Save partial suite evidence before checking invariants.
                normalized = dict(step)
                if "command" in step:
                    normalized.update(command_record(repo, step["name"], step["command"], step["cwd"]))
                record["events"].append(normalized)
                if (step.get("command", [None])[0] == "heavy" and
                        (step["returncode"] < 0 or step["returncode"] == 137)):
                    raise UncertainCompletion("heavy client terminated; job completion requires reconciliation")
                try:
                    checked = snapshot(repo)
                    normalized["checked"] = checked
                    if checked != record["before"]:
                        record["identity_violation"] = checked
                        raise RuntimeError("candidate HEAD/tree/worktree changed during checking")
                    runtime_unchanged()
                    if "launch_error" in step or "error" in step or step.get("returncode", 0) < 0:
                        raise RuntimeError("checker infrastructure failure: " + str(step))
                except Exception as error:
                    raise ObservedFailure(str(error)) from error

            record["completion_known"] = False
            try:
                steps = checker(repo, set(selected) if isinstance(selected, list) else selected,
                                paths, log, check_only=True, metadata=metadata, observer=observe,
                                outcomes=record["steps"])
            except ObservedFailure:
                record["completion_known"] = True
                raise
            record["completion_known"] = True
            record["steps"] = steps
            # Preserve observed argv in events; normalize terminal evidence against the plan.
            for step in steps:
                if "command" in step:
                    step.update(command_record(repo, step["name"], step["command"], step["cwd"]))
                if step["name"] == "tools":
                    for suite in step["suites"]:
                        suite.update(command_record(repo, suite["name"], suite["command"], suite["cwd"]))
            green = validate_steps(repo, record["plan"], steps)
            immutable(repo, record["before"])
            record["outcome"] = "green" if green else "red"
        except Exception as error:
            record["error"] = f"{type(error).__name__}: {error}"
            with log.open("a") as output:
                output.write(record["error"] + "\n")
        finally:
            if not record["completion_known"] and record["error"] is None:
                record["error"] = "worker interrupted; checker completion unknown"
            if admitted:
                if record["completion_known"]:
                    try:
                        record["target_cleanup"] = mq.prune_test_executables(repo, disk_floor, started_ns)
                    except Exception as error:
                        record["target_cleanup"] = {"error": str(error)}
                else:
                    record["target_cleanup"] = {"skipped": "completion unknown; reconcile before reuse"}
                try:
                    record["after"] = snapshot(repo)
                    immutable(repo, record["before"])
                    runtime_unchanged()
                except Exception as error:
                    record["outcome"] = "INFRA"
                    previous = record["error"] + "; " if record["error"] else ""
                    record["error"] = previous + f"final identity: {error}"
            record["end"] = mq.utc_now()
            with log.open("rb") as source:
                os.fsync(source.fileno())
                log_digest = hashlib.file_digest(source, "sha256").hexdigest()
                log_bytes = source.tell()
            final_log = job / "checks.log"
            mq.authority.durable.publish(final_log, mq.authority.durable.read_bytes(log), immutable=True)
            log.unlink()
            sync_directory(job)
            record["log"] = {"path": str(final_log), "bytes": log_bytes, "sha256": log_digest,
                             "complete": record["completion_known"]}
            record["reconciliation_required"] = admitted and (not record["completion_known"] or
                "identity_violation" in record or record["before"] is None or record["after"] != record["before"])
            atomic_json(job / "result.json", record)
            if admitted and not record["reconciliation_required"]:
                marker.unlink()
                sync_directory(marker.parent)
        return record


def main():
    os.umask(0o077)
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--disk-floor", type=mq.positive, default=45)
    sub = parser.add_subparsers(dest="action", required=True)
    sub.add_parser("identity")
    run = sub.add_parser("run")
    run.add_argument("--enable-local", action="store_true", required=True)
    run.add_argument("--repo", type=Path, required=True)
    run.add_argument("--artifacts", type=Path, required=True)
    run.add_argument("--request", type=Path, required=True)
    run.add_argument("--gate", required=True)
    args = parser.parse_args()
    try:
        runtime_unchanged()
        if args.action == "identity":
            print(json.dumps(runtime_identity(args.disk_floor), sort_keys=True))
            return 0
        result = run_job(args.repo, args.artifacts, read_request(args.request), gate=args.gate,
                         disk_floor=args.disk_floor)
        print(str(args.artifacts / result["requested"]["attempt"] / "result.json"))
        return {"green": 0, "red": 1, "INFRA": 2}[result["outcome"]]
    except (OSError, ValueError, RuntimeError) as error:
        print(f"worker: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())

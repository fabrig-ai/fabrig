"""Whole-target eligibility and observations (tools/slots/design.md, 5)."""

from contextlib import ExitStack, contextmanager
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import re
import runpy
import shutil
import socket
import stat
import subprocess
import sys
import time

sys.dont_write_bytecode = True
L = runpy.run_path(str(Path(__file__).resolve().with_name("lifetime.py")))
no_links, identity = L["no_links"], L["identity"]
read, write, locked = L["read"], L["write"], L["locked"]


def command(*argv):
    result = subprocess.run(argv, text=True, capture_output=True, stdin=subprocess.DEVNULL)
    if result.returncode:
        raise RuntimeError(f"{argv[0]} exited {result.returncode}: {result.stderr.strip()}")
    return result.stdout


def units(slot, *, codex=False):
    evidence = []
    rows = command("systemctl", "--user", "list-units", "--type=service", "--plain",
                   "--no-legend", "--state=active,activating,reloading,deactivating,refreshing")
    for row in rows.splitlines():
        name = row.split()[0]
        heavy = "-heavy-" in name
        is_codex = name.startswith(("codex-run-", "codex-owner-")) or name == "codex-run.service"
        if not name.endswith(".service") or not (heavy or codex and is_codex):
            continue
        # This reserved observer has no build cwd. It is never a payload/wrapper.
        if heavy and name.endswith("-reconcile.service"):
            evidence.append(f"observer {name}")
            continue
        props = dict(line.split("=", 1) for line in command(
            "systemctl", "--user", "show", "-p", "WorkingDirectory,ActiveState,LoadState", name
        ).splitlines() if "=" in line)
        if props.get("ActiveState") in ("inactive", "failed"):
            if props.get("LoadState") in ("loaded", "not-found"):
                evidence.append(f"inactive {name}")
                continue
        if (props.get("LoadState") != "loaded" or props.get("ActiveState") not in
                ("active", "activating", "reloading", "deactivating", "refreshing")):
            raise RuntimeError(f"missing heavy unit properties or unknown service: {name}")
        directory = props.get("WorkingDirectory", "").removeprefix("-")
        if heavy and (not directory or not Path(directory).is_absolute()):
            raise RuntimeError(f"missing heavy unit properties: {name}")
        if directory and Path(directory).is_absolute() and Path(directory).resolve().is_relative_to(slot):
            raise RuntimeError(f"live service {name} cwd={directory}")
        # Codex units have no systemd cwd. Run metadata and /proc are mandatory too.
        if is_codex and codex and not directory:
            run_id = name.removeprefix("codex-run-").removesuffix(".service")
            evidence.append(f"Codex service {name}")
            if name.startswith("codex-owner-"):
                raise RuntimeError(f"owner service without slot cwd cannot be excluded: {name}")
            yield ("run", run_id)
    yield ("evidence", evidence)


def process_class(comm, cgroup, uid):
    """Only protected host parents qualify; shells and task children never do."""
    prefix = f"0::/user.slice/user-{uid}.slice/"
    if comm in ("systemd", "(sd-pam)") and cgroup == prefix + f"user@{uid}.service/init.scope":
        return "user manager"
    if comm == "sshd-session" and re.fullmatch(re.escape(prefix) + r"session-\d+\.scope", cgroup):
        return "SSH session parent"
    return None


def processes(slot, proc=Path("/proc")):
    evidence = []
    uid = os.getuid()
    for entry in proc.iterdir():
        if not entry.name.isdecimal():
            continue
        try:
            if entry.stat().st_uid != uid:
                continue
            try:
                directory = os.readlink(entry / "cwd")
            except PermissionError:
                comm = (entry / "comm").read_text().strip()
                cgroup = (entry / "cgroup").read_text().strip()
                classification = process_class(comm, cgroup, uid)
                if classification:
                    evidence.append(f"protected pid={entry.name} {classification}")
                    continue
                # ssh-agent's service cwd must be explicitly outside the slots root.
                if comm == "ssh-agent" and cgroup.endswith("/app.slice/ssh-agent.service"):
                    props = dict(line.split("=", 1) for line in command(
                        "systemctl", "--user", "show", "-p", "WorkingDirectory,ActiveState,LoadState",
                        "ssh-agent.service").splitlines() if "=" in line)
                    wd = props.get("WorkingDirectory", "").removeprefix("!").removeprefix("-")
                    if (props.get("LoadState") == "loaded" and props.get("ActiveState") == "active"
                            and Path(wd).is_absolute() and not Path(wd).resolve().is_relative_to(slot.parent)):
                        evidence.append(f"protected pid={entry.name} ssh-agent cwd={wd}")
                        continue
                raise RuntimeError(f"unreadable own-user cwd pid={entry.name} comm={comm} cgroup={cgroup}")
            if not directory.startswith("/"):
                raise RuntimeError(f"unknown own-user cwd pid={entry.name}")
            directory = directory.removesuffix(" (deleted)")
            if Path(directory).is_relative_to(slot):
                raise RuntimeError(f"own-user process pid={entry.name} cwd={directory}")
        except (FileNotFoundError, ProcessLookupError):
            # A vanished process cannot start work; its surviving children are scanned.
            continue
    return [*evidence, "own-user cwd scan completed"]


def builder(slot, home, host=None):
    canonical = L["slot_path"](slot, home)
    if canonical != slot or slot.parent != Path(home) / "src/fabrig/slots":
        raise RuntimeError("whole-target recovery requires an actual numeric slot root")
    host = host or socket.gethostname().split(".")[0]
    actual_hosts = {f"debian-16gb-hel1-{number}": f"hetzner{number}" for number in (3, 6, 7)}
    host = actual_hosts.get(host, host)
    if host not in ("hetzner3", "hetzner6", "hetzner7"):
        raise RuntimeError(f"not a builder: {host}")
    inventory = Path(__file__).with_name("hosts").read_text().splitlines()
    shares = dict(part.split("=", 1) for row in inventory if row.strip() and
                  not row.startswith("#") and row.split()[0] == host for part in row.split()[1:])
    if slot.name not in shares:
        raise RuntimeError(f"slot absent from builder inventory: {host}:{slot.name}")
    return host, shares[slot.name]


@contextmanager
def maintenance(slot, home, *, release_hold=None):
    directory = L["state_root"](home) / slot.name
    L["private_directory"](directory)
    with ExitStack() as stack:
        stack.enter_context(locked(directory / "lease"))
        for path in directory.iterdir():
            no_links(path)
            if (path.name.endswith(".hold") and release_hold is None or path.name.startswith(".write-")):
                raise RuntimeError(f"durable uncertain/live lifetime: {path}")
            if path.name not in ("lease", "ack.json", "reclaim.json", "owner-notice.json") and not re.fullmatch(
                    r"[0-9a-f]{32}\.(hold|released\.json)", path.name):
                raise RuntimeError(f"unknown maintenance state: {path}")
            if path.suffix == ".json":
                read(path)
        journal = directory / "reclaim.json"
        if journal.exists() and read(journal).get("status") != "completed":
            raise RuntimeError(f"interrupted reclaim needs operator reconciliation: {journal}")
        # The legacy worker uses the sibling worker lock and mq-worker marker.
        gitdir = slot / ".git"
        no_links(gitdir)
        if not gitdir.is_dir():
            raise RuntimeError("recovery requires a standalone .git directory")
        for path in (slot.with_name(slot.name + ".queue.lock"),
                     slot.with_name(slot.name + ".worker.lock"),
                     gitdir / "worker.lock", gitdir / "precheck.lock"):
            # The sibling locks cover the precheck/worker entrance. Observe an
            # existing legacy inner lock without adding files to source metadata.
            if path.parent == gitdir and not path.exists() and not path.is_symlink():
                continue
            stack.enter_context(locked(path))
        markers = [gitdir / name for name in ("precheck.unresolved", "mq-worker-unresolved.json",
                                              "worker.unresolved", "queue.unresolved", "index.lock")]
        markers.extend(slot.with_name(slot.name + suffix) for suffix in
                       (".precheck.unresolved", ".worker.unresolved", ".queue.unresolved"))
        for path in markers:
            no_links(path)
            if path.exists():
                raise RuntimeError(f"unresolved ownership: {path}")
        yield directory


def history(slot, home):
    root = Path(os.path.abspath(os.environ.get("DEV_ROOT", str(Path(home) / "src/rust")))) / "tmp/codex-run"
    no_links(root)
    if not root.is_dir():
        raise RuntimeError("no trustworthy Codex run history")
    matches, by_id = [], {}
    for run in sorted(root.iterdir()):
        if run.name == "runs.tsv":
            continue
        no_links(run)
        if not run.is_dir():
            raise RuntimeError(f"unknown Codex history entry: {run}")
        meta = run / "meta"
        meta_id = L["fingerprint"](meta)
        values = {}
        for line in meta.read_text().splitlines():
            key, separator, value = line.partition("=")
            if not separator or key in values:
                raise RuntimeError(f"corrupt run metadata: {meta}")
            values[key] = value
        directory = Path(values.get("dir", ""))
        if not directory.is_absolute():
            raise RuntimeError(f"unknown run directory: {run}")
        no_links(directory)
        by_id[run.name] = directory
        if not directory.is_relative_to(slot):
            continue
        started = datetime.fromisoformat(values["started"].replace("Z", "+00:00"))
        if started.utcoffset() is None:
            raise RuntimeError(f"unknown start time: {run}")
        ended = L["fingerprint"](run / "ended")
        report = Path(values["report"])
        if not report.is_absolute():
            raise RuntimeError(f"unknown report: {run}")
        report_id = L["fingerprint"](report)
        if ended["identity"][3] < int(started.timestamp() * 1e9):
            raise RuntimeError(f"completion predates start: {run}")
        if ended["identity"][3] > time.time_ns():
            raise RuntimeError(f"completion time is in the future: {run}")
        if L["fingerprint"](meta) != meta_id:
            raise RuntimeError(f"run changed: {run}")
        matches.append({"id": run.name, "meta": meta_id, "ended": ended, "report": report_id,
                        "started": values["started"], "owner": values["to"]})
    if not matches:
        raise RuntimeError("no completed slot run; old slots require explicit evidence")
    latest = max(matches, key=lambda run: (run["started"], run["id"]))
    return {"runs": matches, "latest": latest["id"], "owner": latest["owner"],
            "completed_ns": max(run["ended"]["identity"][3] for run in matches)}, by_id


def target_identity(slot):
    target = slot / "target"
    no_links(target)
    if not target.is_dir() or target.stat().st_dev != slot.stat().st_dev:
        raise RuntimeError("target absent, not a directory, or on another device")
    return identity(target)


def snapshot(slot, home):
    host, share = builder(slot, home)
    history_value, runs = history(slot, home)
    evidence = []
    for kind, value in units(slot, codex=True):
        if kind == "run":
            if value not in runs:
                raise RuntimeError(f"live Codex unit with unknown run: {value}")
            if runs[value].is_relative_to(slot):
                raise RuntimeError(f"live Codex run: {value}")
        else:
            evidence.extend(value)
    evidence.extend(processes(slot))
    return {"slot": str(slot), "slot_identity": identity(slot), "target_identity": target_identity(slot),
            "root_identity": identity(slot.parent), "host": host, "share": share,
            "head": L["fingerprint"](slot / ".git/HEAD"), "history": history_value}, evidence


def acknowledge(number, home, by, reason):
    if not number.isascii() or not number.isdecimal():
        raise RuntimeError("acknowledgement requires a numeric slot number")
    slot = Path(home) / "src/fabrig/slots" / number
    builder(slot, home)
    with maintenance(slot, home) as directory:
        value, evidence = snapshot(slot, home)
        value.update(by=by, reason=reason, acknowledged=datetime.now(timezone.utc).isoformat(),
                     assertions=["report-handled", "no-pending-resume", "exclusive-lease-use"], evidence=evidence)
        write(directory / "ack.json", value)
        return f"slot {number}: acknowledged by={by} reason={reason} latest={value['history']['latest']} owner={value['history']['owner']}"


def release_hold(number, token, home, by, reason):
    if (not number.isascii() or not number.isdecimal() or len(token) != 32
            or any(c not in "0123456789abcdef" for c in token)):
        raise RuntimeError("release requires a numeric slot and exact 32-hex hold token")
    slot = Path(home) / "src/fabrig/slots" / number
    builder(slot, home)
    name = token + ".hold"
    with maintenance(slot, home, release_hold=name) as directory:
        hold = read(directory / name)
        if hold.get("slot") != str(slot) or hold.get("identity") != identity(slot):
            raise RuntimeError("hold does not match the slot identity")
        _, evidence = snapshot(slot, home)
        # Explicit reconciliation is separate from report acknowledgement. A
        # missing ended marker or unknown observation still refuses this API.
        write(directory / (token + ".released.json"),
              {"hold": hold, "by": by, "reason": reason, "evidence": evidence,
               "assertion": "all commands completed; no pending resume", "time": time.time_ns()})
        (directory / name).unlink()
        L["sync"](directory)
        return f"slot {number}: hold {token} released by={by} reason={reason}; fresh report acknowledgement required"


def eligible(slot, home, directory):
    ack = read(directory / "ack.json")
    current, evidence = snapshot(slot, home)
    if (any(ack.get(key) != value for key, value in current.items()) or
            ack.get("assertions") != ["report-handled", "no-pending-resume", "exclusive-lease-use"]
            or not isinstance(ack.get("by"), str) or not ack["by"].strip()
            or not isinstance(ack.get("reason"), str) or not ack["reason"].strip()):
        raise RuntimeError("acknowledgement stale or incomplete; check latest report/resume and acknowledge again")
    return ack, evidence


def target_size(slot):
    target = slot / "target"
    expected = target_identity(slot)
    device = slot.stat().st_dev
    # Bind mounts on the same device also must not be traversed.
    for line in Path("/proc/self/mountinfo").read_text().splitlines():
        mount = re.sub(r"\\([0-7]{3})", lambda m: chr(int(m[1], 8)), line.split()[4])
        if Path(mount).is_relative_to(target):
            raise RuntimeError(f"mount beneath target: {mount}")
    size = 0
    fd = os.open(target, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
    try:
        info = os.fstat(fd)
        if [info.st_dev, info.st_ino] != expected:
            raise RuntimeError("target replaced before measurement")
        def fail(error):
            raise error
        for parent, directories, files, directory_fd in os.fwalk(
                ".", dir_fd=fd, follow_symlinks=False, onerror=fail):
            for name in [*directories, *files]:
                info = os.stat(name, dir_fd=directory_fd, follow_symlinks=False)
                if info.st_dev != device:
                    raise RuntimeError(f"device boundary in target: {parent}/{name}")
                if stat.S_ISREG(info.st_mode):
                    size += info.st_blocks * 512
    finally:
        os.close(fd)
    return size


def recover(slots, home, below, dry_run, free, record):
    candidates = []
    for slot in slots:
        try:
            builder(slot, home)
            with maintenance(slot, home) as directory:
                ack, _ = eligible(slot, home, directory)
                candidates.append((ack["history"]["completed_ns"], slot.name, slot))
        except (OSError, ValueError, KeyError, RuntimeError) as error:
            record(f"slot {slot.name}: whole-target refused ({error})")
    for completed, _, slot in sorted(candidates):
        if free() > below:
            break
        try:
            with maintenance(slot, home) as directory:
                ack, evidence = eligible(slot, home, directory)
                size = target_size(slot)
                before = free()
                if before > below:
                    break
                fields = (f"owner={ack['history']['owner']} ack-by={ack['by']} ack-reason={ack['reason']} "
                          f"latest={ack['history']['latest']} target={slot / 'target'} "
                          f"age-seconds={(time.time_ns() - completed) / 1e9:.3f} allocated-GB={size / 1e9:.3f} "
                          f"before={before:.3f} evidence={json.dumps(evidence)}")
                if dry_run:
                    record(f"slot {slot.name}: would-reclaim {fields} after={free():.3f}")
                    continue
                # Durable intent and owner notice precede mutation. Logging is local;
                # post delivery never controls cleanup success or acknowledges reports.
                write(directory / "reclaim.json", {"status": "uncertain", "ack": ack, "before": before})
                record(f"slot {slot.name}: reclaim-start {fields}")
                eligible(slot, home, directory)
                fd = os.open(slot, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
                try:
                    if ([os.fstat(fd).st_dev, os.fstat(fd).st_ino] != ack["slot_identity"]
                            or identity(slot.parent) != ack["root_identity"]
                            or identity(slot) != ack["slot_identity"]
                            or target_identity(slot) != ack["target_identity"]):
                        raise RuntimeError("slot/target replaced before deletion")
                    # Consume eligibility before touching generated data; no retry after a crash.
                    (directory / "ack.json").unlink()
                    L["sync"](directory)
                    if not shutil.rmtree.avoids_symlink_attacks:
                        raise RuntimeError("fd-safe rmtree is unavailable")
                    shutil.rmtree("target", dir_fd=fd)
                    os.fsync(fd)
                finally:
                    os.close(fd)
                after = free()
                write(directory / "reclaim.json", {"status": "completed", "ack": ack,
                                                   "before": before, "after": after})
                notice = {"owner": ack["history"]["owner"], "slot": str(slot), "before": before,
                          "after": after, "message": "target reclaimed; next build is cold", "ack_by": ack["by"]}
                write(directory / "owner-notice.json", notice)
                record(f"slot {slot.name}: reclaimed {fields} after={after:.3f} owner-notice={directory / 'owner-notice.json'}")
        except (OSError, ValueError, KeyError, RuntimeError) as error:
            record(f"slot {slot.name}: whole-target refused ({error})")

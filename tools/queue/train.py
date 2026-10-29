"""Controller and explicit adapters; tools/queue/design.md, 3.9–3.12.

Operation selects this library only with a private --train-config. Adapters
remain mandatory; authority.GitAuthority alone owns origin effects.
"""

import copy
from contextlib import contextmanager
import importlib.util
import json
import os
from pathlib import Path, PurePosixPath
import stat
import tempfile


DIRECTORY = Path(__file__).resolve().parent
SPEC = importlib.util.spec_from_file_location("train_worker", DIRECTORY / "worker.py")
worker = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(worker)
mq = worker.mq
RECOVERY_SPEC = importlib.util.spec_from_file_location("train_recovery", DIRECTORY / "recovery.py")
recovery = importlib.util.module_from_spec(RECOVERY_SPEC)
RECOVERY_SPEC.loader.exec_module(recovery)
durable = recovery.durable
authority_module = mq.authority
VERSION = 1
CONTROLLER_BYTES = Path(__file__).read_bytes()


def clone(value):
    return copy.deepcopy(value)


read_json = durable.read_json
private_directory = durable.directory


def write_bytes(path, data):
    durable.publish(path, data, immutable=True)


class StateStore:
    """Versioned recovery source. JSONL summaries are never replayed as state."""

    def __init__(self, directory):
        self.directory = private_directory(directory)
        self.path = self.directory / "state.json"

    def load(self):
        if not self.path.exists() and not self.path.is_symlink():
            return None
        state = read_json(durable.read_bytes(self.path))
        try:
            recovery.validate_state(state, validate_request=worker.validate_request)
        except (KeyError, TypeError, AttributeError, OverflowError) as error:
            raise ValueError("malformed recovery state") from error
        return state

    def save(self, state):
        worker.atomic_json(self.path, state)


class Preparer:
    """Owns one independent preparation clone; never touches a gate checkout.

    metadata_loader is an explicit dependency so a test cannot accidentally run
    Cargo. A future production adapter must supply candidate metadata without
    building, under the frozen generation's policy.
    """

    def __init__(self, repo, artifacts, *, metadata_loader, disk_floor=45):
        self.repo = worker.canonical_directory(repo)
        self.artifacts = private_directory(artifacts)
        self.metadata_loader = metadata_loader
        self.disk_floor = disk_floor
        self._ownership = None
        if (not (self.repo / ".git").is_dir() or (self.repo / ".git").is_symlink() or
                mq.git(self.repo, "rev-parse", "--show-toplevel").stdout.strip() != str(self.repo) or
                Path(mq.git(self.repo, "rev-parse", "--path-format=absolute", "--git-common-dir").stdout.strip())
                != self.repo / ".git"):
            raise ValueError("preparation requires an independent clone root")
        if (self.repo == self.artifacts or self.repo in self.artifacts.parents or
                self.artifacts in self.repo.parents):
            raise ValueError("preparation artifacts must be outside checkout")

    @contextmanager
    def ownership(self):
        if self._ownership is not None:
            self._ownership.guard()
            yield
            return
        with authority_module.Lock(self.repo.with_name(self.repo.name + ".queue.lock")) as lease:
            authority_module.checkout_guard(self.repo)
            self._ownership = lease
            try:
                yield
            finally:
                self._ownership = None

    def discover(self, tried):
        with self.ownership():
            mq.git(self.repo, "fetch", "--prune", "origin")
            return mq.list_candidates(self.repo, tried)

    def pin(self, attempt, name, oid):
        ref = f"refs/mq/jobs/{attempt}/{name}"
        if not worker.TOKEN.fullmatch(attempt) or not worker.OID.fullmatch(oid):
            raise ValueError("invalid pin identity")
        mq.git(self.repo, "check-ref-format", ref)
        current = mq.git(self.repo, "rev-parse", "--verify", ref, check=False)
        if current.returncode == 0:
            if current.stdout.strip() != oid:
                raise ValueError("immutable pin already names a different object")
        else:
            mq.git(self.repo, "update-ref", ref, oid, "0" * len(oid))
        return ref

    def expectation(self, request):
        """Compute from controller objects at exact C, never from worker claims."""
        with self.ownership():
            return self._expectation(request)

    def _expectation(self, request):
        worker.runtime_unchanged()
        self.verify_history()
        objects = worker.verify_objects(self.repo, request)
        mq.ensure_clean(self.repo)
        mq.git(self.repo, "switch", "--detach", request["candidate"])
        before = worker.snapshot(self.repo)
        paths = mq.changed_paths(self.repo, request["candidate"], base=request["parent"])
        metadata = {} if request["selection"] == "docs" else self.metadata_loader(self.repo)
        graph = None if request["selection"] == "docs" else worker.graph_identity(self.repo, metadata)
        selected = worker.choose(self.repo, request, paths, metadata)
        plan = worker.check_plan(self.repo, selected, paths, metadata)
        worker.immutable(self.repo, before)
        return {"objects": objects, "changed_paths": paths, "metadata": graph,
                "selection": selected, "plan": plan}

    def prepare_candidate(self, record, runtime):
        repo, attempt, parent = self.repo, record["id"], record["parent"]
        self.verify_history()
        if mq.free_gb(repo) < self.disk_floor or mq.free_gb(self.artifacts) < self.disk_floor:
            raise RuntimeError("preparation/artifact disk floor refusal")
        job = self.artifacts / attempt
        job.mkdir(mode=0o700)
        worker.sync_directory(job.parent)
        pins = [self.pin(attempt, "parent", parent)]
        for index, branch in enumerate(record["inputs"]):
            pins.append(self.pin(attempt, f"input-{index}", branch["tip"]))
        merged, conflicts = mq.build_batch(repo, record["inputs"], base=parent)
        chain, before = [], parent
        for branch in merged:
            contained = before == branch["commit"]
            chain.append({**branch, "before": before, "contained": contained,
                          "message": mq.git(repo, "show", "-s", "--format=%B", branch["commit"]).stdout})
            before = branch["commit"]
        paths = mq.changed_paths(repo, before, base=parent)
        steps, baselines = [], []
        log_path = job / ".prepare.partial"
        with log_path.open("x+") as log:
            os.fchmod(log.fileno(), 0o600)
            for name, argv, cwd in mq.ratchet_commands(repo, paths):
                prior_head = mq.git(repo, "rev-parse", "HEAD").stdout.strip()
                step = mq.run_step(name, argv, cwd, log, runner=mq.subprocess.run, clock=mq.time.monotonic)
                steps.append(step)
                if mq.git(repo, "rev-parse", "HEAD").stdout.strip() != prior_head:
                    raise RuntimeError("ratchet changed prepared HEAD")
                if step["returncode"] == 1:
                    mq.ensure_clean(repo)
                    log.flush()
                    os.fsync(log.fileno())
                    candidate = mq.git(repo, "rev-parse", "HEAD").stdout.strip()
                    pins.append(self.pin(attempt, "candidate", candidate))
                    write_bytes(job / "prepare.log", durable.read_bytes(log_path))
                    log_path.unlink()
                    worker.sync_directory(job)
                    return {"chain": chain, "conflicts": conflicts, "pins": pins,
                            "preparation": steps, "outcome": "red", "request": None,
                            "candidate": candidate, "baseline_commits": baselines,
                            "full": True}
                if not step["green"]:
                    raise RuntimeError("ratchet preparation infrastructure failure: " + str(step))
                baseline = f"{cwd.name}/scripts/old-forms.txt"
                if mq.git(repo, "diff", "--name-only", "--", baseline).stdout:
                    mq.git(repo, "add", "--", baseline)
                    mq.git(repo, "commit", "-m", f"Lower {cwd.name} old-form baselines")
                    baselines.append({"path": baseline,
                                      "commit": mq.git(repo, "rev-parse", "HEAD").stdout.strip()})
            log.flush()
            os.fsync(log.fileno())
        write_bytes(job / "prepare.log", durable.read_bytes(log_path))
        log_path.unlink()
        worker.sync_directory(job)
        mq.ensure_clean(repo)
        candidate = mq.git(repo, "rev-parse", "HEAD").stdout.strip()
        pins.append(self.pin(attempt, "candidate", candidate))
        # Include baseline commits in the explicit tree comparison. If they
        # broaden selection, full is a conservative escalation of legacy policy.
        paths = mq.changed_paths(repo, candidate, base=parent)
        metadata = {}
        docs = all(path.startswith("tools/") or (path.endswith(".md") and not
                   set(PurePosixPath(path).parts[:-1]) & {"src", "tests", "benches", "examples"})
                   for path in paths)
        if record["full"] or not docs:
            metadata = self.metadata_loader(repo)
        locked = None
        if "Cargo.lock" in paths:
            locked = mq.lock_changes(mq.git(repo, "show", parent + ":Cargo.lock").stdout,
                                     mq.git(repo, "show", candidate + ":Cargo.lock").stdout, metadata)
        selected = "full" if record["full"] else mq.selection(
            paths, metadata, always=worker.frozen_lines("always.txt"),
            read_by_tests=worker.frozen_lines("read-by-tests.txt"), locked=locked)
        selected = sorted(selected) if isinstance(selected, set) else selected
        if selected != "docs" and not metadata:
            metadata = self.metadata_loader(repo)
        request = {"version": 1, "attempt": attempt, "generation": record["generation"],
                   "gate": record["gate"], "parent": parent, "candidate": candidate,
                   "branches": [{"name": b["name"], "tip": b["tip"]} for b in merged],
                   "selection": selected, "runtime": clone(runtime),
                   "plan": worker.check_plan(repo, selected, paths, metadata)}
        worker.validate_request(request)
        expected = self.expectation(request)
        if request["plan"] != expected["plan"]:
            raise ValueError("preparation plan changed")
        evidence = {"chain": chain, "conflicts": conflicts, "pins": pins,
                    "preparation": steps, "outcome": "conflict" if conflicts and not merged else None,
                    "baseline_commits": baselines, "candidate": candidate,
                    "request": request, "expected": expected}
        verify_chain(repo, {**record, **evidence})
        evidence["bundle"] = self.export_bundle(attempt, pins, job)
        write_bytes(job / "request.json", worker.encoded(request))
        return evidence

    def verify_history(self):
        if ((self.repo / ".git/info/grafts").exists() or
                mq.git(self.repo, "for-each-ref", "--format=%(refname)", "refs/replace/").stdout):
            raise ValueError("preparation history overrides are not permitted")
        if any(entry and not entry.startswith("H ") for entry in
               mq.git(self.repo, "ls-files", "-v", "-z").stdout.split("\0")):
            raise ValueError("preparation index flags must not hide changes")

    def export_bundle(self, attempt, pins, job):
        path = job / "candidate.bundle"
        fd, name = tempfile.mkstemp(prefix=".bundle-", dir=job)
        os.close(fd)
        temporary = Path(name)
        try:
            mq.git(self.repo, "bundle", "create", str(temporary), *pins)
            temporary.chmod(0o600)
            with temporary.open("rb") as source:
                os.fsync(source.fileno())
            data = durable.read_bytes(temporary)
            write_bytes(path, data)
        finally:
            temporary.unlink(missing_ok=True)
            worker.sync_directory(job)
        return {"path": str(path), "bytes": len(data), "sha256": worker.digest(data),
                "refs": {ref: mq.git(self.repo, "rev-parse", ref).stdout.strip() for ref in pins}}


def import_bundle(repo, bundle, request):
    """Local transport building block: verify all bytes/refs before object import.

    No checkout mutation, origin fetch or speculative origin refs. The transport
    must hold its checkout ownership boundary before calling this helper.
    """
    worker.validate_request(request)
    path = Path(bundle["path"])
    if not path.is_absolute() or path != path.resolve():
        raise ValueError("bundle path must be absolute and canonical")
    if not isinstance(bundle.get("refs"), dict) or any(
            not isinstance(ref, str) or not ref.startswith(f"refs/mq/jobs/{request['attempt']}/") or
            not isinstance(value, str) or not worker.OID.fullmatch(value) or
            mq.git(repo, "check-ref-format", ref, check=False).returncode for ref, value in bundle["refs"].items()):
        raise ValueError("invalid bundle refs")
    data = durable.checked(path, bundle)
    if len(data) != bundle["bytes"] or worker.digest(data) != bundle["sha256"]:
        raise ValueError("corrupt bundle")
    heads = mq.git(repo, "bundle", "list-heads", bundle["path"]).stdout.splitlines()
    if {ref: oid for oid, ref in (line.split() for line in heads)} != bundle["refs"]:
        raise ValueError("bundle objects/refs differ from preparation")
    mq.git(repo, "bundle", "verify", bundle["path"])
    mq.git(repo, "bundle", "unbundle", bundle["path"])
    worker.verify_objects(repo, request)


def verify_chain(repo, record):
    inputs = [{"name": b["name"], "tip": b["tip"]} for b in record["inputs"]]
    by_name = {b["name"]: b["tip"] for b in inputs}
    names = [b["name"] for b in [*record["chain"], *record["conflicts"]]]
    if len(set(names)) != len(names) or set(names) != set(by_name):
        raise ValueError("prepared chain/conflicts do not partition pinned inputs")
    for branch in [*record["chain"], *record["conflicts"]]:
        if by_name[branch["name"]] != branch["tip"]:
            raise ValueError("prepared input tip differs from reservation")
    if [b["name"] for b in record["chain"]] != [b["name"] for b in inputs if b["name"] in
                                                             {m["name"] for m in record["chain"]}]:
        raise ValueError("prepared input order differs from reservation")
    before = record["parent"]
    for branch in record["chain"]:
        if branch["before"] != before:
            raise ValueError("merge chain parent mismatch")
        commit, tip = branch["commit"], branch["tip"]
        if branch["contained"]:
            if commit != before or mq.git(repo, "merge-base", "--is-ancestor", tip, before,
                                          check=False).returncode:
                raise ValueError("invalid contained input")
        else:
            parents = mq.git(repo, "show", "-s", "--format=%P", commit).stdout.strip().split()
            message = mq.git(repo, "show", "-s", "--format=%B", commit).stdout
            if (parents != [before, tip] or message != branch["message"] or
                    message.splitlines()[0] != f"Merge {branch['name']}"):
                raise ValueError("merge chain identity/message mismatch")
        before = commit
    for baseline in record["baseline_commits"]:
        commit, path = baseline["commit"], baseline["path"]
        if path not in {f"{tree}/scripts/old-forms.txt" for tree in mq.TREES}:
            raise ValueError("unexpected baseline path")
        if mq.git(repo, "show", "-s", "--format=%P", commit).stdout.strip() != before:
            raise ValueError("baseline parent mismatch")
        if mq.changed_paths(repo, commit, base=before) != [path]:
            raise ValueError("baseline commit changed other inputs")
        if mq.git(repo, "show", "-s", "--format=%s", commit).stdout.strip() != (
                f"Lower {path.split('/')[0]} old-form baselines"):
            raise ValueError("baseline commit message mismatch")
        before = commit
    if before != record["candidate"]:
        raise ValueError("candidate differs from prepared chain")
    if record.get("request") and record["request"]["branches"] != [
            {"name": b["name"], "tip": b["tip"]} for b in record["chain"]]:
        raise ValueError("request differs from merge chain")


def validate_result(preparer, record, result, log):
    """Validate controller expectations and complete returned bytes independently."""
    request = record["request"]
    worker.encoded(result)  # Refuse nonfinite values, including nested evidence.
    if type(result.get("version")) is not int or result["version"] != 1:
        raise ValueError("unknown result version")
    if worker.encoded(result["requested"]) != worker.encoded(request):
        raise ValueError("result request identity mismatch")
    verify_chain(preparer.repo, record)
    expected = preparer.expectation(request)
    if expected != record["expected"] or expected["plan"] != request["plan"]:
        raise ValueError("controller expectation changed")
    observed = {"gate": record["gate"], "runtime": request["runtime"], **expected["objects"]}
    if worker.encoded(result["observed"]) != worker.encoded(observed):
        raise ValueError("observed identities mismatch")
    if (result["outcome"] not in ("green", "red") or result["error"] is not None or
            result["completion_known"] is not True or result["reconciliation_required"] is not False):
        raise ValueError("worker infrastructure/ownership failure")
    for key in ("changed_paths", "selection", "plan"):
        if worker.encoded(result[key]) != worker.encoded(expected[key]):
            raise ValueError(f"result {key} mismatch")
    if worker.encoded(result.get("metadata")) != worker.encoded(expected["metadata"]):
        raise ValueError("candidate metadata graph mismatch")
    clean = {"head": request["candidate"], "tree": expected["objects"]["candidate_tree"],
             "clean": True, "status": ""}
    if any(worker.encoded(result[key]) != worker.encoded(clean) for key in ("before", "after")):
        raise ValueError("unclean or changed candidate snapshots")
    initial = result["initial"]
    if (initial["clean"] is not True or initial["status"] != "" or
            not worker.OID.fullmatch(initial["head"]) or not worker.OID.fullmatch(initial["tree"])):
        raise ValueError("unclean initial snapshot")
    evidence = result["log"]
    if (evidence["complete"] is not True or type(evidence["bytes"]) is not int or
            evidence["bytes"] != len(log) or evidence["sha256"] != worker.digest(log)):
        raise ValueError("incomplete/corrupt log")
    green = worker.validate_steps(preparer.repo, expected["plan"], result["steps"])
    if result["outcome"] != ("green" if green else "red"):
        raise ValueError("result outcome mismatch")
    # Observer events attest unchanged snapshots after each command, including
    # every tools suite. The tools aggregate also emits an event.
    commands = []
    for step in result["steps"]:
        if step["name"] == "tools":
            commands.extend(step["suites"])
        commands.append(step)
    if len(result["events"]) != len(commands):
        raise ValueError("incomplete observer events")
    for step, event in zip(commands, result["events"]):
        checked = event.get("checked")
        if (worker.encoded(checked) != worker.encoded(clean) or
                worker.encoded({k: v for k, v in event.items() if k != "checked"}) != worker.encoded(step)):
            raise ValueError("observer command/snapshot mismatch")
    return result["outcome"]


class Controller:
    """Two outstanding batches; effect and lifetime boundaries remain explicit.

    transport.identity() freezes gate configuration for this generation.
    transport.start(request, bundle) is idempotent for (generation, attempt).
    transport.status(request) returns {state: running|unknown|terminal, ...}; a
    terminal contains result bytes and log bytes, plus quiescent=True only after
    supervised commands AND checkout ownership are resolved. Exceptions never
    imply quiescence. No cancellation or checkout-reset operation is exposed.

    authority.guard() must refuse unless its sole-main-writer/startup guard is
    held. main() reads fresh main. promote(P,C,id) must recheck P under that same
    authority and push explicit C normally. cleanup(name,tip,id) compares and
    deletes that tip only. post(event) returns exit 0/3 on delivery. Operation supplies the explicit
    train mode. GitAuthority supplies the concrete effects under its
    held shared layout lock. Event ids are stable across every replay.
    """

    def __init__(self, preparer, store, transport, authority, *, generation, gates,
                 runtime, now, batch=8, full_every=5, completion_adapter=None):
        for token in [generation, *gates]:
            if not worker.TOKEN.fullmatch(token):
                raise ValueError("invalid generation/gate token")
        if (len(gates) != 2 or len(set(gates)) != 2 or type(batch) is not int or
                type(full_every) is not int or batch < 1 or full_every < 1):
            raise ValueError("two distinct gates and positive policy values required")
        self.preparer, self.store, self.transport, self.authority = preparer, store, transport, authority
        self.now = now
        self.completion_adapter = completion_adapter
        self.failed = False
        self.config = {"generation": generation, "gates": list(gates), "runtime": clone(runtime),
                       "batch": batch, "full_every": full_every,
                       "preparation": str(preparer.repo), "artifacts": str(preparer.artifacts),
                       "transport": clone(transport.identity()),
                       "controller_sha256": worker.digest(CONTROLLER_BYTES),
                       "authority": clone(authority.identity()) if hasattr(authority, "identity") else None,
                       "completion": (clone(completion_adapter.completion_identity()) if completion_adapter
                                      else {"kind": "local-empty-plan-v1"})}
        if set(self.config["transport"]) != set(gates):
            raise ValueError("transport must configure exactly the two gates")
        endpoints = [(config.get("host", "local"), config["repo"])
                     for config in self.config["transport"].values()]
        if len(set(endpoints)) != 2 or ("local", str(preparer.repo)) in endpoints:
            raise ValueError("preparation and both gates require separate clones")
        authority.guard()
        if hasattr(authority, "bind"):
            authority.bind(store, preparer, config=self.config)
        self.state = store.load()
        self.unreconciled_gates = set(self.state["gates"].values()) - {None} if self.state else set()
        if self.state is None:
            if (any(store.directory.iterdir()) or any(preparer.artifacts.iterdir()) or
                    mq.git(preparer.repo, "for-each-ref", "--format=%(refname)", "refs/mq/jobs/").stdout):
                raise ValueError("missing snapshot with retained recovery evidence/pins; reconciliation required")
            self.state = {"version": VERSION, "config": clone(self.config), "serial": 0,
                          "records": {}, "active": [], "gates": {gate: None for gate in gates},
                          "tried": {}, "batches_since_full": 0, "last_full": 0,
                          "last_full_main": None, "selected_passed": False,
                          "pending": [], "bisecting": False, "force_full": False,
                          "outbox": {}, "stopping": False, "idle_since": now()}
            self.save()
        elif worker.encoded(self.state["config"]) != worker.encoded(self.config):
            raise ValueError("generation/config changed; explicit reconciliation required")
        else:
            recovery.validate_artifacts(self.state, preparer, worker, verify_chain)
        self.guard()
        self.startup_inventory = self.recovery_inventory()

    def guard(self):
        if self.failed:
            raise RuntimeError("state publication failed; reload/reconcile required")
        self.authority.guard()
        worker.runtime_unchanged()
        if Path(__file__).read_bytes() != CONTROLLER_BYTES:
            raise ValueError("controller runtime changed during generation")
        if worker.encoded(self.config["runtime"]) != worker.encoded(worker.runtime_identity(self.preparer.disk_floor)):
            raise ValueError("generation runtime mismatch")
        if (self.config["authority"] is not None and
                worker.encoded(self.config["authority"]) != worker.encoded(self.authority.identity())):
            raise ValueError("authority configuration changed during generation")
        if worker.encoded(self.config["transport"]) != worker.encoded(self.transport.identity()):
            raise ValueError("gate configuration changed during generation")

    def save(self):
        try:
            self.store.save(self.state)
        except BaseException:
            self.failed = True
            raise

    def reservations(self):
        return {b["name"] for r in self.state["records"].values()
                if not r.get("applied") and (r["id"] in self.state["active"] or
                                            r["id"] in self.state["gates"].values())
                for b in r["inputs"]} | {b["name"] for group in self.state["pending"] for b in group}

    def full_due(self):
        # An unresolved full predecessor is charged as an attempt, never a reset.
        return (self.state["force_full"] or self.state["batches_since_full"] +
                len(self.state["active"]) + 1 >= self.config["full_every"] or
                self.now() - self.state["last_full"] >= 3600)

    def main_consistent(self):
        """Fresh P/C reconciliation before new preparation/dispatch, never adopt."""
        main = self.authority.main()
        if self.state["active"]:
            head = self.state["records"][self.state["active"][0]]
            expected = {head["parent"]}
            if head["phase"] == "promotion-intent":
                expected.add(head["candidate"])
            return main in expected
        ordered = sorted(self.state["records"].values(), key=lambda r: int(r["id"].rsplit("-", 1)[1]))
        greens = [r for r in ordered if r.get("decision") == "green"]
        expected = greens[-1]["candidate"] if greens else ordered[0]["parent"] if ordered else main
        return main == expected

    def admit(self):
        self.guard()
        if self.unreconciled_gates or not self.main_consistent():
            return None
        if (self.state["stopping"] or len(self.state["active"]) >= 2 or
                any(owner and (self.state["records"][owner].get("invalidated") or
                               self.state["records"][owner]["applied"])
                    for owner in self.state["gates"].values()) or
                any(self.state["records"][a]["phase"] in {"preparing", "prepared", "dispatch-intent"}
                    for a in self.state["active"]) or
                any(r.get("preparation_unresolved") for r in self.state["records"].values()) or
                any(self.state["records"][a].get("blocked") for a in self.state["active"]) or
                any(self.state["records"][a].get("conflicts") for a in self.state["active"]) or
                any(self.state["records"][a].get("outcome") in ("red", "conflict", "INFRA")
                    for a in self.state["active"])):
            return None
        available = [g for g, owner in self.state["gates"].items() if owner is None]
        if not available:
            return None
        if self.state["bisecting"]:
            # Drain invalidated speculative jobs before sequential recovery.
            if self.state["active"] or any(self.state["gates"].values()):
                return None
            if not self.state["pending"]:
                self.state["bisecting"] = self.state["force_full"] = False
                self.save()
                return None
            branches = clone(self.state["pending"][0])
        else:
            reserved = self.reservations()
            branches = [b for b in self.preparer.discover(self.state["tried"])
                        if b["name"] not in reserved][:self.config["batch"]]
        idle = not branches
        if idle:
            if (self.state["active"] or self.now() - self.state["idle_since"] < 600 or
                    self.authority.main() == self.state["last_full_main"]):
                return None
        parent = (self.state["records"][self.state["active"][-1]]["candidate"]
                  if self.state["active"] else self.authority.main())
        self.state["serial"] += 1
        # Keep the worker's 128-character attempt limit even for a maximum-size
        # generation token, while separating artifact identities by generation.
        attempt = f"g{worker.digest(self.config['generation'].encode())}-{self.state['serial']}"
        record = {"id": attempt, "generation": self.config["generation"], "gate": available[0],
                  "parent": parent, "inputs": clone(branches), "full": idle or self.full_due(),
                  "idle": idle, "bisection": self.state["bisecting"],
                  "phase": "preparing", "outcome": None, "applied": False}
        self.state["records"][attempt] = record
        self.state["active"].append(attempt)
        if self.state["bisecting"]:
            self.state["pending"].pop(0)
        self.save()  # Reservations and preparation intent precede clone mutation.
        # The marker survives controller death and failed snapshot publication.
        # Only the completed snapshot permits clearing preparation ownership.
        with self.preparer.ownership():
            marker = self.preparer.repo / ".git/mq-preparation-unresolved.json"
            try:
                worker.atomic_json(marker, {"attempt": attempt, "generation": self.config["generation"],
                                            "state": str(self.store.path)})
            except BaseException:
                self.failed = True
                raise
            try:
                record.update(self.preparer.prepare_candidate(record, self.config["runtime"]))
                record["prepare_log"] = durable.fingerprint(durable.read_bytes(
                    self.preparer.artifacts / attempt / "prepare.log"))
                record["phase"] = "prepared"
            except Exception as error:
                record["preparation_unresolved"] = True
                self.infra(attempt, error)
                return attempt
            self.save()
            marker.unlink()
            worker.sync_directory(marker.parent)
        if record["outcome"] is None:
            self.dispatch(attempt)
        return attempt

    def dispatch(self, attempt):
        self.guard()
        self.check_artifacts()
        if not self.main_consistent():
            raise RuntimeError("unexpected main; reconcile before dispatch")
        record = self.state["records"][attempt]
        if record["phase"] not in ("prepared", "dispatch-intent", "unknown") or record.get("invalidated"):
            raise ValueError("attempt cannot be dispatched")
        gate = record["gate"]
        if self.state["gates"][gate] not in (None, attempt):
            raise RuntimeError("gate is owned by another attempt")
        self.state["gates"][gate] = attempt
        record["phase"] = "dispatch-intent"
        self.save()  # Persist ownership before even a transfer/start request.
        try:
            bundle = record["bundle"]
            data = durable.checked(Path(bundle["path"]), bundle)
            if len(data) != bundle["bytes"] or worker.digest(data) != bundle["sha256"]:
                raise ValueError("bundle changed before transfer")
            self.transport.start(clone(record["request"]), clone(bundle))
            record["phase"] = "running"
        except Exception as error:
            record["phase"] = "unknown"
            record["transport_error"] = str(error)
            self.infra(attempt, error)
            return
        self.save()

    def poll(self, attempt):
        self.guard()
        record = self.state["records"][attempt]
        if self.state["gates"][record["gate"]] != attempt:
            return
        try:
            reply = self.transport.status(clone(record["request"]))
            if reply.get("request") != record["request"]:
                raise ValueError("status does not identify the owning request")
            if reply["state"] == "running":
                if not record["applied"]:
                    record["phase"] = "draining" if record.get("invalidated") else "running"
                self.unreconciled_gates.discard(attempt)
            elif reply["state"] == "reconciled":
                self.reconcile_job(attempt, quiescent=reply.get("quiescent"), evidence=reply.get("evidence"), supervision=reply.get("supervision"))
                if not record["applied"] and not record.get("invalidated"):
                    self.infra(attempt, "operator reconciled interrupted supervisor; no check certificate")
                return
            elif reply["state"] == "terminal":
                self.ingest(attempt, reply["result"], reply["log"],
                            quiescent=reply.get("quiescent"), evidence=reply.get("evidence"), supervision=reply.get("supervision"))
                return
            else:
                raise RuntimeError("worker status unknown")
        except Exception as error:
            # Invalid transport is not a check failure and cannot free a gate.
            if self.failed:
                raise
            self.infra(attempt, error)
            return
        self.save()

    def ingest(self, attempt, result_bytes, log, *, quiescent=None, evidence=None, supervision=None):
        self.guard()
        record = self.state["records"][attempt]
        if type(quiescent) is not bool or self.state["gates"][record["gate"]] != attempt:
            raise ValueError("terminal evidence requires explicit ownership status for this job")
        job = self.preparer.artifacts / attempt
        try:
            write_bytes(job / "returned.log", log)
            write_bytes(job / "returned.json", result_bytes)
            if supervision is not None:
                write_bytes(job / "supervision.json", supervision)
        except BaseException:
            self.failed = True
            raise
        try:
            record["returned"] = {"returned.log": durable.fingerprint(log),
                                  "returned.json": durable.fingerprint(result_bytes)}
            if supervision is not None:
                record["returned"]["supervision.json"] = durable.fingerprint(supervision)
                if durable.fingerprint(supervision) != evidence.get("receipt"):
                    raise ValueError("supervisor receipt identity mismatch")
            if quiescent is not True:
                raise ValueError("terminal without resolved job/checkout ownership")
            self.verify_completion(record, evidence)
            result = read_json(result_bytes)
            outcome = validate_result(self.preparer, record, result, log)
        except Exception as error:
            self.infra(attempt, error)
            return
        # Callers may only ingest terminal replies with independently established
        # quiescence (poll enforces it); malformed evidence retains ownership.
        self.state["gates"][record["gate"]] = None
        self.unreconciled_gates.discard(attempt)
        record["result"] = result
        record["completion_evidence"] = clone(evidence)
        if record.get("invalidated") or record["applied"]:
            record["phase"] = "discarded"
        else:
            record["outcome"], record["phase"] = outcome, "terminal"
            if outcome == "red":
                self.invalidate_descendants(attempt, "ancestor red")
        self.save()

    def invalidate_descendants(self, attempt, reason):
        active = self.state["active"]
        if attempt not in active:
            return
        index = active.index(attempt)
        for successor in active[index + 1:]:
            record = self.state["records"][successor]
            record["invalidated"] = reason
            record["phase"] = "draining" if successor in self.state["gates"].values() else "discarded"
        del active[index + 1:]

    def infra(self, attempt, error):
        record = self.state["records"][attempt]
        record["error"] = str(error)
        if not record["applied"]:
            record["outcome"] = "INFRA"
            record["phase"] = "unknown" if attempt in self.state["gates"].values() else "terminal"
            self.invalidate_descendants(attempt, "ancestor infrastructure failure")
        self.save()

    def verify_completion(self, record, evidence):
        if self.completion_adapter is not None:
            if worker.encoded(self.completion_adapter.completion_identity()) != worker.encoded(self.config["completion"]):
                raise ValueError("completion adapter changed during generation")
            verified = self.completion_adapter.verify_completion(
                clone(record["request"]), clone(evidence), clone(self.config["transport"][record["gate"]]))
            if not isinstance(verified, dict) or verified != evidence:
                raise ValueError("completion adapter must return verified identity-bound evidence")
            return verified
        return recovery.local_completion(record["request"], evidence,
            self.config["transport"][record["gate"]], authority_module)

    def reconcile_job(self, attempt, *, quiescent, evidence, supervision=None):
        """Only supported evidence releases ownership; no reset or PID inference."""
        self.guard()
        record = self.state["records"][attempt]
        if quiescent is not True:
            raise ValueError("explicit supported job/ownership reconciliation evidence required")
        self.verify_completion(record, evidence)
        if self.state["gates"][record["gate"]] != attempt:
            raise ValueError("attempt no longer owns gate")
        if supervision is not None:
            if durable.fingerprint(supervision) != evidence.get("receipt"):
                raise ValueError("reconciliation receipt identity mismatch")
            try:
                write_bytes(self.preparer.artifacts / attempt / "reconciled-supervision.json", supervision)
                record.setdefault("returned", {})["reconciled-supervision.json"] = durable.fingerprint(supervision)
            except BaseException:
                self.failed = True
                raise
        record["reconciliation"] = clone(evidence)
        self.unreconciled_gates.discard(attempt)
        self.state["gates"][record["gate"]] = None
        record["phase"] = "discarded" if record.get("invalidated") or record["applied"] else "terminal"
        if record["outcome"] is None:
            record["outcome"] = "INFRA"
        self.save()

    def recheck_certificate(self, record):
        try:
            log = durable.checked(self.preparer.artifacts / record["id"] / "returned.log", record["result"]["log"])
            outcome = validate_result(self.preparer, record, record["result"], log)
            if outcome != record["outcome"]:
                raise ValueError("certificate differs from authoritative outcome")
        except BaseException:
            self.failed = True
            raise

    def check_artifacts(self):
        try:
            recovery.validate_artifacts(self.state, self.preparer, worker, verify_chain)
        except BaseException:
            self.failed = True
            raise

    def event(self, record, kind, branch=None, detail=None):
        suffix = branch["name"] if branch else "ops"
        identity = f"{record['id']}:{kind}:{suffix}"
        branch_text = (f"Branch: {branch['name']} at {branch['tip']}\n"
                       f"Merge: {branch.get('commit', 'none')}\n" if branch else "")
        self.state["outbox"].setdefault(identity, {"id": identity, "kind": kind,
            "attempt": record["id"], "branch": clone(branch), "detail": detail,
            "body": f"Event: {identity}\n{kind}\nAttempt: {record['id']}\n"
                    f"Gate: {record['gate']}\n{branch_text}Detail: {detail or kind}\n"
                    f"Evidence: {self.preparer.artifacts / record['id']}\n",
            "delivered": False})

    def account(self, record, outcome):
        if outcome not in ("green", "red"):
            return
        selected = record.get("request", {}).get("selection") if record.get("request") else None
        if selected is None:
            return
        if selected == "full":
            self.state["batches_since_full"] = 0
            self.state["last_full"] = self.now()
            if outcome == "green":
                self.state["last_full_main"] = record["candidate"]
                self.state["selected_passed"] = False
            elif self.state["selected_passed"]:
                self.event(record, "selection-gap")
        else:
            self.state["batches_since_full"] += 1
            if outcome == "green":
                self.state["selected_passed"] = True

    def apply_decision_once(self, record, outcome):
        self.guard()
        if record["applied"]:
            return
        if (self.state["active"][:1] != [record["id"]] or record.get("invalidated") or
                outcome != record["outcome"]):
            raise ValueError("decision requires the matching head outcome")
        if outcome == "conflict" and record.get("chain"):
            raise ValueError("partial conflicts require a checked outcome")
        if outcome == "green" and (record["phase"] != "promotion-intent" or
                                   self.authority.main() != record["candidate"]):
            raise ValueError("green decision requires landed certified candidate")
        self.account(record, outcome)
        if outcome == "green":
            for branch in record["chain"]:
                self.state["tried"][branch["name"]] = branch["tip"]
                self.event(record, "landed", branch)
                self.event(record, "cleanup", branch)
        elif outcome in ("red", "conflict"):
            self.state["bisecting"] = True
            inherited_full = (record["request"]["selection"] == "full"
                              if record.get("request") else record["full"])
            self.state["force_full"] |= inherited_full
            merged = [{"name": b["name"], "tip": b["tip"]} for b in record.get("chain", [])]
            if outcome == "red" and len(merged) == 1:
                branch = merged[0]
                self.state["tried"][branch["name"]] = branch["tip"]
                steps = record.get("result", {}).get("steps", record.get("preparation", []))
                failed = next((s["name"] for s in steps if not s["green"]), "preparation")
                self.event(record, "rejected", branch, "failing step: " + failed)
            elif merged:
                groups = [merged[:len(merged)//2], merged[len(merged)//2:]]
                self.state["pending"] = groups + self.state["pending"]
        else:
            # No conflict rejection survives infrastructure failure.
            self.event(record, "infrastructure", detail=record.get("error"))
            if self.state["bisecting"]:
                self.state["pending"].insert(0, clone(record["inputs"]))
        if outcome in ("green", "red", "conflict"):
            # Partial conflicts stay provisional through worker checking and
            # promotion; only this same non-INFRA decision makes them tried.
            for branch in record.get("conflicts", []):
                self.state["tried"][branch["name"]] = branch["tip"]
                self.event(record, "rejected", branch, branch["error"])
        record["accounting"] = {k: clone(self.state[k]) for k in (
            "batches_since_full", "last_full", "last_full_main", "selected_passed")}
        record["applied"], record["decision"] = True, outcome
        record["phase"] = "decided"
        self.state["active"].remove(record["id"])
        self.state["idle_since"] = self.now()
        self.save()  # Decision, counters, bisection groups and outbox are atomic.

    def adjudicate_head(self):
        self.guard()
        self.check_artifacts()
        if not self.state["active"]:
            return False
        record = self.state["records"][self.state["active"][0]]
        if record["phase"] == "promotion-intent":
            return self.promote(record)
        outcome = record["outcome"]
        if outcome in ("green", "red") and record.get("result") is not None:
            self.recheck_certificate(record)
        if outcome is None:
            return False
        if outcome != "INFRA" and self.authority.main() != record["parent"]:
            record["blocked"] = "fresh main differs from certified parent"
            self.invalidate_descendants(record["id"], record["blocked"])
            self.save()
            return False
        if outcome == "green":
            record["promotion"] = {"id": record["id"] + ":promotion", "parent": record["parent"],
                                   "candidate": record["candidate"]}
            record["phase"] = "promotion-intent"
            self.save()
            return self.promote(record)
        self.invalidate_descendants(record["id"], "ancestor " + outcome)
        self.apply_decision_once(record, outcome)
        return True

    def promote(self, record):
        self.guard()
        self.check_artifacts()
        if (self.state["active"][:1] != [record["id"]] or
                record["phase"] != "promotion-intent" or record.get("invalidated")):
            raise ValueError("promotion requires the head's recorded intent")
        self.recheck_certificate(record)
        intent = record["promotion"]
        main = self.authority.main()
        if main == intent["parent"]:
            if intent["candidate"] != intent["parent"]:
                try:
                    self.authority.promote(intent["parent"], intent["candidate"], intent["id"])
                except Exception as error:
                    record["promotion_error"] = str(error)
                    self.save()
                    return False
            main = self.authority.main()
        if main == intent["candidate"]:
            self.apply_decision_once(record, "green")
            return True
        record["blocked"] = "unexpected main during promotion reconciliation"
        self.invalidate_descendants(record["id"], record["blocked"])
        self.save()
        return False

    def flush_outbox(self):
        self.guard()
        self.check_artifacts()
        for event in self.state["outbox"].values():
            if event["delivered"]:
                continue
            try:
                if event["kind"] == "cleanup":
                    branch = event["branch"]
                    delivered = self.authority.cleanup(branch["name"], branch["tip"], event["id"])
                else:
                    code = self.authority.post(clone(event))
                    delivered = type(code) is int and code in (0, 3)
                event["delivered"] = delivered is True
                event.pop("error", None)
            except Exception as error:
                event["error"] = str(error)
            self.save()

    def stop(self):
        self.guard()
        self.state["stopping"] = True
        self.save()

    def drained(self):
        if (self.failed or self.state["active"] or any(self.state["gates"].values()) or
                any(r.get("preparation_unresolved") or r["phase"] == "preparing"
                    for r in self.state["records"].values())):
            return False
        try:
            snapshot = self.store.load()
            return snapshot is not None and worker.encoded(snapshot) == worker.encoded(self.state)
        except (OSError, ValueError, TypeError):
            return False

    def recovery_inventory(self):
        """No mutation: next-part startup must reconcile every returned boundary."""
        self.guard()
        main = self.authority.main()
        result = {"main": main, "unreconciled_gates": sorted(self.unreconciled_gates),
                  **clone(recovery.inventory(self.state)),
                  "artifacts": recovery.artifact_inventory(self.state, self.preparer.artifacts)}
        result["promotions"] = [{**r["promotion"], "disposition": (
            "candidate" if main == r["candidate"] else "parent" if main == r["parent"] else "unexpected")}
            for r in self.state["records"].values() if r["phase"] == "promotion-intent"]
        return result

    def reconcile_preparation(self, attempt):
        """Clear only a completion marker backed by the durable prepared snapshot.

        Incomplete preparation has no safe reset/reconstruction path in part 4.
        """
        self.guard()
        record = self.state["records"][attempt]
        if record["phase"] == "preparing" or record.get("preparation_unresolved"):
            raise RuntimeError("incomplete preparation requires supported supervisor evidence")
        self.check_artifacts()
        repo = self.preparer.repo
        with authority_module.Lock(repo.with_name(repo.name + ".queue.lock")):
            marker = repo / ".git/mq-preparation-unresolved.json"
            expected = {"attempt": attempt, "generation": self.config["generation"],
                        "state": str(self.store.path)}
            if read_json(durable.read_bytes(marker)) != expected:
                raise ValueError("preparation marker identity differs")
            marker.unlink()
            worker.sync_directory(marker.parent)

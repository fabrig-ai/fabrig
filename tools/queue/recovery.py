"""Fail-closed version-1 recovery inventory (tools/queue/design.md, 3.10).

This is validation and explicit reconciliation, never candidate reconstruction.
There is no remote supervisor evidence adapter in part 4.
"""

import importlib.util
import hashlib
import math
from pathlib import Path
import re

DIRECTORY = Path(__file__).resolve().parent
SPEC = importlib.util.spec_from_file_location("recovery_durable", DIRECTORY / "durable.py")
durable = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(durable)
TOKEN = re.compile(r"[A-Za-z0-9][A-Za-z0-9_.-]{0,127}\Z")
OID = re.compile(r"(?:[0-9a-f]{40}|[0-9a-f]{64})\Z")
PHASES = {"preparing", "prepared", "dispatch-intent", "running", "unknown", "draining",
          "terminal", "discarded", "promotion-intent", "decided"}
OUTCOMES = {None, "green", "red", "conflict", "INFRA"}


def require(condition, message):
    if not condition:
        raise ValueError("invalid recovery state: " + message)


def token(value):
    return isinstance(value, str) and TOKEN.fullmatch(value) is not None


def oid(value):
    return isinstance(value, str) and OID.fullmatch(value) is not None


def number(value):
    return type(value) in (int, float) and math.isfinite(value) and value >= 0


def branch(value):
    require(isinstance(value, dict), "branch object")
    name = value.get("name")
    require(isinstance(name, str) and len(name.split("/")) == 3 and name.startswith("ready/") and
            all(token(p) and not p.endswith(".") and ".." not in p and not p.endswith(".lock")
                for p in name.split("/")), "ready ref")
    require(oid(value.get("tip")), "pinned tip")


def branches(values):
    require(isinstance(values, list), "branch list")
    for value in values:
        branch(value)
    require(len({v["name"] for v in values}) == len(values), "duplicate branch")


def validate_state(state, *, validate_request=None):
    require(isinstance(state, dict), "snapshot object")
    require(type(state.get("version")) is int and state["version"] == 1,
            "unknown version; no automatic migration")
    fields = {"version", "config", "serial", "records", "active", "gates", "tried",
              "batches_since_full", "last_full", "last_full_main", "selected_passed",
              "pending", "bisecting", "force_full", "outbox", "stopping", "idle_since"}
    require(set(state) == fields, "snapshot fields")
    config = state["config"]
    require(isinstance(config, dict), "config object")
    require(set(config) == {"generation", "gates", "runtime", "batch", "full_every",
                           "preparation", "artifacts", "transport", "controller_sha256", "authority", "completion"},
            "exact generation configuration")
    require(token(config["generation"]), "generation")
    gates = config["gates"]
    require(isinstance(gates, list) and len(gates) == 2 and all(token(g) for g in gates) and
            len(set(gates)) == 2, "two gates")
    for field in ("batch", "full_every"):
        require(type(config[field]) is int and config[field] > 0, field)
    for field in ("preparation", "artifacts"):
        require(isinstance(config[field], str) and Path(config[field]).is_absolute() and
                str(Path(config[field])) == config[field] and ".." not in Path(config[field]).parts,
                "canonical " + field)
    require(isinstance(config["runtime"], dict) and isinstance(config["transport"], dict) and
            set(config["transport"]) == set(gates), "runtime/transport identity")
    for endpoint in config["transport"].values():
        require(isinstance(endpoint, dict) and isinstance(endpoint.get("repo"), str) and
                Path(endpoint["repo"]).is_absolute(), "gate endpoint")
    require(isinstance(config["controller_sha256"], str) and
            re.fullmatch(r"[0-9a-f]{64}", config["controller_sha256"]), "controller digest")
    require(isinstance(config["completion"], dict), "completion adapter identity")
    require(config["authority"] is None or isinstance(config["authority"], dict), "authority identity")
    for field in ("serial", "batches_since_full"):
        require(type(state[field]) is int and state[field] >= 0, field)
    for field in ("selected_passed", "bisecting", "force_full", "stopping"):
        require(type(state[field]) is bool, field)
    for field in ("last_full", "idle_since"):
        require(number(state[field]), field)
    require(state["last_full_main"] is None or oid(state["last_full_main"]), "full main")
    records, active, owners = state["records"], state["active"], state["gates"]
    require(isinstance(records, dict) and isinstance(active, list) and
            all(isinstance(a, str) for a in active) and len(active) <= 2 and
            len(set(active)) == len(active) and all(a in records for a in active), "active records")
    require(isinstance(owners, dict) and set(owners) == set(gates) and
            all(a is None or isinstance(a, str) and a in records for a in owners.values()), "gate owners")
    require(len([a for a in owners.values() if a is not None]) ==
            len(set(a for a in owners.values() if a is not None)), "duplicate gate ownership")
    require(isinstance(state["pending"], list) and
            (state["bisecting"] or not state["pending"]) and
            (state["bisecting"] or not state["force_full"]), "bisection flags")
    for group in state["pending"]:
        branches(group)
        require(bool(group), "empty bisection group")
    require(isinstance(state["tried"], dict), "tried map")
    for name, tip in state["tried"].items():
        branch({"name": name, "tip": tip})
    serials = []
    tried = {}
    frontier, recovery_full = [], False
    for attempt, record in sorted(records.items(), key=lambda item: (
            int(item[0].rsplit("-", 1)[-1]) if item[0].rsplit("-", 1)[-1].isdigit() else -1)):
        require(token(attempt) and isinstance(record, dict) and record.get("id") == attempt,
                "attempt identity")
        require(record.get("generation") == config["generation"] and record.get("gate") in gates,
                "record generation/gate")
        prefix = "g" + hashlib.sha256(config["generation"].encode()).hexdigest() + "-"
        require(attempt.startswith(prefix) and attempt[len(prefix):].isdigit(), "attempt serial")
        serials.append(int(attempt[len(prefix):]))
        require(oid(record.get("parent")), "record parent")
        branches(record.get("inputs"))
        for field in ("full", "idle", "applied", "bisection"):
            require(type(record.get(field)) is bool, "record " + field)
        if record["bisection"]:
            require(bool(frontier) and record["inputs"] == frontier[0], "bisection admission order")
            frontier.pop(0)
            require(not recovery_full or record["full"], "inherited full requirement")
        elif not record.get("invalidated") and not frontier:
            recovery_full = False
        require(isinstance(record.get("phase"), str) and record["phase"] in PHASES, "phase")
        require(record.get("outcome") is None or isinstance(record["outcome"], str), "outcome type")
        require(record.get("outcome") in OUTCOMES, "outcome")
        for field in ("invalidated", "blocked", "error", "transport_error", "promotion_error"):
            if field in record:
                require(isinstance(record[field], str), field)
        if attempt in active:
            require(not record["applied"] and not record.get("invalidated"), "active decision")
        if owners[record["gate"]] == attempt:
            require(isinstance(record.get("request"), dict), "owning request")
            require(not record["applied"] or record["outcome"] == "INFRA", "decided physical ownership")
            require(record["phase"] in {"dispatch-intent", "running", "unknown", "draining", "decided"},
                    "owning phase")
        elif record["phase"] in {"dispatch-intent", "running", "unknown", "draining"}:
            require(False, "lost durable gate ownership")
        if record["phase"] == "preparing":
            require(attempt in active and not record["applied"], "preparation reservation")
        if record["phase"] != "preparing" and not record.get("preparation_unresolved"):
            require("candidate" in record, "completed preparation identity")
        if "preparation_unresolved" in record:
            require(type(record["preparation_unresolved"]) is bool, "preparation ownership flag")
        if "candidate" in record:
            require(oid(record["candidate"]), "record candidate")
            for field in ("chain", "conflicts", "pins", "baseline_commits", "preparation"):
                require(isinstance(record.get(field), list), field)
            branches(record["chain"])
            branches(record["conflicts"])
            for item in record["chain"]:
                require(oid(item.get("before")) and oid(item.get("commit")) and
                        type(item.get("contained")) is bool and isinstance(item.get("message"), str),
                        "merge identity")
            for item in record["conflicts"]:
                require(isinstance(item.get("error"), str), "conflict evidence")
            expected_pins = [f"refs/mq/jobs/{attempt}/parent", *(
                f"refs/mq/jobs/{attempt}/input-{i}" for i in range(len(record["inputs"]))),
                f"refs/mq/jobs/{attempt}/candidate"]
            require(record["pins"] == expected_pins, "exact pinned refs")
            for step in record["preparation"]:
                require(isinstance(step, dict) and isinstance(step.get("name"), str) and
                        step["name"].startswith("ratchet:"), "preparation step")
                tree = step["name"].split(":", 1)[1]
                require(tree in {"skein", "smith", "jig", "temper"} and
                        step.get("command") == ["python3", "-I", "-B", str(Path(config["preparation"]) /
                            tree / "scripts/old_forms.py"), "--write"] and
                        step.get("cwd") == str(Path(config["preparation"]) / tree) and
                        type(step.get("returncode")) is int and step["returncode"] in {0, 1} and
                        type(step.get("green")) is bool and step["green"] == (step["returncode"] == 0) and
                        number(step.get("duration")), "preparation command evidence")
            for item in record["baseline_commits"]:
                require(isinstance(item, dict) and oid(item.get("commit")) and
                        item.get("path") in {f"{t}/scripts/old-forms.txt" for t in
                                             ("skein", "smith", "jig", "temper")}, "baseline identity")
        request = record.get("request")
        if request is not None:
            require(isinstance(request, dict), "request object")
            if validate_request:
                validate_request(request)
            require(all(request.get(k) == record[k] for k in ("generation", "gate", "parent", "candidate"))
                    and request.get("attempt") == attempt and request.get("runtime") == config["runtime"],
                    "request identity")
            require(isinstance(record.get("expected"), dict), "expectation")
        if record["outcome"] == "conflict":
            require(not record.get("chain") and bool(record.get("conflicts")), "unchecked conflict decision")
        if record["outcome"] == "red" and request is None:
            steps = record.get("preparation", [])
            require(bool(steps) and steps[-1]["returncode"] == 1 and
                    all(s["returncode"] == 0 for s in steps[:-1]) and record["full"] is True,
                    "ratchet red evidence")
        if record["outcome"] == "green" or (record["outcome"] == "red" and request is not None) or record.get("result") is not None:
            require(request is not None and isinstance(record.get("result"), dict), "checked evidence")
            result = record["result"]
            require(result.get("requested") == request, "result requested identity")
            if record["outcome"] in {"green", "red"}:
                require(result.get("outcome") == record["outcome"], "checked outcome consistency")
        if record["phase"] == "promotion-intent" or "promotion" in record:
            require(record.get("promotion") == {"id": attempt + ":promotion", "parent": record["parent"],
                                                 "candidate": record.get("candidate")} and
                    record["outcome"] == "green", "promotion intent")
        if record["applied"]:
            require(record.get("decision") == record["outcome"] and
                    record["phase"] in {"decided", "discarded"}, "applied decision")
            require(isinstance(record.get("accounting"), dict), "decision accounting checkpoint")
            if record["outcome"] in {"red", "conflict"}:
                recovery_full |= (record["request"]["selection"] == "full" if record.get("request") else record["full"])
                merged = [{"name": b["name"], "tip": b["tip"]} for b in record.get("chain", [])]
                if len(merged) > 1:
                    frontier = [merged[:len(merged)//2], merged[len(merged)//2:]] + frontier
            elif record["outcome"] == "INFRA" and record["bisection"]:
                frontier.insert(0, record["inputs"])
            accepted = record.get("chain", []) if record["outcome"] == "green" else []
            if record["outcome"] == "red" and len(record.get("chain", [])) == 1:
                accepted = record["chain"]
            if record["outcome"] in ("green", "red", "conflict"):
                accepted = [*accepted, *record.get("conflicts", [])]
            for b in accepted:
                tried[b["name"]] = b["tip"]
        else:
            require("decision" not in record, "unapplied decision")
            require(attempt in active or bool(record.get("invalidated")), "orphan undecided record")
    require(sorted(serials) == list(range(1, state["serial"] + 1)), "serial inventory")
    require(tried == state["tried"], "decision/tried consistency")
    require(frontier == state["pending"], "bisection frontier consistency")
    require(not state["bisecting"] or state["force_full"] == recovery_full, "full bisection consistency")
    if len(active) == 2:
        require(int(active[0].rsplit("-", 1)[1]) < int(active[1].rsplit("-", 1)[1]), "serial order")
        require(records[active[1]]["parent"] == records[active[0]].get("candidate"), "active ancestry")
    applied = [r for r in records.values() if r["applied"]]
    if applied:
        last = max(applied, key=lambda r: int(r["id"].rsplit("-", 1)[1]))
        require(last["accounting"] == {k: state[k] for k in (
            "batches_since_full", "last_full", "last_full_main", "selected_passed")},
            "cadence checkpoint")
    outbox = state["outbox"]
    require(isinstance(outbox, dict), "outbox")
    for identity, event in outbox.items():
        require(isinstance(event, dict) and event.get("id") == identity and
                isinstance(event.get("attempt"), str) and event["attempt"] in records and
                records[event["attempt"]]["applied"], "event decision identity")
        require(event.get("kind") in {"landed", "cleanup", "rejected", "infrastructure", "selection-gap"}
                and type(event.get("delivered")) is bool, "event kind/delivery")
        if event.get("branch") is not None:
            branch(event["branch"])
        suffix = event["branch"]["name"] if event.get("branch") else "ops"
        require(identity == f"{event['attempt']}:{event['kind']}:{suffix}" and
                isinstance(event.get("body"), str) and
                event["body"].startswith("Event: " + identity + "\n") and
                "\nEvidence: " + str(Path(config["artifacts"]) / event["attempt"]) + "\n" in event["body"],
                "event body/evidence")
        r = records[event["attempt"]]
        if event["kind"] in {"landed", "cleanup"}:
            require(r["decision"] == "green" and event["branch"] in r["chain"], "landing effect")
        if event["kind"] == "infrastructure":
            require(r["decision"] == "INFRA", "infrastructure decision")
        if event["kind"] == "selection-gap":
            require(r["decision"] == "red" and r.get("request", {}).get("selection") == "full" and
                    r["accounting"]["selected_passed"] is True, "selection gap decision")
        if event["kind"] == "rejected":
            require(r["decision"] in {"green", "red", "conflict"} and
                    (event["branch"] in r.get("conflicts", []) or
                     r["decision"] == "red" and len(r["chain"]) == 1 and
                     all(event["branch"][k] == r["chain"][0][k] for k in ("name", "tip"))),
                    "rejection effect")
    for r in applied:
        required = []
        if r["decision"] == "green":
            required += [(kind, b) for b in r["chain"] for kind in ("landed", "cleanup")]
        if r["decision"] in {"green", "red", "conflict"}:
            required += [("rejected", b) for b in r.get("conflicts", [])]
            if r["decision"] == "red" and len(r["chain"]) == 1:
                required.append(("rejected", r["chain"][0]))
        else:
            require(r["id"] + ":infrastructure:ops" in outbox, "missing infrastructure effect")
        for kind, b in required:
            require(f"{r['id']}:{kind}:{b['name']}" in outbox, "missing decision effect")
    return state


def validate_artifacts(state, preparer, worker, verify_chain):
    """Read-only: no fetch, switch, reset, reconstruction or effect replay."""
    for attempt, record in state["records"].items():
        job = preparer.artifacts / attempt
        if "candidate" not in record:
            continue  # Incomplete preparation remains reserved for reconciliation.
        durable.directory(job)
        require(durable.fingerprint(durable.read_bytes(job / "prepare.log")) == record["prepare_log"],
                "preparation log")
        pins = {record["pins"][0]: record["parent"],
                **{record["pins"][i + 1]: b["tip"] for i, b in enumerate(record["inputs"])},
                record["pins"][-1]: record["candidate"]}
        for ref, expected in pins.items():
            require(worker.mq.git(preparer.repo, "rev-parse", "--verify", ref).stdout.strip() == expected,
                    "pinned object identity")
        verify_chain(preparer.repo, record)
        if record.get("request") is not None:
            require(durable.read_json(durable.read_bytes(job / "request.json")) == record["request"],
                    "immutable request")
            bundle = record.get("bundle")
            require(isinstance(bundle, dict) and bundle.get("path") == str(job / "candidate.bundle") and
                    bundle.get("refs") == pins, "bundle identity")
            durable.checked(job / "candidate.bundle", bundle)
        if record.get("result") is not None:
            result = durable.read_json(durable.read_bytes(job / "returned.json"))
            require(result == record["result"] and result.get("requested") == record["request"],
                    "returned evidence")
            log = durable.checked(job / "returned.log", result["log"])
            require(result.get("error") is None, "certificate infrastructure error")
            require(result.get("completion_known") is True and result.get("reconciliation_required") is False,
                    "terminal completion")
            require(result.get("log", {}).get("complete") is True, "log completeness")
            expected, request = record["expected"], record["request"]
            require(result.get("observed") == {"gate": record["gate"], "runtime": request["runtime"],
                                               **expected["objects"]}, "observed certificate identity")
            require(result.get("plan") == request["plan"] == expected["plan"] and
                    all(result.get(k) == expected[k] for k in ("selection", "changed_paths", "metadata")),
                    "certificate plan/selection")
            clean = {"head": request["candidate"], "tree": expected["objects"]["candidate_tree"],
                     "clean": True, "status": ""}
            require(result.get("before") == clean and result.get("after") == clean,
                    "certificate snapshots")
            green = worker.validate_steps(preparer.repo, request["plan"], result["steps"])
            require(result["outcome"] == ("green" if green else "red"), "certificate check outcome")
        if "returned" in record:
            for name, identity in record["returned"].items():
                require(name in {"returned.json", "returned.log"}, "returned artifact name")
                durable.checked(job / name, identity)


def inventory(state):
    return {"preparing": [r["id"] for r in state["records"].values() if
                          r["phase"] == "preparing" or r.get("preparation_unresolved")],
            "dispatch": [r["id"] for r in state["records"].values() if r["phase"] in
                         {"dispatch-intent", "unknown", "running", "draining"} or
                         r["id"] in state["gates"].values()],
            "gates": dict(state["gates"]),
            "intents": [r["promotion"] for r in state["records"].values() if r["phase"] == "promotion-intent"],
            "evidence": [r["id"] for r in state["records"].values() if r.get("returned")],
            "outbox": [e["id"] for e in state["outbox"].values() if not e["delivered"]]}


def local_completion(request, evidence, endpoint, authority_module):
    """Supported evidence ONLY for local jobs with an empty command plan.

    No assertion string/boolean or remote PID can release a gate. Part 5 must
    supply supervised heavy/SSH evidence; until then such gates stay owned.
    """
    require(isinstance(evidence, dict) and set(evidence) == {"kind", "result", "sha256"} and
            evidence["kind"] == "local-empty-plan-v1" and endpoint.get("host", "local") == "local",
            "supported local completion evidence required")
    job = Path(endpoint["artifacts"]) / request["attempt"]
    require(evidence["result"] == str(job / "result.json"), "evidence path identity")
    durable.directory(job)
    with authority_module.checkout_lock(Path(endpoint["repo"])):
        result_bytes = durable.read_bytes(job / "result.json")
        require(durable.fingerprint(result_bytes)["sha256"] == evidence["sha256"], "evidence digest")
        result = durable.read_json(result_bytes)
        require(result.get("requested") == request and result.get("completion_known") is True and
                result.get("reconciliation_required") is False, "resolved job identity")
        durable.checked(job / "checks.log", result["log"])
        require(request["plan"] == [], "nonempty plans require part-5 supervisor evidence")
    return evidence


def artifact_inventory(state, root):
    """List retained and interrupted publications without following symlinks."""
    result = {}
    for entry in sorted(Path(root).iterdir()):
        if entry.is_symlink() or not entry.is_dir():
            result[entry.name] = {"unrecognized": True}
            continue
        files = sorted(p.name for p in entry.iterdir())
        record = state["records"].get(entry.name, {})
        result[entry.name] = {"files": files, "orphan": not bool(record),
            "incomplete": [name for name in files if name.startswith(".")],
            "unrecorded_returned": [name for name in files if name.startswith("returned.") and
                                    name not in record.get("returned", {})]}
    return result

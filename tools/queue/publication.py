"""Saved-candidate publication and reconciliation (tools/queue/design.md, 3.11)."""

from datetime import datetime
import hashlib
import importlib.util
from pathlib import Path
import re
import uuid
from urllib.parse import urlsplit

SPEC = importlib.util.spec_from_file_location("publication_authority", Path(__file__).with_name("authority.py"))
authority = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(authority)
durable = authority.durable
KEY = re.compile(r"[A-Za-z0-9_.-]+\Z")
IDENTITY = re.compile(r"[0-9a-f]{32}\Z")
DIGEST = re.compile(r"[0-9a-f]{64}\Z")
INTENT_FIELDS = {"version", "publication_id", "origin", "ref", "parent", "candidate", "merges",
                 "coordinators_sha256", "created", "journal_path", "journal_row"}
LANDING_FIELDS = {"publication_id", "publication_time", "merges"}


def coordinators(data):
    """Parse exact configuration bytes; an absent key is a supported route."""
    text = data.decode("utf-8")
    if any(ord(c) < 32 and c not in "\n\t" or ord(c) == 127 for c in text):
        raise ValueError("controls in coordinators.txt")
    result = {}
    for line in text.splitlines():
        cells = line.split("#", 1)[0].split()
        if not cells:
            continue
        if len(cells) != 2:
            raise ValueError("coordinators.txt requires KEY COORDINATOR_ADDRESS")
        key, address = cells
        if not KEY.fullmatch(key) or key in result:
            raise ValueError("invalid or duplicate coordinator key")
        if address != "-" and not authority.ADDRESS.fullmatch(address):
            raise ValueError("invalid hosted coordinator")
        result[key] = address
    return result


def route(mapping, name):
    address = mapping.get(name.split("/")[1])
    if address is None:
        return {"route_status": "unresolved", "route_reason": "missing_key"}
    if address == "-":
        return {"route_status": "unresolved", "route_reason": "unassigned"}
    return {"route_status": "resolved", "coordinator": address}


def validate_route(value):
    if not isinstance(value, dict):
        raise ValueError("route must be an object")
    if value.get("route_status") == "resolved":
        if (set(value) != {"route_status", "coordinator"} or
                not isinstance(value["coordinator"], str) or
                not authority.ADDRESS.fullmatch(value["coordinator"])):
            raise ValueError("invalid resolved route")
    elif value.get("route_status") == "unresolved":
        if (set(value) != {"route_status", "route_reason"} or
                value["route_reason"] not in ("unassigned", "missing_key", "historical_unknown")):
            raise ValueError("invalid unresolved route")
    else:
        raise ValueError("invalid route status")


def explicit_origin(value):
    # Git accepts URL userinfo and helpers; saved authority accepts neither passwords nor helpers.
    if (not isinstance(value, str) or not value or value.startswith("-") or
            any(ord(c) < 33 or ord(c) == 127 for c in value)):
        raise ValueError("invalid explicit credential-free origin")
    if Path(value).is_absolute():
        return value
    if re.fullmatch(r"[A-Za-z0-9_.-]+@[A-Za-z0-9_.-]+:[A-Za-z0-9_./-]+", value):
        return value
    parsed = urlsplit(value)
    if (parsed.scheme not in ("https", "ssh") or not parsed.hostname or parsed.password is not None or
            parsed.scheme == "https" and parsed.username is not None or parsed.query or parsed.fragment or
            not re.fullmatch(r"(?:ssh|https)://[A-Za-z0-9_.@:/-]+", value)):
        raise ValueError("invalid explicit credential-free origin")
    return value


def utc(value):
    if not isinstance(value, str) or not re.fullmatch(r"\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d(?:\.\d+)?(?:Z|\+00:00)", value):
        raise ValueError("expected UTC time")
    datetime.fromisoformat(value.replace("Z", "+00:00"))


def validate(intent, git, repo):
    if not isinstance(intent, dict) or set(intent) != INTENT_FIELDS:
        raise ValueError("invalid publication intent fields")
    if type(intent["version"]) is not int or intent["version"] != 1:
        raise ValueError("invalid publication version")
    if not isinstance(intent["publication_id"], str) or not IDENTITY.fullmatch(intent["publication_id"]):
        raise ValueError("invalid publication identity")
    explicit_origin(intent["origin"])
    if intent["ref"] != "refs/heads/main":
        raise ValueError("invalid publication ref")
    for key in ("parent", "candidate"):
        authority.oid(intent[key])
    if not isinstance(intent["coordinators_sha256"], str) or not DIGEST.fullmatch(intent["coordinators_sha256"]):
        raise ValueError("invalid coordinators digest")
    utc(intent["created"])
    if not isinstance(intent["journal_path"], str) or not Path(intent["journal_path"]).is_absolute():
        raise ValueError("journal path must be absolute")
    merges = intent["merges"]
    if not isinstance(merges, list) or not merges:
        raise ValueError("publication requires carrying merges")
    names = set()
    for entry in merges:
        if not isinstance(entry, dict):
            raise ValueError("merge must be an object")
        fixed = {"name", "tip", "commit", "parent"}
        if not fixed <= entry.keys():
            raise ValueError("incomplete merge")
        validate_route({key: value for key, value in entry.items() if key not in fixed})
        name = entry["name"]
        if (not isinstance(name, str) or len(name.split("/")) != 3 or not name.startswith("ready/") or
                not KEY.fullmatch(name.split("/")[1]) or name in names or
                git(repo, "check-ref-format", "refs/heads/" + name, check=False).returncode):
            raise ValueError("invalid merge branch")
        names.add(name)
        for key in ("tip", "commit", "parent"):
            authority.oid(entry[key])
    row = intent["journal_row"]
    if (not isinstance(row, dict) or row.get("outcome") != "green" or row.get("dry") is not False or
            "event" in row or row.get("publication_id") != intent["publication_id"] or
            row.get("parent") != intent["parent"] or row.get("commit") != intent["candidate"] or
            row.get("merges") != merges or row.get("publication_time") != intent["created"] or
            row.get("end") != intent["created"] or
            not {"start", "branches", "selection", "steps", "posts", "conflicts"} <= row.keys()):
        raise ValueError("prospective journal row differs from intent")
    utc(row["start"])
    for key in ("branches", "steps", "posts", "conflicts"):
        if not isinstance(row[key], list):
            raise ValueError("invalid journal array")
    for branch in row["branches"]:
        if (not isinstance(branch, dict) or not isinstance(branch.get("name"), str) or
                type(branch.get("timestamp")) is not int):
            raise ValueError("invalid journal branch")
        authority.oid(branch.get("tip"))
    if any(not any(b["name"] == m["name"] and b["tip"] == m["tip"] for b in row["branches"]) for m in merges):
        raise ValueError("carrying merge absent from batch")
    durable.encoded(intent)


def validate_record(row):
    if not isinstance(row, dict):
        raise ValueError("journal row must be an object")
    if not LANDING_FIELDS & row.keys():
        return  # Ordinary legacy attempts and event diagnostics retain their shapes.
    required = {"start", "end", "publication_time", "publication_id", "parent", "commit", "merges",
                "branches", "selection", "steps", "posts", "conflicts", "dry", "outcome"}
    if (not required <= row.keys() or "event" in row or row["outcome"] != "green" or row["dry"] is not False or
            not isinstance(row["publication_id"], str) or not IDENTITY.fullmatch(row["publication_id"])):
        raise ValueError("incomplete or nonpublication landing row")
    for key in ("start", "end", "publication_time"):
        utc(row[key])
    if row["end"] != row["publication_time"]:
        raise ValueError("publication observation differs")
    for key in ("parent", "commit"):
        authority.oid(row[key])
    for key in ("merges", "branches", "steps", "posts", "conflicts"):
        if not isinstance(row[key], list):
            raise ValueError("invalid publication array")
    if not row["merges"]:
        raise ValueError("publication requires carrying merges")
    for entry in row["merges"]:
        fixed = {"name", "tip", "commit", "parent"}
        if (not isinstance(entry, dict) or not fixed <= entry.keys() or
                not isinstance(entry["name"], str) or len(entry["name"].split("/")) != 3 or
                not entry["name"].startswith("ready/")):
            raise ValueError("invalid publication merge")
        for key in ("tip", "commit", "parent"):
            authority.oid(entry[key])
        validate_route({key: value for key, value in entry.items() if key not in fixed})


def graph(intent, git, repo):
    """Prove exact P/B parents and complete first-parent carrying-merge order."""
    parent, candidate = intent["parent"], intent["candidate"]
    rows = git(repo, "rev-list", "--first-parent", "--parents", parent + ".." + candidate).stdout.splitlines()
    carrying, previous = [], parent
    for row in reversed(rows):
        commit, *parents = row.split()
        if not parents or parents[0] != previous:
            raise ValueError("publication parent absent from first-parent chain")
        if len(parents) > 1:
            carrying.append((commit, parents))
        previous = commit
    if previous != candidate or len(carrying) != len(intent["merges"]):
        raise ValueError("incomplete carrying merge array")
    for entry, (commit, parents) in zip(intent["merges"], carrying):
        if commit != entry["commit"] or parents != [entry["parent"], entry["tip"]]:
            raise ValueError("carrying merge parents/order differ")



class Store:
    def __init__(self, root, repo, git, guard):
        self.root = Path(root)
        self.repo, self.git, self.guard = repo, git, guard

    def prepare(self, origin, row, merges, data, created):
        self.guard()
        mapping = coordinators(data)
        row = durable.read_json(durable.encoded(row))
        identity = uuid.uuid4().hex
        entries = [{**{k: merge[k] for k in ("name", "tip", "commit", "parent")},
                    **route(mapping, merge["name"])} for merge in merges]
        row = {**row, "outcome": "green", "end": created, "publication_time": created,
               "publication_id": identity, "merges": entries}
        intent = {"version": 1, "publication_id": identity, "origin": explicit_origin(origin),
                  "ref": "refs/heads/main", "parent": row["parent"], "candidate": row["commit"],
                  "merges": entries, "coordinators_sha256": hashlib.sha256(data).hexdigest(),
                  "created": created, "journal_path": str(Path(row.pop("journal_path")).absolute()),
                  "journal_row": row}
        validate(intent, self.git, self.repo)
        graph(intent, self.git, self.repo)
        self.guard()
        root = durable.directory(self.root, create=True)
        directory = durable.directory(root / identity, create=True)
        durable.publish(directory / "coordinators.txt", data, immutable=True)
        durable.publish(directory / "intent.json", durable.encoded(intent), immutable=True)
        return intent

    def load(self, directory):
        durable.directory(directory)
        intent = durable.read_json(durable.read_bytes(directory / "intent.json"))
        validate(intent, self.git, self.repo)
        if directory.name != intent["publication_id"]:
            raise ValueError("publication directory identity differs")
        data = durable.read_bytes(directory / "coordinators.txt")
        if hashlib.sha256(data).hexdigest() != intent["coordinators_sha256"]:
            raise ValueError("saved coordinator bytes differ")
        mapping = coordinators(data)
        for entry in intent["merges"]:
            if {k: entry[k] for k in entry if k not in {"name", "tip", "commit", "parent"}} != route(mapping, entry["name"]):
                raise ValueError("saved route differs from coordinator snapshot")
        return intent

    def entries(self):
        self.guard()
        if not self.root.exists() and not self.root.is_symlink():
            return []
        durable.directory(self.root)
        result = []
        for directory in sorted(self.root.iterdir()):
            if not IDENTITY.fullmatch(directory.name):
                raise ValueError("unexpected publication evidence")
            durable.directory(directory)
            if (directory / "intent.json").exists() or (directory / "intent.json").is_symlink():
                marker = directory / "effects-confirmation.json"
                if marker.exists() or marker.is_symlink():
                    effects = durable.read_json(durable.read_bytes(marker))
                    if (not isinstance(effects, dict) or type(effects.get("version")) is not int or
                            effects != {"version": 1, "publication_id": directory.name}):
                        raise ValueError("invalid effects confirmation")
                    saved = durable.read_json(durable.read_bytes(directory / "intent.json"))
                    if (not isinstance(saved, dict) or set(saved) != INTENT_FIELDS or
                            type(saved["version"]) is not int or saved["version"] != 1 or
                            saved["publication_id"] != directory.name):
                        raise ValueError("invalid retired intent")
                    confirmation = durable.read_json(durable.read_bytes(directory / "journal-confirmation.json"))
                    if (not isinstance(confirmation, dict) or type(confirmation.get("version")) is not int or
                            durable.encoded(confirmation) != durable.encoded(self.confirmation(saved))):
                        raise ValueError("retired publication has invalid confirmation")
                    continue
                result.append((directory, self.load(directory)))
            elif (directory / "journal-confirmation.json").exists():
                raise ValueError("confirmation has no intent")
            # A crash before intent commit leaves nonauthoritative staged bytes.
        return result

    def journal(self, intent, *, append):
        self.guard()
        path = Path(intent["journal_path"])
        # The queue creates this external directory before preparing an intent.
        durable.directory(path.parent, private=False)
        data = durable.read_bytes(path) if path.exists() or path.is_symlink() else b""
        if data and not data.endswith(b"\n"):
            raise ValueError("incomplete journal tail")
        matched = False
        for line in data.splitlines():
            row = durable.read_json(line)
            validate_record(row)
            if row.get("publication_id") == intent["publication_id"]:
                if matched or durable.encoded(row) != durable.encoded(intent["journal_row"]):
                    raise ValueError("conflicting or duplicate journal publication")
                matched = True
        if not matched:
            if not append:
                raise ValueError("confirmation has no exact journal row")
            durable.publish(path, data + durable.encoded(intent["journal_row"]), private_parent=False)
        else:
            # A previous rename may have returned before directory fsync.
            with path.open("rb") as source:
                durable.os.fsync(source.fileno())
            durable.sync_directory(path.parent)

    def confirmation(self, intent):
        return {"version": 1, "publication_id": intent["publication_id"],
                "journal_path": intent["journal_path"],
                "row_sha256": hashlib.sha256(durable.encoded(intent["journal_row"])).hexdigest()}

    def confirmed(self, directory, intent):
        path = directory / "journal-confirmation.json"
        if not path.exists() and not path.is_symlink():
            return False
        value = durable.read_json(durable.read_bytes(path))
        if (not isinstance(value, dict) or type(value.get("version")) is not int or
                durable.encoded(value) != durable.encoded(self.confirmation(intent))):
            raise ValueError("invalid journal confirmation")
        self.journal(intent, append=False)
        durable.publish(path, durable.encoded(self.confirmation(intent)), immutable=True)
        return True

    def reconcile_one(self, intent):
        self.guard()
        directory = self.root / intent["publication_id"]
        saved = self.load(directory)
        if durable.encoded(saved) != durable.encoded(intent):
            raise ValueError("publication differs from durable intent")
        if self.confirmed(directory, intent):
            return
        # Fetch the saved target, independent of current origin config or HEAD.
        def fetch():
            self.guard()
            self.git(self.repo, "fetch", "--no-tags", "--", intent["origin"], intent["ref"])
            return authority.oid(self.git(self.repo, "rev-parse", "FETCH_HEAD^{commit}").stdout.strip())
        # Finish durability even when a prior process died after rename and before fsync.
        durable.publish(directory / "coordinators.txt", durable.read_bytes(directory / "coordinators.txt"), immutable=True)
        durable.publish(directory / "intent.json", durable.encoded(intent), immutable=True)
        durable.sync_directory(self.root)
        durable.sync_directory(self.root.parent)
        remote = fetch()
        graph(intent, self.git, self.repo)
        contained = self.git(self.repo, "merge-base", "--is-ancestor", intent["candidate"], remote, check=False)
        if contained.returncode not in (0, 1):
            raise RuntimeError("cannot establish candidate containment")
        if contained.returncode:
            if remote != intent["parent"]:
                raise RuntimeError("publication remote diverged; retain intent and block replacement")
            self.guard()
            # Even a lost acknowledgement is resolved by a fresh containment observation.
            self.git(self.repo, "push", "--", intent["origin"],
                     intent["candidate"] + ":" + intent["ref"], check=False)
            remote = fetch()
            if self.git(self.repo, "merge-base", "--is-ancestor", intent["candidate"], remote, check=False).returncode:
                raise RuntimeError("publication unconfirmed; retain intent and block replacement")
        self.journal(intent, append=True)
        durable.publish(directory / "journal-confirmation.json", durable.encoded(self.confirmation(intent)), immutable=True)
        self.git(self.repo, "update-ref", "refs/remotes/origin/main", remote)

    def reconcile(self):
        entries = self.entries()
        # Validate every intent before effects; replacement routes are not involved.
        for directory, intent in entries:
            if not self.confirmed(directory, intent):
                self.reconcile_one(intent)
        return [intent for _, intent in entries]

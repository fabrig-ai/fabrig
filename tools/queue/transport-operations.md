# Two-gate validation and rollout runbook

Production is OFF. These are separate Ops actions after review, never commands
performed by the implementation task. hetzner8 alone owns main authority;
hetzner2 is the dedicated second gate. Neither is a builder. Keep the existing
queue stopped during validation; do not borrow a builder or change budgets.
Use login shells remotely, umask 077 and stdin /dev/null for tools. Sign-ins,
credentials, dedicated-host reboot and production activation are separate
operator/user decisions. Never copy credentials, stop heavy units, delete heavy
state, or remove queue ownership markers by hand.

## 1. Prepare frozen private paths

On h8 and h2, arrange an identical reviewed queue runtime outside all candidate
checkouts, for example `/home/free/src/fabrig/mq-runtime/<reviewed-tip>/`.
Copy exactly worker.RUNTIME_FILES, preserving bytes. Make runtime/stores 0700
and configs/logs 0600, no symlink components. Retain this runtime while the
generation exists. Do not run the worker from its candidate checkout.

On h8 keep four disjoint roots: shared authority, authority clone, preparation
clone, preparation artifacts. Gates use independent clones, artifact roots and
endpoint job roots. The existing warm queue checkout may become h8's worker
only after legacy has drained. Do not share targets, .git, alternates or object
stores. h2's paths may have the same text but are on its dedicated machine.
All compliant legacy/train/restart writers must use the SAME h8 authority root.
Keep the currently reviewed batch 16, full_every 5, queue floor 45 decimal GB,
60-second cadence, three-second startup acknowledgement and 1800-second drain.

Verify heavy's supported recovery health separately with its existing
`--check-recovery`; install/repoint only through the separately approved heavy
rollout. The new `--completion-query` needs fresh harmless real validation; old
identical-migration evidence is insufficient. Record machine/date/runtime Git
OID and policy/receipt identities. No real systemd recovery suite is automatic.

Endpoint config (0600) on EACH gate, explicit absolute paths:

```json
{
  "version": 1,
  "gate": "h8",
  "incarnation": "h8-reviewed-generation-1",
  "repo": "/home/free/src/fabrig/queue",
  "artifacts": "/home/free/src/fabrig/mq-h8-artifacts",
  "jobs": "/home/free/src/fabrig/mq-h8-jobs",
  "runtime": "/home/free/src/fabrig/mq-runtime/REVIEWED-TIP",
  "disk_floor": 45,
  "heavy": "/home/free/src/fabrig/main/tools/heavy/heavy",
  "heavy_sha256": "COMPUTED-64-HEX-DIGEST",
  "heavy_root": "/home/free/src/rust",
  "heavy_tmp": "/home/free/src/rust/tmp"
}
```

For h2 use gate h2, a fresh distinct incarnation, and its own directories.
`supervisor.heavy_identity(absolute_heavy_executable)` computes the digest of
heavy/its five companions without opening heavy state. Use an isolated Python
import from the frozen runtime; set `sys.dont_write_bytecode=True` before local
imports. `worker.py identity` gives the request runtime (default floor 45).
Canonical config digest is SHA-256 of `durable.encoded(parsed_config)`, including
its newline. Do not guess or paste a digest from another runtime. Private files
must remain 0600; a mutable installed heavy path changing bytes blocks the old
endpoint, so pin/retain that reviewed heavy runtime before starting production.

## 2. Harmless protocol and pinned Git-only smoke

First use PRIVATE disposable origins/clones/stores on h8 and h2, not production
origin or the live queue layout. Keep their source/runtime clean. Review the
fixture paths before every command. Run `test_transport.py`, `test_supervisor.py`
and `test_operation.py` there with fixed 60-second entry limits and closed stdin;
record counts, wall times and logs. Their fake SSH/heavy cases are offline only.

Create a small disposable bare origin with one README commit P and a child C
changing only unread Markdown. Clone independent preparation/authority/two gates.
In preparation, pin refs `refs/mq/jobs/smoke-docs/parent` at P and `/candidate`
at C. Export a self-contained bundle including both refs. Construct request:

```json
{
  "version": 1, "attempt": "smoke-docs", "generation": "private-smoke",
  "gate": "h8", "parent": "FULL-P-OID", "candidate": "FULL-C-OID",
  "branches": [], "selection": "docs", "runtime": "REPLACE-WITH-IDENTITY-OBJECT",
  "plan": []
}
```

Use the actual identity object, not that placeholder string. Bundle metadata
is absolute path/bytes/sha256/exact refs map. Send `Transport.start(request,
bundle)` using the frozen transport API below; poll `status` for the SAME request.
Retain all bytes. Confirm exact C/tree, empty computed plan, full checks log,
terminal supervision and release receipt, and unchanged bare-origin main P.
Repeat identical start after deliberately dropping its acknowledgement; one job
and one result must exist. A new attempt must have a new identity. Repeat h2
through the actual pinned SSH endpoint, using a trusted prepopulated known-hosts
file; no key discovery, credential copying or StrictHostKeyChecking bypass.

Transport API from a Python `-I -B` driver outside candidate source:

```python
# Load transport.py by its explicit reviewed path using importlib, with
# sys.dont_write_bytecode=True before local imports.
t = transport.Transport(endpoints, timeout=60)
t.start(request, bundle)     # bundle bytes read from its checked absolute path
reply = t.status(request)   # running/unknown/terminal/reconciled
# terminal: exact request, result/log/supervision bytes and verified evidence
```

`endpoints` maps h8/h2 to exact transport entries:

```json
{
  "host": "local",
  "config": "/ABSOLUTE/PRIVATE/endpoint.json",
  "config_sha256": "CANONICAL-ENDPOINT-DIGEST",
  "endpoint": "REPLACE-WITH-ENDPOINT-CONFIG-OBJECT",
  "known_hosts": null,
  "known_hosts_sha256": null
}
```

The h2 entry uses host alias `hetzner2`, private h8 known-hosts path/digest, and
h2's config object/path. SSH runs one configured `bash -lc` program; body blobs
are never shell source. Inspect returned binary framing/digests; no stdout
banner is acceptable. To exercise a refusal harmlessly, use a mismatched runtime
request or corrupted bundle in a NEW private attempt and verify no checking,
main push or ready rejection. Retain evidence of the refusal.

## 3. Genuine nonempty checks and heavy proof

Still on PRIVATE disposable origins and dedicated gates, create a candidate
adding only `tools/test_smoke.py` with `print("smoke")`. Compute the exact docs
selection/tools plan using `worker.check_plan`; do not handwrite arbitrary
commands. The endpoint must execute that ordinary Python check and return its
complete result/log/supervision into controller-owned storage. Repeat with exit
1; it is a complete red check with no worker origin effects.

Next validate a genuine heavy command, separately on each dedicated gate, with
a harmless ordinary Python payload. In an isolated supervisor job use
`Commands.run(["heavy", "--gate", "python3", "-I", "-B", "-c", "print('smoke')"],
cwd=dedicated_clone, stdin=DEVNULL, stdout=private_log, stderr=STDOUT)` under the
same subreaper/owned checkout setup as `supervise`. This is a proof fixture,
not an arbitrary request-plan override: no production candidate is certified.
Inspect its pre-spawn command intent/unique mq digest-index label, actual
admitted token/id/queue, released-and-empty receipt, sealed log digest/bytes and
command completion journal. Query that EXACT label with the configured heavy
`--completion-query`; compare repo, commit/tree, args, token/id and receipt.
No mutation of heavy files or synthetic receipt is permitted.

Then use a tiny valid Cargo workspace in the private origin (ordinary harmless
unit test), resolve metadata normally and allow the recomputed full/package
plan through the endpoint. This exercises planned heavy clippy/build/default/
fuzzy entries and their exact profile/exit interpretation on real tools.
Record one command at a time per gate, disk floors, complete worker and heavy
logs, receipt identities and independent controller certificate validation.
Run on BOTH h8 and h2. Real execution of this step is Ops' task; offline fakes
are insufficient evidence. No timeout or budget increase is an accepted fix.

Exercise controller/SSH client loss while a pipe-controlled harmless check
continues. Verify no duplicate start, marker persists, no checkout writer
enters, then normal supervisor completion releases it. For separately approved
worker-loss validation, kill only the private supervisor process; do not stop
heavy units. Verify UNKNOWN and marker refusal while heavy may continue, then
use heavy's supported recovery tools. Same-boot lost-supervisor release MUST
refuse. A dedicated-host reboot, if separately authorized, supplies the old
process-lifetime boundary; after heavy recovery all matching labels must be
released/contained before the endpoint `reconcile-reboot` action can release.

For that endpoint action call `t.rpc(request, "reconcile-reboot")`. Its returned
reconciliation receipt is checked and retained; then controller
`train-reconcile` records INFRA and releases physical ownership without
manufacturing green/red. Missing task is admissible only after boot change;
unknown, ambiguous or unreleased heavy evidence remains blocked. Preserve dirty
checkout changes for inspection. Never reset/remove markers as reconciliation.

## 4. Explicit controller configuration and operation

Only after the private real proofs and review, prepare (do not implicitly
activate) h8's 0600 train config:

```json
{
  "version": 1,
  "authority_dir": "/home/free/src/fabrig/queue-authority",
  "authority_repo": "/home/free/src/fabrig/mq-authority",
  "origin": "EXPLICIT-REVIEWED-ORIGIN",
  "leads": {"ops": "REVIEWED-OPS-ROUTE"},
  "preparation": "/home/free/src/fabrig/mq-preparation",
  "artifacts": "/home/free/src/fabrig/mq-preparation-artifacts",
  "generation": "reviewed-generation-1",
  "batch": 16, "full_every": 5, "disk_floor": 45,
  "endpoints": {"h8": "FULL-ENTRY-OBJECT", "h2": "FULL-ENTRY-OBJECT"},
  "rpc_timeout": 60,
  "stop": "/home/free/src/fabrig/queue.stop",
  "output": "/home/free/src/fabrig/notes/queue-train.out"
}
```

Replace all placeholder strings with reviewed values/objects and every lead
route needed by ready inputs. RPC timeout is a new explicit transport bound;
it changes no checker timeout or cadence. Freeze this exact config/runtime
for the generation. Do not change batch/policy/path/known-hosts/runtime bytes
in place. Confirm sole h8 authority and drained legacy under the same lock.
Start only after separate activation approval with the reviewed frozen mq:

```text
mq run --train-config /ABSOLUTE/PRIVATE/train.json
mq train-status --train-config /ABSOLUTE/PRIVATE/train.json
mq train-stop --train-config /ABSOLUTE/PRIVATE/train.json
mq train-drain --train-config /ABSOLUTE/PRIVATE/train.json
mq train-reconcile --train-config /ABSOLUTE/PRIVATE/train.json
mq train-resume --train-config /ABSOLUTE/PRIVATE/train.json
restart --train-config /ABSOLUTE/PRIVATE/train.json --wait 1800 --batch 16
```

Use explicit frozen script paths, not an uninstalled command assumption.
Status is read-only and reports unobserved main as unknown. Reconcile admits
nothing; resume requires both gates/preparation/promotions drained and outbox
delivered. Stop never starts replacement or bisection groups. Restart stops and
waits for the actual authority holder, reconciles BOTH physical gates under
that same authority, then relaunches the SAME runtime/config via acknowledged
inherited authority. Timeout, unreachable owner or UNKNOWN leaves stop and
starts nothing. RPC and authority-effect clients inherit the remaining drain
deadline; its expiration does not extend the wait. A completed preparation
interrupted before dispatch resumes only its exact admitted request; incomplete
preparation remains blocked. There is no automatic fetch/update of a frozen generation.
Keep returned evidence/logs, branch pins and stable event IDs; delivery failure
never changes an outcome. Monitor useful overlap, invalidation, ready wait,
full cadence, infra episodes and main's exact ordered P/C ancestry.

## 5. Return to legacy and retained evidence

After stop/drain/reconcile and ALL outbox deliveries, `mq train-retire
--train-config /ABSOLUTE/PRIVATE/train.json` writes an immutable retirement
audit only when no active/unknown jobs, incomplete preparation or pending
bisection remain. Finish pending bisection by explicit resume before stopping
again; retirement never silently abandons it. Legacy then verifies the exact
retained snapshot/owner/operation hashes under common authority. Keep every
file; deleting a snapshot is not retirement. Retired generations cannot revive.
Deploying changed runtime/new train generation in that retired layout needs a
separately reviewed succession procedure; current restart only relaunches the
same frozen generation. There is no artifact/pin GC or age-based deletion.

Any unexpected main, certificate/receipt mismatch, unreturned log, unknown
version, missing admission identity or unresolved lifetime keeps promotion and
reuse blocked. The report must name exact remaining evidence and supported
operator action. No real proof or activation is implied by this runbook.


## 6. Repair priority and separate launcher validation

The integrated source retains the reviewed legacy repair triplet, hold,
consumed-pin restart, refusal audit and unknown-heavy ownership receipts.
Without an explicit train configuration that path remains the default. Train
configuration combined with legacy repair/runtime options refuses before
launch; repair refusals retain their exact identity in the external audit.
Restart's complete frozen manifest matches the worker's fourteen files,
including wire, transport, supervisor and operation. This changes no live
frozen external runtime or installed program.

Independent integration review, the separate launcher CPU repair and real
private h8/h2 proof remain required. The reported h8 launcher CPU result is
19.682 seconds against the unchanged limit of 15; passing local queue fixtures
does not close it. The port-race/hold/tidy changes on main supply no proof that
the original operational port gap is repaired. Production train remains OFF.

## 7. Reviewed deadline and completed-preparation recovery checks

The tools' existing 60-second limit covers fork descendants through ECHILD,
including descendants that create another session. The supervisor waits on
SIGCHLD events, and after expiry signals only pidfd identities proved to be
its adopted children. Timeout becomes code 124/red only after all descendants
are reaped; a zero direct-child exit cannot certify green after the deadline.
While containment is unresolved, the command has no completion journal or
terminal receipt and its checkout marker/ownership remain held. A kill request
or an unkillable child never supplies release proof. No extra timeout budget
is granted to make the result green.

On recovery, inspect the preparation marker before polling either gate or
adjudicating any result. The endpoint's exact attempt/generation/state marker
must match a snapshot-backed COMPLETED preparation and its retained objects/logs.
This applies even when its record has a conflict/red outcome or has been
invalidated and removed from active. A completed successor's marker must not
make its predecessor's certificate fail. Unknown/mismatching/incomplete markers
refuse before polling; never remove them by hand.

Operation scopes policy Git, both Cargo metadata calls, certificate/chain/
artifact validation and preparation runners, including calls without a drain
deadline. The enclosing drain/restart deadline still bounds scoped calls; no
expired client starts. Before launch, an immutable intent in the authority's
0700 `preparation-clients/` journal and a distinct 0600
`.git/mq-preparation-unresolved.json` client marker must both publish durably.
This private version-1 `preparation-client-v1` identity binds a unique client,
exact attempt/generation/state path and snapshot digest, candidate, argv/cwd/
input digest, and any preceding normal preparation marker. Calls without an
admission-specific context use null attempt/candidate fields and retain the
exact snapshot digest; certificate validation binds its exact request.
Unknown versions/fields refuse; no stored
state/request/config version is relaxed or migrated.

Each client runs under an isolated Linux subreaper which inherits the held
authority/checkout descriptors. Captured channels remain private journal files.
Only ECHILD for that complete command family permits its exact immutable
completion journal. A direct zero exit, EOF, closed inherited descriptors,
released lock, PID absence or termination request is insufficient. The live
caller validates that proof and the unchanged client marker before removing it
or restoring its exact preceding normal preparation marker. A normal completed
preparation marker supplies only its own snapshot/artifact proof and never
supplies client completion proof.

On expiry the owning command's session receives a termination request; escaped
descendants can survive. The distinct pre-launch marker already retains their
ownership even if controller loss precedes timeout, marker publication is
followed by an I/O error, or snapshot publication of preparation_unresolved
never completes. Where an attempt exists, timeout also records that existing
flag. Physical gate ownership, stop and returned receipts remain retained;
no candidate is promoted or replaced. Drain refuses an unresolved marker.
Publication failures poison the live controller and preserve its held leases.

Reload refuses client markers before completed-preparation reconciliation or
checkout reuse. This repair exposes no recovery action to consume orphaned
client journals, even if a lost caller's subreaper later produced a completion
journal. Unsupported interrupted-client and incomplete-preparation recovery
remain blocked for separately reviewed supported evidence; never remove or
rewrite markers by hand, infer release from absence or fabricate certificates.

Repeat these checks only in the private fixtures during the separate approved
real validation: zero-exit parent with a blocked session-escaped descendant;
controller loss before client timeout and after client marker publication but
before the unresolved snapshot flag, with escaped descendants alive through
reload/reuse attempts; controller loss after successor preparation publication
before marker removal; and bounded certificate metadata/Git. Record stop,
marker, snapshot, intent/completion journals and result evidence. The offline tests exercise these local
boundaries; they activate no queue and prove no real heavy/systemd containment.

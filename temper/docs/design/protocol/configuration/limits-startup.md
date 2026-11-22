# Limits checked at startup

This document holds the limits checked at startup, split from [`limits.md`](limits.md).

## 1. Startup checks

- **Limits checked against each other at start,** each failure naming
  both limits:
  - the forge component's: its connections equal the forge client's
    calls in flight; its `kept_bytes` is the forge client's `calls`
    times its `answer_bytes`, set by temper, not configured (skein's
    `forge/connection.md`, 4.6); its deadline is at most the forge client's
    lifetime and, with a GitHub forge, at most 270,000, as the bullet on
    the forge client's lifetime says; its body bound holds the forge client's largest write and
    a key's marker; every listing's page size is at least 1 (../forge/README.md,
    section 6);
  - the checkout's capacity, whole: `checkout.conflicts`,
    `checkout.path_bytes` and `checkout.repositories` feed every bound
    that carries a merge's conflicted paths, and the start derives each
    from them before any of its other checks returns:
    `git_component.stdout_bytes`, at least `checkout.conflicts *
    (checkout.path_bytes + 1)` (../git.md, section 8); the agent's
    conflicts and conflict path bytes, at least the checkout's (the
    worker's, below), within smith's channel ceilings, 1,024 conflicts
    of 4,096 bytes; the worker wire's saved work, `Saved`'s encoded
    bound, within jig's ceiling for it, 16 MiB (../channel.md, section
    8); and the largest answer a run's saved work makes, a frame with
    its header, within each link's room, the engine's and each worker's
    `io.output`. A configuration one of them does not hold is refused
    once, naming every one that falls short, a line each, as the start's
    other refusals name that bound, after a first line naming the
    checkout's values they follow:

    ```
    checkout.conflicts 1024, checkout.path_bytes 4096, checkout.repositories 2: raise these with them
    git_component.stdout_bytes: 262144 bytes, short of checkout.conflicts * (checkout.path_bytes + 1), 4195328 bytes
    engine.limits.io.output: 1048576 bytes, short of the largest answer a run's saved work makes, 8401196 bytes
    workers[0].limits.io.output: 1048576 bytes, short of the largest answer a run's saved work makes, 8401196 bytes
    ```

    Both profiles hold 512 conflicts of 255 bytes in 2 repositories
    (`stock.md`), and an operator may raise `checkout.conflicts` to
    1,024, temper's payload ceiling, at those path bytes, with
    `git_component.stdout_bytes` alone raised with it, which this
    refusal names; nothing derived from the checkout's values refuses
    on its own;
  - the git component's: each rule of ../git.md, section 8 (its operations
    at least the checkout's workspaces, its output caps at least what
    it parses and the diagnostic it keeps);
  - the records': `records.value_bytes` within the largest value jig's
    secrets' store keeps (jig's `protocol/store/secrets.md`), and
    large enough for the maximum encoded `AppKey`, `Accepted`, `AppSlug`
    and `Origin`: the component's `key_bytes`, 8 bytes per
    `forges.installations`, or `forge.name_bytes`, each with its version
    byte; `Origin` counts both its kind and version prefixes and the
    largest API address, a host of 253 bytes with its port
    (../credentials.md, 2.1);
  - the forges': `forges.installations` at most
    `forge_component.installations`; `forges.log_origins` at most
    `forge_component.log_origins`, and each kind's origins within it; `forge_component.renew_before_ms` below the hour an
    installation's token lives; `forge_component.log_catch_up_ms` at
    least `forge_component.log_poll_ms`; each of these limits, and
    `routes.query_bytes` and `picker.page`, above 0;
  - the setup's and the box's: each backoff at most its maximum;
    `setup.backoff_max_ms` below `setup.deadline_ms`; `box.backoff_max_ms`
    below `box.window_ms`; `setup.states`, `setup.state_life_ms`,
    `setup.runs` and `setup.fetch_bytes` above 0; `box.log_bytes` at least
    one whole record (the time, the box's longest piece and a line feed);
    `box.tail_lines` above 0 and at most 100; `box.line_bytes` at
    least 64, which holds each line but a `failed` one's `WHY`, which
    the box cuts to fit (../setup/box.md, 1);
  - the bundle's: `clients.bundle_files` and `clients.bundle_bytes`
    above 0;
  - the sign-ins': each above 0; `sign_in.body_bytes` within skein-oauth's response bound;
    `sign_in.members` within the forge's bindings' bound;
  - the facts': `forge.judge_projects` at most 1024, an eighth of the bound on
    a party's facts on the client's wire
    (`../../client/forge/records.md`);
  - a tier's failing tests, within a finding's signature, so that a
    tier's ids always fit a finding whole (`../../domain/findings.md`,
    section 4; `../../draft/after-landing-repairs.md`, section 4):
    `forge.tiers.failing_tests` at most 64, the most ids a signature
    holds, and `forge.tiers.test_id_bytes` at most 512, the most bytes
    it holds of an id. Each names its field and its bound:
    `application.limits.overrides.forge.tiers.failing_tests: 65, past
    its bound of 64, the most a finding's signature holds of a tier's
    failing tests` and
    `application.limits.overrides.forge.tiers.test_id_bytes: 513, past
    its bound of 512, the most a finding's signature holds of a test's
    id`. Both bounds are the signature's own, which the finding's
    check of a signature reads too. The signature's 512 is within the
    store schema's ceilings on a failing test's id and a flake's
    (`../store-versions.md`, 1), 1,024, which a focused test pins, so this check
    stands for the store's too, run before the store's sizing, which
    would otherwise refuse a longer id naming a schema field;
  - the store's tier rows': `forge.changes` at most the most they hold,
    `(c - 1) / 2` for `c` the least of the store schema's ceilings on a
    tier row's five lists of landings (`../store-versions.md`, 1), 768 at their
    1,537, checked before the store's
    sizing, which would otherwise refuse it as past its codec's bounds,
    naming no limit:
    `application.limits.overrides.forge.changes: 769, past its bound of
    768, the most the store's tier rows hold (2 * forge.changes + 1
    within 1537)`. Within it, a store bound the larger rows outgrow
    refuses the start as the store's sizing names it (`engine.limits.store:
    the store's map value, N bytes, past its bound of M bytes`), the
    store limit to raise with it;
  - the landing log's: `forge.landings_kept + forge.landings_pending`
    at most the store schema's ceiling on a log's landings, 256, and a
    tier's posted comments, `(forge.landings_kept +
    forge.landings_pending) * forge.landing_members + forge.changes`,
    at most its ceiling on them, 8,192 (`../store-versions.md`, 1), each checked
    before the store's sizing and naming `forge.landings_pending`:
    `application.limits.overrides.forge.landings_pending: 241, past its
    bound of 240, the most the store's landing log holds beside
    forge.landings_kept 16 (within 256)`;
  - the webhooks' listener's: each of its limits above 0;
    `hook_secret_bytes` at least 32; `hook_head_bytes` at most the
    HTTP server's largest read within io's intake; `hooks` within the
    sockets left once the clients' listener and the forge's connections
    are counted. The sockets are temper's own, a constant of
    `temper-service`'s limits module that its io check takes in place
    of the sum of jig's engine's and worker's, and hold, under `standard`, a
    document with both listeners and a sign-in, as the owned example has (../setup/init.md,
    1);
  - the channel's (../channel.md, section 8): each of temper's largest
    parts within the ceiling jig's schema gives its field; the largest
    part the engine can send within what the worker's limits take; the
    checkout's `repositories` used by both sides, independently of the
    forge's watched-name bound; the longest remote the routing can make
    on the machine's places (the longest of `machine`'s remotes' bases
    and github.com's, an owner and a name of the forge's `name_bytes`,
    two `/` and `.git`), and the forge's names and branches, within the
    checkout's `name_bytes` (../channel.md, section 8); and the smallest
    resumed start the engine's link can cut a start to (jig's
    `protocol/channel/README.md`, 8 and 10), within the engine's room, its
    `io.output`, so a run that committed every call `core.call_records`
    admits can still resume. Its failure names both: `engine.limits.io.output:
    4096 bytes, short of the smallest resumed start, 27201 bytes`, the
    start's frame with its header, as `standard` has it. A document
    reaches it only by an `io.output` override below that start, which
    the worker wire's ceilings keep at most 218,177 bytes;
  - the worker's, against the agent's configuration
    (`temper-worker-domain::valid`, entrance constraints and count
    representability, with no heap sum), among its rules: the
    checkout's repositories, names, conflicts and paths within the
    agent's directories, their names, conflicts and paths; the agent's
    largest call within the checkout's commit message; and, of the hub's
    (jig's, in the worker's `limits`), its accounts within the agent's,
    its largest event with smith's 2-byte separator within the agent's
    message, and the agent's message less that separator within the
    hub's event. So `hub.event_bytes` is the agent's `message_bytes` less
    2, and no other value starts. Payload-related refusals name the
    exact override, its configured bytes and its required lower or
    upper bound: `checkout.message_bytes`, `hub.event_bytes` and
    `hub.turn_queue_bytes`. The event's requirement names
    `agent.json`'s `profile.declared.tool_payload` less the two-byte
    separator, so the value to set is explicit;
  - the turn's, against the agent's configuration: smith's largest
    turn, the `turn_body` its configuration derives (smith's
    `smith-agent-service`, `profile`; its models' reasoning allowances
    among what it derives from, `worker.md`, section 2), is a whole conversation
    turn's body as the hub measures it, and each bound on its way holds
    it. Within each hub's `turn_bytes`, the engine's and the worker's,
    since each side of the link takes a turn of its own hub's bound;
    as a frame of the worker's link, header included, within the
    worker's `io.output`, the room its stream grants at once, or the
    worker's link is not made (jig's `protocol/channel/README.md`, 8 and 10);
    with its 4-byte length, within `core.resume_bytes`, so that a chat's
    largest turn resumes rather than starting afresh, and so within
    `root.journal.transcript_bytes`, a journal row's (the resume
    setting's check below); within `root.loads.transcript_bytes`, a
    loaded row's, and with a row's slot, within `root.loads.bytes`, a
    loaded page's, so that the engine loads it back; and with a read's
    row, within `root.loads.read_bytes`, so that a person's read
    carries it whole. Then the engine's resume window, as each hub
    counts it in an assignment's transcript (jig's `domain/hosts/README.md`):
    `core.resume_bytes` and, for each turn it can hold, a box past the
    4 bytes it already counts, within each hub's `transcript_bytes`.
    Each failure names the bound, its bytes, and the figure it must
    hold: `workers[0].limits.overrides.hub.turn_bytes: 65536 bytes,
    short of the agent's largest turn, 1066989 bytes`.
- **In the cluster's shapes** (`../cluster.md`, 1), each process checks what
  its document holds: an engine alone, every check above but the
  worker's and those of the turn's that need the agent; a worker alone,
  the worker's, the checkout's and the git component's, the channel's
  for what its side receives, and the agent's largest turn within its
  hub's `turn_bytes`. What only an engine's document and a worker's
  hold together, the engine's largest part against the worker's
  `limits` among it and the agent's largest turn against the engine's
  bounds, is held by the cluster's description, whose one template
  writes the same `application.limits` into every document (README,
  15.5), and at the link's opening by jig's hello and terms (jig's
  `protocol/channel/README.md`, 4.3).
- **Call input derivation for a local agent:** when
  `core.call_answer_bytes` is omitted from an engine's or worker's
  `limits.overrides`, temper uses the larger of its selected jig profile's
  allowance and the agent's `profile.declared.tool_payload`. An explicit
  override stays unchanged, including an explicit profile-default value.
  This composition precedes the core, store, worker wire and memory bounds;
  offline store readers use the same checked agent metadata. Engine-only
  processes retain their configured allowance. A cluster template must give
  its engine and workers consistent explicit allowances; per-kind terms
  reject a disagreement before hello.
  An explicit allowance below the local agent's `tool_payload` is
  refused at startup by its path, in two lines: what it is short of and
  the remedy, then the bounds a call meets, for instance
  `workers[0].limits.overrides.core.call_answer_bytes: 2048 bytes, short
  of the agent's call input (profile.declared.tool_payload), 4096 bytes;
  raise it to at least 4096 bytes, or omit it to follow the agent's call
  input` and `workers[0].limits.overrides.core.call_answer_bytes: call
  input 4096 bytes; wire answer 2048 bytes; smith host_reply_bytes 4096
  bytes`. Every entrance that composes the allowance refuses it alike,
  the offline store readers included. An answer past the agent's
  `host_reply_bytes` reaches its run as smith's `TooLarge`
  (`../../domain/agent/tools.md`).

## 2. Open questions

- **The rest of an assignment's transcript and of a person's read.**
  The turn's check (`limits.md`) holds the resume window's share of each
  hub's `transcript_bytes`, and the turn's share of
  `root.loads.read_bytes`. The answered calls an assignment carries
  beside its window (`core.call_records`, each within
  `core.call_answer_bytes`), and the owner's task record and ancestors
  a read carries with its row, have no bound the start can name: jig
  gives neither's worst case. Default: the hub refuses an assignment
  past its bound at run time, and the documents leave room beside the
  turn for the read's evidence (../setup/init.md, 1).

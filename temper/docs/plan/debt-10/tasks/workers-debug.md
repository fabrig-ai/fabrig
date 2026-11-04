+++
id = "workers-debug"
part = "findings"
kind = "feature"
needs = []
files = ["jig/crates/jig-protocol-engine-workers/src/host.rs", "jig/crates/jig-protocol-engine-workers/src/component.rs", "jig/crates/jig-protocol-engine-workers/src/connection.rs", "jig/crates/jig-protocol-engine-workers/src/boundary.rs", "jig/crates/jig-protocol-engine-workers/src/tests.rs", "jig/crates/jig-protocol-engine-workers/src/tests/debug.rs", "jig/testing/jig-test-service/src/tests/engine.rs", "jig/docs/design/protocol/service.status.md"]
effort = "high"
minutes = 40
+++

# jig: the workers' side shows no grant in `Debug`

## Goal

`jig_protocol_engine_workers::host::Workers` derives `Debug` and can print grant values, against
`jig/docs/design/protocol/service.md`, 11: a `GrantValue::Credential`'s token, once encoded, waits as
`Lower::Write(OutputDown::Send { .. })` in `Workers.out`, in `Workers.io` as `io::Request::Output`, and in
`Connection.below`. Give the part hand-written `Debug`s that count, and a canary test that fails if the
bytes come back.

Acceptance:

- `Workers`, `Component`, `Request` and `Event` implement `Debug` by hand, as 11.3 says: counts and
  variants, no bytes. `{:?}` of each holds no byte a queued write or read holds.
- The canary tests below pass, and each bites (`notes/common.md`).
- No public field, method or variant changes; every world that matches on `Request` or `Event` compiles
  untouched.
- `jig/docs/design/protocol/service.status.md` loses its **Not yet** line for 11.3 and its date is today's.
- **Size cap:** net at most +80 lines, tests included.

## Inputs

- `temper/docs/plan/debt-10/notes/common.md`, `AGENTS.md`, `jig/AGENTS.md`.
- `jig/docs/design/protocol/service.md`, 11 (its "A part that holds credentials") and 11.3 ("Its `Debug`
  counts"), on the trunk. 11.4 and 11.5 say the same of `Server` and `Hosted`.
- The form to follow: `impl core::fmt::Debug for Hosted` in `jig/crates/jig-protocol-store/src/host.rs`, and
  those for `Server` and `Routed` in `jig/crates/jig-protocol-engine-clients-http/src/server.rs`.

**What this cut surveyed, every type of the shipped parts (service.md, 11's table) that can hold those bytes:**

| Type | Holds | Verdict |
|---|---|---|
| `workers::host::Workers` | `out: Queue<Request>`, `io: Queue<io::Request>`, `component` | derives: hand-write |
| `workers::Component` (public, and a public field of `Workers`) | `connections` → `Connection.{above, below}`, `Machine` | derives: hand-write |
| `workers::Request` | `Lower { lower }` | derives: hand-write |
| `workers::Event` | `Lower { event }`: a read's bytes, which carry a worker's `Credential` in the opening | derives: hand-write |
| `workers::GrantValue` | `Secret`, whose own `Debug` gives only its length | sound |
| `workers::host::Output` | `Hello`, `Up`, `Lost`: what comes up, past the opening; no grant goes up | sound |
| accounts `Accounts`, `Output` | implement no `Debug` | sound |
| accounts `Runtime` | endpoints and origins, no credential | sound |
| store `Hosted` | counts | sound |
| store `Host`, `Submission` | file requests and events, which skein's `Debug`s hold back (`file.rs`, `file_layer.rs`'s `Contents`) | sound by reading: step 3 checks it |
| `worker_agents::sources` `Resources` | hidden | sound |
| server `Server`, `Routed` | count | sound |
| clients `Bridge`, `Endings` | tokens and numbers | sound |
| temper's and jig's adapters `Workers` | derive, over this host | sound once it counts |

## Owns and may change

- Owns: `jig/crates/jig-protocol-engine-workers/src/{host,component,connection,boundary}.rs`, its `tests.rs`, the
  new `src/tests/debug.rs`.
- May change: `jig/testing/jig-test-service/src/tests/engine.rs`, one new test at its end;
  `jig/docs/design/protocol/service.status.md`, its 11.3 line and date alone.
- `Connection` is `pub(crate)`: once `Component` is hand-written, drop its derive if nothing needs it. If a
  lint or a test does, leave it and say so.
- If store's `Host` or `Submission` turns out to print a kept value (step 3), hand-write it the same way, in
  `jig/crates/jig-protocol-store/src/host.rs`, and say so in the report.
- skein's types (`Lower`, `Machine`, `io::Request`, `stream::OutputDown`) are out of scope: they are not a part,
  and the parts' `Debug`s don't reach them. List under `## Left` that `{:?}` of `Workers.io` taken alone prints a
  write's bytes.

## Steps

1. `impl core::fmt::Debug` for `Workers` (listening, opened, ending, and the lengths of `io`, `out`, `sockets`,
   `listeners`), `Component` (life, connections' count, dropped downs), `Request` and `Event` (each its variant
   and its tokens; `Lower` by which of read, write or finish; `Down` by its grants' count). Remove the derives.
2. New `src/tests/debug.rs`, `mod debug;` in `src/tests.rs`:
   - `a_queued_write_shows_no_bytes`: a `Request::Lower` and an `Event::Lower` holding a canary, and a `Component`
     with an accepted connection (`fixtures::component()`) whose `below` holds it. `{:?}` of each holds no canary.
   - The check is on what `{:?}` of bytes prints: a `Box<[u8]>` formats as `[99, 97, ..]`, so look for the decimal
     list as well as the text, or a restored derive passes. A small helper in this file does it.
3. At the end of `jig/testing/jig-test-service/src/tests/engine.rs`, `the_workers_host_counts_what_it_holds`: from
   `fixtures::engine`, push an `io::Request::Output` holding a canary onto the public `workers.host.io`, and check
   `{:?}` of `workers.host`. Then check the table's one open row: format a store `Host` with a queued write in your
   slot, read it, and throw that code away.
4. Bite both: restore each derive, see the test fail for it, put it back.
5. Update the status file. `heavy cargo nextest run -p jig-protocol-engine-workers -p jig-test-service`, then
   `heavy cargo check -p jig-engine-workers-world -p jig-worker-link-world -p temper-service --tests`. fmt, clippy,
   old forms, citations. Commit, push.

## Report

As the implementer's text says, with `notes/common.md`'s table of what was broken and what failed, and the
branch's net lines.

+++
id = "dismissed-facts"
part = "client"
kind = "feature"
needs = ["dismiss-wire"]
files = ["temper/crates/temper-client-domain/src/forge/dismissed.rs", "temper/crates/temper-client-domain/src/tests/dismissed.rs", "temper/crates/temper-client-domain/src/forge.rs", "temper/crates/temper-client-domain/src/forge/me.rs", "temper/crates/temper-client-domain/src/route.rs", "temper/crates/temper-client-domain/src/tests.rs", "temper/crates/temper-client-domain/src/forge/panel.rs", "temper/crates/temper-client-domain/src/tests/connect.rs", "temper/crates/temper-client-domain/src/tests/pages/welcome.rs"]
effort = "high"
minutes = 35
+++

# A client's facts of its person's dismissals

## Goal

The forge's client side watches `dismissed` once for the session and keeps its rows as facts, read
through `Side::dismissed`; and it can make the request "Skip" will send (contracts K6). No page
reads them, and nothing sends the request but the story's test.

Acceptance:

- K6's module, `Dismissal`, `State`, `watch`, `ask` and `Side::dismissed`.
- The watch opens as `forges`' does: once the party is known, while the link is not signed out or out
  of date, of `c::Scope::Own`, under a handle of its own (`me::DISMISSED`, 2, beside `me::FORGES`);
  closed and its facts cleared at sign-out or out of date; a watch refused or ended stays so, with no
  rows, until the next sign-in.
- `ask` is `Ask::Family` of `c::Scope::Project(project)`, the record `Requesting { Dismiss { what } }`
  at `CLIENT_VERSION`.
- In a new `temper-client-domain/src/tests/dismissed.rs`:
  `the_watch_opens_once_the_party_is_known`, `the_document_is_the_sides_facts`,
  `a_later_document_replaces_the_facts`, `an_undecodable_document_ends_the_facts`,
  `a_refused_watch_keeps_no_rows_until_the_next_sign_in`, `signing_out_closes_the_watch_and_clears_the_facts`,
  `ask_is_of_the_projects_scope`.
- `heavy cargo nextest run -p temper-client-domain` passes; `heavy cargo check --tests -p
  temper-client-view -p temper-client-tools -p temper-client-shell-terminal -p temper-client-world`
  pass.

## Inputs

- `temper/docs/plan/to-begin-store/notes/common.md`; `notes/contracts.md`, K3 and K6.
- `temper/docs/design/client/README.md`, 4.6 ("A person's dismissals, the subject `dismissed`",
  "`Dismiss`").
- The pattern: `temper-client-domain/src/forge/forges.rs`, `watch`, `configure`, `told`'s first arm
  and `handle`; `forge/deployment.rs`, for a small module's shape.

## Owns and may change

- The files of the header, each edit its few lines: `forge.rs`, the `mod` line, the `Side` field and
  its constructor; `me.rs`, the constant; `route.rs`, `dismissed::configure` beside
  `forges::configure` in `forge_me`, and `dismissed::told` in the chain `forge_forges` tries.
- `temper-client-domain/src/tests.rs`: the `mod` line.
- `forge/panel.rs`, `tests/connect.rs` and `tests/pages/welcome.rs`: only as the Answers below say.
- `look-round-3` is on `pages/` and `forge/listed.rs`: not yours, but for that one helper in
  `tests/pages/welcome.rs`.
- No word, view, gallery sample or page reads the facts.
- Pre-approved:
  - `Side::changed` on a fact's change, as `forges` calls it: keep it, though nothing shows yet.
  - If the order of `route.rs`'s chain matters to another handle's event, put `dismissed::told`
    first in it, since it consumes its own handle alone.

## Steps

1. The module: the types, `watch`, `ask`, `configure`, `told`, the row translation from
   `w::Dismissals`.
2. `Side`, `me.rs` and `route.rs`.
3. The tests on the fixtures of `tests/forges.rs`, each biting: open the watch again after an end,
   and leave the facts at sign-out, each once.
4. The implementer's checks; commit and push.

## Report

As the implementer's text asks.

## Answers

2026-11-04. Authorize the narrow repairs to the forge-watch fixtures and the panel's cleanup, or have
their owners repair them first?

**Authorized here, as three small repairs:** a second session watch of `Family` shape is this task's
change, and its acceptance (the whole package passes) cannot hold without them. Start from the pushed
`db4b50ab39`; what it holds stands, the configuration order forges then dismissed included.

1. `tests/pages/welcome.rs`, `opened`, and `tests/connect.rs`, `watch`: select the open whose subject
   is `forge::forges::watch`'s, by comparing `subject`, in place of the last or the first `Family`
   open. They mean the forges watch; no assertion changes.
2. `forge/panel.rs`, `watched`, the `Watched::Ended` arm: push
   `Request::Client { asked: jig_client::Event::Unwatch { handle } }` beside `panel.handle = None`,
   before any `Request::PanelEnded`, as the `Watched::Delivered(_)` arm does. Read in jig: `Told::Ended`
   leaves the handle in the domain's `handles` (`jig-client-domain/src/watches.rs`, `ended`), only
   `unwatch` removes it, and a `Watch` past `limits.handles` ends with `Why::Limit`
   (`domain.rs`, `watch`). So each ended panel holds a slot until the session ends, in production
   too; the session's extra handle only brought it into the fixture's nine. Its `watch` is `None`
   by then, so the `Unwatch` sends nothing down and the tests' `is_empty` stands. The two panel tests
   named by the first run bite on it: they fail without the line.

Not this: raising `handles` in `tests/fixtures.rs` or a shell's `limits.rs`, which hides the leak;
reordering production's configure calls; changing what any test asserts.

The panel's cause is read from the code, not run: this call could not build. If the two tests still
fail with the line in, or any test's assertion would have to change, report `Question:` with the log.

`temper-client-shell-cli/src/tests/fixtures/setup.rs:25` stays out: it is the wire's consumer, not this
task's, and goes under `## Left`.

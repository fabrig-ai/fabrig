+++
id = "walk-story"
part = "walk-story"
kind = "feature"
needs = ["kept-metas", "no-group-file"]
files = ["temper/crates/temper/tests/support/forgejo_sign_in.rs", "temper/crates/temper/tests/support/owned.rs", "temper/docs/design/domain/testing.status.md"]
effort = "high"
minutes = 75
+++

# The process tier runs the walk's commands on a real Forgejo

## Goal

`an_owned_forgejo_signs_its_member_in_through_temper` is nearly the walk: Forgejo prepared and run by temper's commands, beside `temper serve`, with no box. Make it the walk's meeting point: no group's file, a bundle, the forge connected as its admin by the handover file, and the page's meta. No new test and no real host; a story that grows, by what `temper/docs/design/domain/testing.md`, 2.6 now says.

Acceptance:

- `heavy cargo nextest run -p temper --test process --profile process-temper -E 'test(=an_owned_forgejo_signs_its_member_in_through_temper)'` passes, one test selected. See `tools/tiers/packages.toml` for what that profile's run needs built and set.
- In that run `box/groups` does not exist, `forge-1/admin_token` remains after the connect, `forge list` shows forge 1 `managed` and connected, and `GET /` holds `<meta name="temper-forge-sign-in" content="1 forgejo http://FORGE other">`.
- Your report gives the story's whole seconds and CPU seconds before and after, beside the 2 s aim and about 3 s `testing.md` records. Over the tier's budget is red (`skein/docs/foundation/testing-strategy.md`, 8.1): the numbers go in the report, with where the time went.

## Inputs

- `temper/docs/design/domain/testing.md`, 2.6's bullet `Forgejo signing in through temper`, which this cut changed.
- `temper/docs/design/protocol/setup.md`, 7.4; `temper/docs/design/protocol/sign-in.md`, 9.7.
- `temper/crates/temper/tests/support/forgejo_elsewhere.rs`, `setup`: how a story makes its ctl calls.
- `temper/docs/plan/help-server/notes/common.md`.

## Owns and may change

Owns: `temper/crates/temper/tests/support/forgejo_sign_in.rs`.

May also change, and how:

- `temper/crates/temper/tests/support/owned.rs`: `tier_configuration` writes no group's file. Its other reader, `support/forgejo.rs`, then runs with none too, which is the same proof; if a test of its needs the file, write it in that test and say so.
- `temper/crates/temper/tests/support/forge_sign_in.rs`: `configure`, only if the bundle's member belongs there.
- `temper/docs/design/domain/testing.md`: the bullet's figures alone, to what you measure. `temper/docs/design/domain/testing.status.md` (departures from your text, which leaves `docs/design/` alone): remove the **Not yet** entry for 2.6's story.

Options, each pre-approved:

- The printed link is one use, and the story's browser spends it. For ctl's sign-in, read the fresh link temper prints after a refused one, or sign ctl in first and read the browser's link next; take whichever the story's stdout gives without a sleep.
- The story's stderr log may compare exactly somewhere: the new `no sign-in through temper` line before the connect is expected. Allow that line.

## Steps

1. `tier_configuration` loses its two group's-file lines. Assert in the story that `box/groups` is absent after `prepare` and at the end.
2. Before `serve`, `application.clients.bundle` names a small bundle directory the story writes: `index.html` with `</head>`, and one file in a digest directory, as `suite/startup.rs`'s `fixture_bundle` shapes it.
3. After the story's sign-in through temper, as `temper help server`'s step 6 does: `forge connect forgejo-admin forge-1/admin_token --api FORGE --git http://FORGE --key ...`, `forge list --until connected:1`. Each with `</dev/null` (`temper/AGENTS.md`).
4. Assert the acceptance's facts. `GET /` before the connect holds no such meta: read it first.
5. `cargo fmt`, `heavy cargo clippy -p temper --all-targets -- -D warnings`, the acceptance's run, push.

## Report

As the implementer's text says, with the measurements.

+++
id = "fake-backends"
part = "fake"
kind = "feature"
needs = []
files = [
  "temper/crates/temper/src/fake_backends/tests.rs",
  "temper/docs/plan/limits-heavy/figures/fake-backends.md",
]
effort = "high"
minutes = 90
+++

# `fake_backends::tests`: what does a fake machine's preparation cost?

## Goal

`temper`'s `fake_backends::tests` is 50 tests and 11.5 CPU s, about 0.23 each: `retained_peer_edits_are_checked_without_replacement` 2.2, `missing_required_retained_documents_refuse_without_writes` 1.5. They belong where they are (`plan.md`, *Belonging*). Find what a test pays for before its check, and fix the cause: a person's `temper box --fake-backends` may well pay it too.

Acceptance:

- Every test still exists and asserts what it asserts today, or is a row of a test named under *Tests that went*, by the rule in the common note.
- `temper/docs/plan/limits-heavy/figures/fake-backends.md` holds the tests you profiled and those you changed, before and after in instructions, with the cause of each.
- The figures carry a *Belonging* section after *Tests that went*, confirming or correcting `plan.md`'s reading in a line per group of tests, from reading the bodies.
- The next cause's gain is under about 0.2 CPU s, or what's left is under *Found and not fixed*.
- A product change has its own focused test, and changes no behaviour.

## Inputs

- `temper/docs/plan/limits-heavy/inputs/request.md`.
- `temper/docs/plan/limits-heavy/notes/common.md`: how to measure, the figures' form and the rules.
- `temper/docs/plan/limits-heavy/plan.md`, *Belonging*.
- `temper/docs/design/protocol/setup.md`, 7.5: what `temper box --fake-backends` promises.
- `skein/docs/foundation/testing-strategy.md`, 8, *A test is a process* and *A slow test is understood, never hidden*, and 8.1.

## Owns and may change

Owns `temper/crates/temper/src/fake_backends/tests.rs` and its figures. `limits-fixtures` isn't in this file.

May change, at the size the cause needs:

- `temper/crates/temper/src/fake_backends.rs`, `fake_backends/**`, `init.rs` and `setup*`.
- `temper-init`, `temper-fake`, `temper-service` and `skein`, `smith` or `jig` crates for a cause a profile puts there (`model-catalogue-2/stock-models` holds `temper-init/template/*`, `temper-fake/src/**` and `temper-service/src/startup*`). A file format or template's bytes stay unless the profile names them.

Options, each decided:

- **The cost is in `init::write` or `init::write_with_program`, whichever test it serves:** fix it in the product. Generating or checking something once where it is done several times is the kind of change this is.
- **The cost is key or certificate generation in an unoptimized build:** read whether the test needs a fresh key. A fixture's key may be a constant in the test where the product's key isn't what it checks; a product path that generates one stays as it is. Build settings stay.
- **The cost is the same fixture built per case** (`retained_peer_edits_are_checked_without_replacement` builds a scratch and a sandbox for each of six kinds): build it once per test and copy or reset it per case, if that's cheaper. Every case still runs from the state it runs from today.
- **Parsing or checking a stock document repeatedly:** once where once will do, in the product where the repetition is the product's.
- **A cost no test can avoid:** it stays, and goes under *Found and not fixed* with its count.
- **A test reads as the process tier's:** none is expected, and none of them spawns the binary (`rg 'Command::new|paths::binary'` finds nothing in the file). If reading its body shows one only a path through `main` could hold, leave it where it is and say so in the figures.

## Steps

1. Read `fake_backends/tests.rs` to the end, then write the *Belonging* lines.
2. Profile, with `-p temper --lib` and filters reading `fake_backends::tests::NAME`: `chosen_ports_refuses_all_but_nine_nonzero_ports` (a floor, with no fixture), `fresh_preparation_is_private_and_isolates_imported_resources` (a median), `retained_peer_edits_are_checked_without_replacement` and `missing_required_retained_documents_refuse_without_writes`.
3. Starting points, to check and not findings: `initialized_source` runs `init::write` for every test, and `prepare` runs `write_with_program` again for the sandbox, with `setup::check` and `Source::read` around them. What of that is parsing, key material or file I/O? `own_app_key_is_the_seeded_apps_private_pem` suggests a seeded key. What does `reserve` cost?
4. Fix the causes, largest first, re-measuring after each.
5. Write the figures.
6. The implementer's text's end of a run: `cargo fmt`, old forms, `heavy cargo clippy -p temper --all-targets -- -D warnings` with any other package you changed, and the focused tests of those packages and their direct dependents. Commit and push.

## Report

What a test paid for before its check, the before and after, the causes found and not fixed, and any product change with the test that holds it. Whether `temper::suite`'s `startup` or the box's first run gains too, if the cause is shared. Your implementation and check minutes apart.

+++
id = "fake-box"
part = "fake-backends"
kind = "feature"
needs = ["fake-migrate"]
files = ["temper/crates/temper-fake/src/forge.rs", "temper/crates/temper-fake/src/backends.rs", "temper/crates/temper-fake/src/lib.rs", "temper/crates/temper-fake/src/tests/forge.rs", "temper/crates/temper-fake/src/backends/tests.rs", "temper/crates/temper/src/fake_backends.rs"]
effort = "medium"
minutes = 35
+++

# Two sources on the fake backends

## Goal

`temper box --fake-backends`' `fake forgejo` peer seeds two sources and the box prints their addresses
(`temper/docs/design/protocol/setup.md`, 7.5; `temper/docs/design/protocol/configuration.md`, 12.1).

Acceptance:

- `temper_fake::forge::Document` gains `sources`, each an address, a repository to seed and an optional
  `private`, decoded and bounded by `ENTRIES`; `Document::seed` seeds each repository and gives
  `Config.sources`. A host outside `.invalid` is refused at decode, naming it, in a test.
- `backends.rs`' `forgejo.json` literal (about 425) names two: `https://code.example.invalid/acme/widget`,
  one commit on `main`, and `https://code.example.invalid/acme/gadget`, two branches, its default `trunk`,
  each with a `.github/workflows/ci.yml`, so a walk sees the starter added beside it. A test reads them
  back from the document.
- `fake_backends.rs` prints `box: copy source ADDRESS` for each, once per launch, after `box: fake forgejo`;
  a test of the lines, none existing today.
- `heavy cargo nextest run -p temper-fake -p temper` pass.

## Inputs

- `temper/docs/plan/repository-copies-1/notes/common.md`; `notes/contracts.md`, S4.
- `temper/docs/design/protocol/setup.md`, 7.5, "Sources to copy".
- `temper/docs/design/protocol/configuration.md`, 12.1.

## Owns and may change

- The files of the header.
- `temper/crates/temper/tests/fake_backends/walk/open_default.rs` is `look-round-3`'s: stay off it.
- The sources are the **`fake forgejo`** peer's alone: not `github.json`, not the playground's fake.
- Remove "7.5: ..." from `temper/docs/design/protocol/setup.status.md`, and "12.1: `box: copy source`." from
  `temper/docs/design/protocol/configuration.status.md`.
- Pre-approved: the retained `forgejo.json` of an earlier launch is refused or rewritten as today's
  `rewrite` does for any change of its document.

## Steps

1. `Document`. 2. The literal. 3. The printing: `print` is called on two paths, so once per launch needs a
look. 4. The tests, each biting: let a host outside `.invalid` through once. 5. The implementer's checks;
commit and push.

## Report

As the implementer's text asks.

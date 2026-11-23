# Codecs from schemas: the debt goal's contracts

2026-11-23. The shapes the codec debt goal's changes share and no other
area reads (`temper/docs/design/draft/design-economy.md`, 4.3). They
bind as any delivered design section does. The goal's last change
deletes this document and its status.

## 1. The memory harness while the owners move

`skein/testing/codec-memory.rs`'s `family!`, from the change that adds
`bounds_text` and the bounds files of skein's families
([golden.md](golden.md), section 1) until the goal's last change:

- **Its limits** at a clamp `k` are `CEILINGS.at_most(k)`
  ([README.md](README.md), section 4); it reads no list of limit names.
- **A `limits [...]` list** is still accepted, in its place in the
  call, and ignored, so jig's, smith's and temper's calls compile
  unchanged until their own change drops them.
- **A root's `=> (encoded, decoded)` pin** is still accepted and
  asserted against the root's bounds at the ceilings where it is given;
  a root over the cap with no pin passes, its ceiling bounds being its
  bounds file's.
- **The last change** removes both forms and the pin's assertion, once
  every call has dropped them.

## 2. Who writes which bounds files

Each tree's change writes the bounds files of the families whose drift
tests run in its tree's worlds, and drops its tree's limit lists and
pins in the same change, so that no ceiling bound goes unchecked between
them:

- **skein:** `skein/tests/codec` (its fixtures and `sample`, `mapped`),
  `skein/tests/oauth`, `skein/examples/ui/view`, and `skein-codec`'s
  own lean fixture where `skein-codegen` checks its drift.
- **jig:** the families of `jig/tests/codec-families.rs`, in
  `jig/tests/bounds/`, and those of `jig/tests/worker-wire` and
  `jig/tests/protocol-accounts`; `jig/tests/store-codecs`'
  `task_bounds.rs` goes.
- **smith:** `smith/tests/codecs`' four families.
- **temper:** `temper/tests/codecs`' five records families.

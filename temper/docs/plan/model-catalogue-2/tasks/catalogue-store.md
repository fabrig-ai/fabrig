+++
id = "catalogue-store"
part = "catalogue"
kind = "feature"
needs = ["model-surface"]
files = ["jig/crates/jig-protocol-store/src/lib.rs", "jig/crates/jig-protocol-store/src/codec.rs", "jig/crates/jig-protocol-store/src/keys/settings.rs", "jig/crates/jig-protocol-store/src/tests/settings.rs", "jig/crates/jig-protocol-store/src/tests/models.rs"]
effort = "high"
minutes = 90
+++

# The store knows a person's model at open, and each later catalogue

## Goal

The store keeps a vocabulary of today's model names beside every record that holds an index, and reads any name it doesn't know as gone. Today's names are no longer fixed at the start: at open it overlays the settings family's `model` changes, and a catalogue change replaces them.

Acceptance:

- `a_persons_model_isnt_read_as_gone_after_a_restart`: a template's choice and a policy's named models naming `Model(64)` restore at that position, through a close and an open given the defaults alone.
- `a_later_catalogue_is_the_vocabulary_of_the_next_commit`: `Store::catalogue` then a commit writes those names beside the record.
- `a_name_that_left_stays_gone`: a stored vocabulary's name in no default and no change still remaps to `ModelEffort::GONE`.
- A withdrawn model keeps its name and position.
- Focused and fuzzy tests of `jig-protocol-store` and `jig/tests/store-codecs` pass.

## Inputs

- `jig/docs/design/protocol/store.md`, section 5, *A `model` setting's value holds no index*; 5.1, 4.4.
- `jig/docs/design/domain/engine.md`, 17.11, *Who hears*.
- `temper/docs/plan/model-catalogue-2/notes/contracts.md`, 2.
- `temper/docs/plan/model-catalogue-2/notes/common.md`.

## Owns and may change

Owns the header's files. Neighbours: `jig-protocol-store/src/{component,restore}*` where open reads; `jig/testing/jig-core-kit/src/world/store.rs:216`, `temper-service/src/store.rs:167,179`, for call sites alone.

Options, each pre-approved:

- Reading: by the settings family's key range for `Setting::Model` changes, in the open step that reads the format row. If the open's reader can't take a range at that step, do it as the first read of the settings restore, before any record is decoded, and say so in the report.
- A hole's name in a vocabulary: `ModelName { name: empty, efforts: empty }`. `model_bit` never matches an empty name.

## Steps

```rust
impl Store {
    /// Today's models, before a commit that holds them (domain/engine.md, 17.11).
    pub fn catalogue(&mut self, names: Box<[jig_domain_core::settings::ModelName]>);
}
```

Each test breaks its rule once by hand. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p jig-protocol-store --all-targets -- -D warnings`; `heavy cargo nextest run` on it and its direct dependents, and with `--profile fuzzy` on `jig/tests/store-codecs`' package. Commit and push.

## Report

Where the read sits in the open, and why. What a store with no settings yet reads.

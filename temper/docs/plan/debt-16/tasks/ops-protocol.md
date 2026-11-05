+++
id = "ops-protocol"
part = "ops-protocol"
kind = "feature"
needs = []
files = ["jig/examples/ops/protocol-observability/**", "jig/examples/ops/domain/Cargo.toml", "jig/examples/ops/domain/src/translate.rs", "jig/examples/ops/domain/src/route.rs", "Cargo.toml", "Cargo.lock"]
effort = "high"
minutes = 45
+++

# jig: `ops`'s delegate decoding and alert words move to its protocol layer

## Goal

D004's part 3. `ops`'s root decodes delegate JSON and assembles alert words in its `translate.rs`. They move to a new
crate, `jig-ops-protocol-observability`, which `jig/docs/design/examples.md`, 2 names. The root calls its functions in the
same step, at the same places.

**They are not handed out by an `Output` and an `Event`, and that's deliberate.** A hand-out would put the connector's
`Save` rows and `core::Event::ProcedureStep`, which commit in one decision today, in two commits, which moves commit counts
and opens a crash window between them. Behaviour stays, so the root keeps calling. `d004-rewrite` records the hand-out as
what remains.

Acceptance:

- `rg -n 'jig_protocol_tools::batch|fn alert_words|fn health_words|fn triage|fn lead' jig/examples/ops/domain/src` finds
  nothing.
- The new crate takes the `step` profile, and the lints check passes: `python3 -I -B tools/lints/lints.py` at the root.
- `jig/examples/ops/domain/Cargo.toml` keeps `jig-protocol-tools`: `Policy.delegate` is a public field, and it stays.
- The new crate's tests pin what nothing pinned: an undecodable delegate, a batch of other than one, the exact bytes of an
  alert's words, and what `triage` appends.
- D004's Measure falls by about 70: `find jig/examples/ops/domain/src -name '*.rs' -not -name 'tests*.rs' -not -path
  '*/tests/*' | xargs cat | wc -l`.
- `heavy cargo nextest run -p jig-ops-world` passes, `fuzzy` included, with no test changed.

## Inputs

- `temper/docs/plan/debt-16/notes/common.md`
- `jig/docs/development/debt/D004-ops-root-jig-wiring.md`
- `jig/docs/design/examples.md`, 2 and 4: this cut has already written the row and the bullet you build.
- `jig/docs/design/domain/root.md`, 1 and 11.
- `tools/lints/design.md`, 1 and 5.

## Owns and may change

- **Owns:** `jig/examples/ops/protocol-observability/` (new); `jig/examples/ops/domain/src/translate.rs` (these items).
- **May change, the calls only:** `jig/examples/ops/domain/src/route.rs`, the four calls in its connector routing.
- **May change, one line each:** `jig/examples/ops/domain/Cargo.toml`, the root `Cargo.toml` (`members` and
  `[workspace.dependencies]`, beside their `jig/examples/ops/` neighbours), `Cargo.lock`.
- **Stays in `translate.rs`:** `news_class`, `read_description`, `resource_role` and the other translations of a variant to a
  variant: they're the small total functions a root keeps (`root.md`, 1).
- **Not a stop:** `jig/tests/boundary/src/tests/graph.rs`'s `application_breaches` reads names beginning `jig-protocol-`,
  so this crate, beginning `jig-ops-`, may name `jig-ops-domain-observability`.

## Steps

1. The crate, `jig/examples/ops/protocol-observability`:

   ```toml
   [package]
   name = "jig-ops-protocol-observability"
   description = "Ops's observability words and delegates, for its root"

   [dependencies]
   skein-lib.workspace = true
   jig-domain-core.workspace = true
   jig-protocol-tools.workspace = true
   jig-ops-domain-observability.workspace = true
   ```

   Lay the rest out as `jig/examples/ops/domain-connectors` is. `python3 -I -B tools/lints/lints.py --set step
   jig/examples/ops/protocol-observability/Cargo.toml`, and its `clippy.toml` if the profile asks for one.

   ```rust
   //! Ops's observability words and delegates (jig/docs/design/examples.md, 2 and 4).
   pub fn alert_words(alert: &observability::Alert) -> Box<[u8]>
   pub fn health_words(healthy: bool) -> Box<[u8]>
   pub fn triage(delegate: &[u8], alerts: &[u64], limits: &Limits) -> Option<Box<[core::Delegate]>>
   pub fn lead(delegate: &[u8], limits: &Limits) -> Option<Box<[core::Delegate]>>
   ```

   The bodies move as they are, but for `health_words`' unused parameter. Comments stay with what they say.

2. `route.rs` calls them; `translate.rs` loses them.

3. Tests in the new crate's `src/tests.rs`: `alert_words` yields exactly `alert 7 severity 3`; `health_words` both ways;
   `triage` on undecodable bytes and on a batch of two gives `None`; on a batch of one, its words gain `\nAlerts: 1 2`;
   `lead` the same on `None`, and its words unchanged. Break by hand: remove the length check in `triage`; change the
   heading; see each fail; restore.

4. `heavy cargo nextest run -p jig-ops-protocol-observability -p jig-ops-domain -p jig-ops-domain-observability -p
   jig-boundary`, then `heavy cargo nextest run -p jig-ops-world`, then the same with `--profile fuzzy` for
   `the_alert_storm_survives_the_commit`.

5. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p jig-ops-protocol-observability -p
   jig-ops-domain --all-targets -- -D warnings`; the citations check; `python3 -I -B tools/lints/lints.py`. Commit and push.

## Report

The common note's items, with the breaks above, and D004's Measure before and after.

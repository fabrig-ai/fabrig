# D002: Three crates pump the JSON tokenizer by hand beside `skein_json::parse`

- **Kind:** duplication
- **Where:** `temper-settings-form/src/json.rs`, `temper-client-tools/src/jig/json.rs`, `temper-engine-smith/src/json.rs`
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig review of that day (`jig/docs/development/debt/reviews/2026-10-31-jig.md`), theme T5's table ("JSON tokenizer pump: 2 in jig, 4+ in temper") and A7 F9
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** medium (about 120 lines fewer, and one tested loop instead of three)
- **Size:** S
- **Status:** open
- **Paid when:** no crate under `temper/crates` drives `skein_json::tokenizer` over a whole buffer; each calls `skein_json::parse` (the measure is 0).
- **Measure:** `rg -l --type rust 'tokenizer::largest_demand' temper/crates | wc -l`: 3

## What

skein-json parses one whole borrowed input to a retained document
(`skein/docs/design/json.md`, 5.3; `skein_json::parse`, added by
`1ca8281e9f`). temper still has three hand-written pumps over the
tokenizer, each with its own `Intake`, queues, step count and event
match:

- `temper-settings-form/src/json.rs`, `document_depth`, and
  `temper-client-tools/src/jig/json.rs`, `document_depth`: the same 40
  lines twice, each ending in `Document::from_tokens`, differing only in
  their error type.
- `temper-engine-smith/src/json.rs`, `parse`: the same pump, building its
  own `Value` tree from the tokens.

`temper-client-tools` already calls `skein_json::parse` elsewhere
(`src/jig/render/document/account_text.rs`, `account_lines.rs`). The two
streaming users, `temper-fake-git/src/exchange.rs` and
`temper-forge-setup-shell/src/exchange.rs`, feed a body as it arrives
and are not this item.

## Why it matters

Each pump picks its own step bound (`len * 4 + 8`) and its own refusals;
a fix to one does not reach the others, and none is tested as the shared
`parse` is.

## What pays it back

Replace both `document_depth` bodies by a call to `skein_json::parse`
with the same limits. For `temper-engine-smith`, read the `Value` tree
from the returned `Document`, or read the `Document` directly where the
callers allow. Nothing else depends on it. skein's own pumps, and the
slice-fed driver that would replace them, are skein's
`skein/docs/development/debt/D001-json-whole-buffer-reads.md`; this item
does not wait on it, since `parse` exists.

+++
id = "limits-link"
part = "limits-once"
kind = "feature"
needs = []
files = ["jig/crates/jig-client-protocol-link/src/limits.rs", "jig/crates/jig-client-protocol-link/src/tests/attempts.rs", "jig/docs/design/client/domain/client.status.md"]
effort = "high"
minutes = 40
+++

# jig: the link's wire limits come from one derivation, by class

## Goal

The link's half of `jig/docs/development/debt/D009-limits-mapped-by-hand.md`. `wire()` and `receiving()` list about 225 limits field by field, 15 through `.min(w::CEILINGS.…)`. A knob sizes a class of bounds (`jig/docs/design/client/domain/client.md`, 14), so each class gets one rule, written once, and no field decides for itself. Every configuration gets the same wire limits as at base.

Acceptance:

- The item's **Measure** reads 0: `rg -o '\.min\(w::CEILINGS' jig/crates/jig-client-protocol-link/src/limits.rs | wc -l`. That alone could be a rename, so also:
- `rg -n 'wire\.\w+ = ' jig/crates/jig-client-protocol-link/src/limits.rs` lists only `owned_record` and `document_text_bytes`, and `rg -n '= w::CEILINGS\.' jig/crates/jig-client-protocol-link/src/limits.rs` lists no field (five no-ops at base): `wire` starts at `w::CEILINGS` and stays there unless a class narrows it.
- `heavy cargo nextest run -p jig-client-protocol-link` passes, and `template_overrides_keep_the_wire_ceiling_when_the_client_accepts_more_rows`, `receiving_inventory_bounds_keep_their_schema_ceilings`, `charter_documents_use_the_clients_row_and_name_bounds`, `an_invalid_client_limit_literal_is_refused` and `invalid_receiving_limits_are_refused_at_construction` don't change. If one needs an edit, stop.
- A new test: `rows: 33`, `segments: 33`, `authority: 33` and `text: 65537` are each refused at construction. By hand: remove the class check (it goes red). Take a capped member out of its class's list (`receiving_inventory_...` or `template_overrides_...` goes red, or say which). Restore each.
- Before you change anything, a throwaway check saves `wire()` and `receiving()`'s whole value (`{:?}`) for each of the shipped clients' knobs (`temper-client-shell-web`, `-terminal`, `-tools`'s `limits.rs`, and `fixtures::limits()`); after, they're equal. Throwaway: untracked and gone before the push.
- Net production falls (estimate -10 to -20). `jig/docs/design/client/domain/client.status.md`'s **Not yet** entry for 14 goes.

## Inputs

- `temper/docs/plan/debt-19/notes/common.md`
- `jig/docs/development/debt/D009-limits-mapped-by-hand.md`
- `jig/docs/design/client/domain/client.md`, 14 (a knob and its class): written, and binding
- `skein/docs/design/codec.md`, section 4 (limits may be smaller than the ceilings, never larger)

## Owns and may change

- **Owns:** the files in `files`. No other file of the crate: `link-files` and the rest of D008 own `translate/` and `tests/fixtures/`, and run beside you.
- **The shape:**

  ```rust
  /// One class of the wire's bounds, sized by one knob. A member the schema bounds below its class keeps its ceiling.
  fn narrow(knob: u32, members: &mut [&mut u32]) {
      for member in members.iter_mut() {
          **member = knob.min(**member);
      }
  }

  fn wire(client: &jig_client_domain::Limits) -> Option<w::Limits> {
      // a knob past its class's ceiling is refused
      if client.rows > w::CEILINGS.circle_tasks || ... { return None; }
      let mut wire = w::CEILINGS;
      ...
      narrow(client.rows, &mut [&mut wire.circle_tasks, ...]);
      narrow(client.text, &mut [...]);
      ...
      w::check_limits(&wire).ok()?;
      Some(wire)
  }
  ```

  One `narrow` per knob: `rows`, `text`, `segments`, `authority`, `document_bytes` (with `owned_record`), `ask_bytes`, and the small ones (`owners`, `accounts`, `watches`, `turn_text`, `md`, ...), a class of one or more. `document_text_bytes = text.max(HOLD_ACCOUNT_BYTES)` stays as it is, a special case with its own reason.
- **The class check keeps refusal the same.** Today a plain member past its ceiling fails `check_limits`, and `narrow` would cap it silently instead. So each class's knob is checked against its class's ceiling first, read from a plain member of `w::CEILINGS`, never a literal. A class whose plain members have different ceilings is a question.
- **Membership stays listed, once per class:** generating it is `jig/docs/development/debt/D007-contract-written-by-hand.md`'s (bound classes in the schema), and this round doesn't. `limits-close` records that.
- **Capping stays:** **the cut decided it** (`temper/docs/plan/debt-19/plan.md`). Refusing a knob above a member's lower ceiling would refuse every shipped client.
- **`client.too_many_lines`:** if clippy no longer asks for the `expect`, it goes.

## Steps

1. The throwaway check, at your base.
2. `narrow`, `wire()`, `receiving()`.
3. The check again; compare; delete it. The new test. Break the rules by hand. Delete the status entry.
4. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p jig-client-protocol-link --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-client-protocol-link -p jig-client -p jig-client-protocol-http -p jig-client-protocol-world -p jig-client-world`. The citations check. Commit and push.

## Report

The common note's items, the classes and their member counts, that the values were equal, what you broke and what failed, and the net lines.

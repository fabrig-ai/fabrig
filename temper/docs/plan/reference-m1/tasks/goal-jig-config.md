+++
id = "goal-jig-config"
part = "goal-jig"
kind = "feature"
needs = ["goal-jig-surface"]
files = ["jig/crates/jig-protocol-config/src/templates.rs", "jig/crates/jig-protocol-config/src/tests/templates.rs"]
effort = "medium"
minutes = 40
+++

# goal-jig-config: the configuration reads a template's resources

## Goal

jig's configuration reads a template's optional `resources` member, its connectors' symbols
(jig's `protocol/configuration.md`, 3.3; contract C2), into `jig_domain_core::Template.resources`,
where `goal-jig-surface` left `Box::new([])`.

Acceptance:

- In `jig-protocol-config/src/templates.rs`, `template`: `resources` is optional. Absent, or
  `[]`: no symbols. Otherwise a JSON array of objects, each exactly `{"connector": c,
  "symbol": s}`, `c` and `s` integers within `u16` (any other member, a missing one, or a
  value out of range: `What::Form` at the item's path; through `read::Object`'s `finish` as
  the other members are checked).
- More items than `limits.core.tasks.holdings`: `What::Bound` at `resources`; a pair named
  twice: `What::Unique` at the second item.
- The resulting `Template.resources` keeps the configuration's order.
- jig's defaults (`defaults.rs`, if it writes jig's chat and goal) name none: no change there.
- Tests in `src/tests/templates.rs`: `a_template_naming_resources_parses_them_in_order`,
  `a_template_resource_naming_an_unknown_member_is_refused`,
  `a_template_resource_named_twice_is_refused`, `a_template_with_more_resources_than_holdings_is_refused`;
  and the existing `the_chat_and_goal_templates_parse_with_their_defaults` still passes,
  with `resources` empty.
- No status edit: `goal-jig-client` removes the 3.3 **Not yet** lines once the whole path
  holds.

## Inputs

- `jig/docs/design/protocol/configuration.md`, 3.3 (the bullets "In the format" and "Checked
  at start"), and section 10 (a template's format, which 3.3 extends).
- `temper/docs/plan/reference-m1/notes/common.md`; `notes/contracts.md`, C2.

## Owns and may change

- Owns the files in the header. The settings family of this plan changes
  `jig-protocol-config` for a charter's default wake policy (`charters.rs` and its tests);
  keep to `templates.rs` and its tests.
- **Options:** if `read::Object` has no helper for an optional array, read it with
  `object.member` as `ended_by` is read, and say so. If the checks already in
  `templates.rs` give a template its problems in another order, follow theirs.
- No other configuration document changes (no example configuration names resources yet:
  the settings family writes the template for drafts).

## Steps

1. Read `template` in `templates.rs` (how `ended_by` and `llm` are read, and the checks after
   `Template` is built).
2. Read `resources`; the four tests: `heavy cargo nextest run -p jig-protocol-config --lib -E
   'test(/tests::templates::a_template_(naming_resources|resource_|with_more_resources)/)'`
   (check four selected).
3. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p
   jig-protocol-config --all-targets -- -D warnings`; `heavy cargo nextest run -p
   jig-protocol-config`.
4. Commit ("jig config: a template's resources"), push.

## Report

The shape read, the problems given and where, the tests and counts.

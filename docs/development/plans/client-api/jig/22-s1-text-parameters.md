# Supplement jig-22.s1: a task's text parameters refused when they are not UTF-8

Added by ruling 399 (the designer's answer of 2026-10-18T22:06Z to
temper repin's question of 2026-10-18T21:56:58Z, point 2). Read first:
jig's `AGENTS.md`; `docs/design/client/domain/contract.md`, 5.3 ("a
task's spec, its words and its text parameters, refused with a spec's
other faults wherever the tasks child admits a spec … A value that is not
text, such as a commit's id, is a task's parameter only as text its
application spells"); `domain/tasks.md`, 3 and 4.

Scope: the tasks child refuses a spec whose words or a `Parameter::Bytes`
value is not UTF-8, wherever it admits one. Today `valid_spec` checks no
encoding, and the documents' projection (`documents/small.rs:11`) panics
on such a value.

Needs: nothing. `temper-repin` waits for it.

Files:
- `crates/jig-domain-tasks/src/batch.rs` (`valid_spec`, `:361`);
- `crates/jig-domain-tasks/src/boundary.rs` (the doc comments of
  `Parameter::Bytes`, `:340-345`, and `Spec::words`, `:360-362`);
- `crates/jig-domain-tasks/src/tests/`.

Shape:
- `valid_spec` also returns false when `spec.words` or a
  `Parameter::Bytes` value is not UTF-8. It checks with one private fn
  under the `#[expect(clippy::disallowed_methods, reason =
  "client/domain/contract.md, 5.3 requires UTF-8 text validation")]` that
  `jig-domain-people/src/words.rs:50-53` uses.
- Its callers refuse as they do today: `Refusal::Spec` in `batch.rs`
  (`:276-282`) and `control.rs` (`:301`); false in proposals, recurring
  and the restore (`stored.rs:177`, `:328`, `:1067`).
- The doc comments of `Parameter::Bytes` and `Spec::words` say UTF-8
  text.
- A test or fixture of jig's whose byte parameter or words are not UTF-8
  spells them as text; list each in the commit body.

Test shape, in `crates/jig-domain-tasks/src/tests/`: a batch whose member
has a 32-byte non-UTF-8 parameter is refused `Refusal::Spec`; so is one
with non-UTF-8 words, and an amendment with either. Each test under
20 ms.

Proof: jig's gate.

Size: aim at most 30 production lines added; past 60, block with the
count.

Don't: change the wire, a store schema or golden, `small::text`'s
`expect` (`documents/small.rs:11`), or `Parameter::Bytes`'s name.

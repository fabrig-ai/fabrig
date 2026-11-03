# bugs-security: standing rules

Every task of this plan names this note.

- Read `AGENTS.md`, then the `AGENTS.md` of your task's tree. Your task stays in its tree, but
  for the files its spec names in another one, each edited only as the spec says.
- **The request** is `temper/docs/plan/bugs-security/inputs/request.md`: bugs that leak a
  secret, each fixed with a test, in the lowest world that shows it, that fails on the old code
  and passes on the new. Do only your fix; what you notice beyond it goes in your report's
  `## Left`. A fix that would need a design change stops and asks.
- **The test fails first.** Write the test, run it on the unchanged code and see it fail (a
  compile-fail check: see it reject the old code, or say why it cannot be run on it), then fix
  and see it pass. The report gives both runs: the command, the count, and the failing
  assertion's line.
- **A hand-written `Debug` keeps the derived form** for every field and variant that holds no
  secret: the same type name, variant names, field names, field order and the same `debug_struct`
  or `debug_tuple` shape, so a trace line that names a variant or a field reads as before. A
  field of secret bytes shows its length (`len`) or a count, never a byte of it. Where clippy's
  `missing_fields_in_debug` fires, a scoped `#[expect(clippy::missing_fields_in_debug, reason =
  "...")]` on the impl, citing the design, as `skein_lib::Buffer`'s does.
- **A canary** is a value no other code or fixture holds (for example `b"canary-7f3a9c"`). A
  test formats the type holding it with `{:?}` and asserts the canary is absent and the length
  or count is there; where the spec gives the whole expected string, it asserts that string.
  Formatting in a test of a crate whose `clippy.toml` bans `format!` uses
  `alloc::fmt::format(format_args!("{value:?}"))`, as
  `skein/crates/skein-lib/src/tests/crypto_rng.rs` does, or a scoped
  `#[expect(clippy::disallowed_macros, reason = "...")]` on the test, as
  `temper/crates/temper-service/src/tests/worker_parts.rs` does.
- **Design and status files:** a task edits exactly the design documents and status files its
  spec lists, in the way it lists, and no other. This departs from the implementer's text ("Not
  yours: any `docs/design/`") only where a spec says so. Drafts and `docs/help/` stay untouched.
  A status file's form is `DESIGN.md`, 2: its date line is today's (`date -u +%F`), it holds the
  gaps as they stand after your change, never history.
- **Design text says what is true now,** in the design's words: no "no longer", "was",
  "removed", "now" for a change.
- **Old-form counts only fall.** Run `python3 -I -B scripts/old_forms.py` from your tree's root
  before you push; if your change lowers a count, run it with `--write` and commit the lowered
  `scripts/old-forms.txt`; if a count would rise, fix it before you push.
- **Citations:** before pushing a change to Markdown, run
  `python3 -I -B tools/citations/citations.py --base origin/main` from the repository root and
  fix what it lists in the files you own.
- **No secret in a test or a log:** a canary is made up; no test reads a real key, token or
  login, and no test prints the canary on success.
- One commit for the code and its test, saying why; one for any design or status text. Push
  your branch.

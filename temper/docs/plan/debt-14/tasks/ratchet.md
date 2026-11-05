+++
id = "ratchet"
part = "ratchet"
kind = "feature"
needs = []
files = ["hearth/scripts/old_forms.py", "hearth/scripts/old-forms.txt", "hearth/crates/hearth-protocol-config/src/document.rs", "hearth/docs/development/debt/D011-ratchet-counts-two-forms.md", "hearth/docs/development/debt/D001-room-checks-and-max-out.md"]
effort = "medium"
minutes = 30
+++

# hearth's ratchet counts what the others count

## Goal

`hearth/scripts/old_forms.py` counts two forms, so nothing stops new copies of what the rest of this plan pays off.
It takes skein's and smith's forms, each with its baseline, and exempts `io::MAX_OUT_*`, the kernel's edge, as jig
and temper do. Step code's one `while` becomes a `for`, so that form's baseline is 0.

Acceptance:

- `(cd hearth && python3 -I -B scripts/old_forms.py)` exits 0 and prints nothing.
- `hearth/scripts/old-forms.txt` has these rows, in this order. At this plan's base they read: `max_out 77`,
  `worst_case 132`, `io_resume 1`, `earliest_helper 1`, `hex_decimal_fn 15`, `host_forwarder 1`, `path_attribute 25`,
  `domain_while 0`, `comment:actual 2`, `comment:genuine 0`. The trunk may have moved them: each count is what the
  script reads on your branch, and `max_out` falls by exactly the eight `io::MAX_OUT_*` uses.
- `git grep -nE '\bwhile\b' -- 'hearth/crates/hearth-protocol-config/src/document.rs'` finds nothing.
- `heavy cargo nextest run -p hearth-protocol-config` and `-p hearth-config-world` pass, with their counts in your
  report.
- `hearth/docs/development/debt/D011-ratchet-counts-two-forms.md` is gone.
- Each new form bites (Steps, 5), reported.

## Inputs

- `temper/docs/plan/debt-14/notes/common.md`
- `hearth/docs/development/debt/D011-ratchet-counts-two-forms.md`
- `skein/docs/design/draft/component-kit.md`, 4
- `tools/old_forms/README.md`
- `skein/scripts/old_forms.py`, `smith/scripts/old_forms.py` and `jig/scripts/old_forms.py`: the forms you copy

## Owns and may change

- **Owns:** `hearth/scripts/old_forms.py`, `hearth/scripts/old-forms.txt`, `refuse_list_null` in
  `hearth/crates/hearth-protocol-config/src/document.rs`, and the deletion of D011's file.
- **May change:** `hearth/docs/development/debt/D001-room-checks-and-max-out.md`: the **Measure** line's count, and
  the sentence "Eight of the 85 ... (D011).", which becomes one saying the count leaves out the eight `io::MAX_OUT_*`
  uses, the kernel's edge. Nothing else of that file.
- **Not to touch:** any `MAX_OUT` form or `io::resume` call: this task counts them. No form of `tools/old_forms/`: the
  engine has what you need. Don't exempt the `fn earlier` of `hearth-fake-machine`: it stays counted at its baseline.
- **Options:**
  - If another `while` turns up in step code on your branch, it is a new arrival from the trunk: convert it the same
    way if it is as small, otherwise set the baseline to what you find and say so in the report.
  - If clippy refuses `1 << 33` in the constant, write the literal.

## Steps

1. `scripts/old_forms.py`. Add to `FORMS`, after `worst_case`, in this order:

   ```python
   # The component kit's counts (skein/docs/design/draft/component-kit.md, 4).
   'io_resume': re.compile(r'\b(?:skein_)?io::resume\b'),
   'earliest_helper': re.compile(r'\bfn\s+(?:earliest|earlier)\b'),
   'hex_decimal_fn': re.compile(r'\bfn\s+(?:hex|decimal)\b'),
   'host_forwarder': re.compile(r'\bimpl\s+(?:\w+::)*Host\s+for\s+Process\b'),
   'path_attribute': re.compile(r'#\s*!?\s*\[\s*path\b'),
   'domain_while': ENGINE['domain_keyword']('while'),
   ```

   Add `MARKERS` for the first four as skein has them (`'io::resume'`; `'earliest', 'earlier'`; `'hex', 'decimal'`;
   `'Process'`), and:

   ```python
   WORDS = ('max_out', 'worst_case')
   EXEMPT = {
       'max_out': {
           # The kernel's submission bounds stay at its edges.
           name: ('io', 'skein_io')
           for name in ('MAX_OUT_DOWN', 'MAX_OUT_UP', 'MAX_OUT_RESUME', 'MAX_OUT_FIRE')
       },
   }
   # Syntax forms mask strings: a panic's message saying "while" is not a loop.
   SYNTAX_FORMS = ('path_attribute', 'domain_while')
   # Every crate under crates/ is step code but the binary's shell (hearth/AGENTS.md).
   INCLUDE = {'domain_while': re.compile(r'^crates/hearth-[^/]+/')}
   ```

   `HEADER` is a tuple, and the engine calls `splitlines` on it when the baseline is missing: make it one string with
   a newline, as jig's is. `HEADER` and `RULE` add "and component-kit.md, 4", and `RULE` names the shared items, as
   skein's does. Keep the path form hearth's text has.
2. `refuse_list_null`: the loop becomes a `for` over the bound its comment already names.

   ```rust
   /// Steps of the walk: the decoder's length is a u32, each node takes a byte of it, and each is read once and
   /// closed once.
   const WALK_STEPS: u64 = 1 << 33;

   for _ in 0..WALK_STEPS {
       let Some(index) = pending.len().checked_sub(1) else { break };
       // the body as it is
   }
   ```

   The `expect("nonempty stack")` goes with the condition. Nothing else in the function changes: `continue`, the
   doubling and `first` stay. This is the one change to Rust.
3. `scripts/old-forms.txt`: `--write` accepts only rows that are a prefix of the configured order, and new forms sit
   before the comment rows. So add the six rows by hand after `worst_case`, under a line
   `# The component kit's counts (skein/docs/design/draft/component-kit.md, 4).`, each at the count the script gives,
   update the two header lines to the new `HEADER`, then run `python3 -I -B scripts/old_forms.py --write` from
   `hearth/`. It lowers `max_out`.
4. `D001`, as above. Delete D011's file. Check `python3 -I -B tools/debt/debt.py hearth` exits 0.
5. **Bite.** For each of the six new forms, add one use in a throwaway edit to a tracked Rust file where the form
   counts, run the script, see it exit 1 naming that form, and restore. Use a step crate's `src/` for `domain_while`.
   Then show the two scopes hold: a `while` in `crates/hearth/src/`, or in a step crate's `src/tests/`, raises nothing;
   a `"while"` in a string raises nothing; a bare `MAX_OUT_UP` raises `max_out`, and `io::MAX_OUT_UP` does not.
   Report each.
6. Narrow tests for `document.rs`, `cargo fmt`, the ratchet, clippy on `hearth-protocol-config`, the world's tests.
   Commit and push.

## Report

The rows as they read after `--write`. The bites and what each printed. Whether `WALK_STEPS` kept its form. Net lines.

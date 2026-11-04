+++
id = "old-forms-wording"
part = "old-forms"
kind = "docs"
needs = []
files = ["skein/scripts/old-forms.txt", "smith/scripts/old-forms.txt", "jig/scripts/old-forms.txt", "temper/scripts/old-forms.txt", "skein/scripts/old_forms.py", "smith/scripts/old_forms.py", "jig/scripts/old_forms.py", "temper/scripts/old_forms.py", "smith/scripts/test_old_forms.py", "jig/scripts/test_old_forms.py", "temper/scripts/test_old_forms.py", "tools/old_forms/README.md", "skein/docs/foundation/testing-strategy.md", "jig/docs/design/domain/testing.md"]
effort = "medium"
minutes = 20
+++
# The old-forms counts and the budget rule, without the lab's roles

## Goal

The old-forms baselines, the header their scripts write and those scripts' tests say "only the
director raises one"; `tools/old_forms/README.md` speaks of "the queue" and prefixes its
commands with the lab's build tool; two design texts send measurements "through the director"
or speak of "a lead's run". Say each in fabrig's own words. Wording only: no count, form or
script behaviour changes; the tests change only their expected header and fixture text.

Acceptance:

- Each line of step 1 reads as given; `git diff` changes comments, header strings, fixture
  strings and prose only, and no count line of any `old-forms.txt`.
- `rg -n -i '\bdirector\b|\blead\b'` over the files in `files` finds nothing in the lines you
  changed (`DIRECTORIES`, `directory` and an ordinary "lead" elsewhere are not the lab's: leave
  them).
- `python3 -I -B scripts/old_forms.py`, from each of the `skein`, `smith`, `jig` and `temper`
  roots, passes with no count moved.
- `tools/old_forms/test_old_forms.py` and the `smith`, `jig` and `temper`
  `scripts/test_old_forms.py` suites pass.
- After your last commit, `python3 -I -B tools/citations/citations.py --all` reads 0.

## Inputs

- `temper/docs/plan/own-rules/notes/common.md`: the rules every task here keeps.
- `temper/docs/plan/own-rules/inputs/request.md`, items 3 and 4: the request.
- `tools/old_forms/README.md`: how the counter reads a baseline (comments, `# Raised:`
  included, survive a rewrite); `tools/old_forms/old_forms.py`, `main`: `HEADER` only seeds a
  missing baseline, so the baselines' line 2 and the scripts' `HEADER` change apart.
- `skein/docs/foundation/testing-strategy.md`, 8.1: the rule whose wording changes.

## Owns and may change

- Owns the lines step 1 names in the files of `files`, and no other line of them.
- **Departure from your implementer's text** (`notes/common.md`): you edit two sentences of
  `skein/docs/foundation/testing-strategy.md`, 8.1 and one of `jig/docs/design/domain/testing.md`,
  as given. No Rust changes, so no cargo build, clippy or nextest.
- Not yours: the root `AGENTS.md` and `DESIGN.md` (`agents-rules`, in parallel);
  `tests/boundary/outward.txt` (none of your files is listed there, and none of your lines adds a
  name it counts); `skein` has no `scripts/test_old_forms.py`: none to write.
- Options, pre-approved:
  - **A count line of an `old-forms.txt` differs at your base:** leave it; change line 2 only.
  - **`rg -i '\bdirector\b'` finds another line of your files:** reword it the same way and
    list it.
  - **Rewrapping:** `notes/common.md`.

## Steps

1. Reword, each as given:
   - **Each tree's `scripts/old-forms.txt`, line 2** (skein, smith, jig, temper) becomes:

     ```text
     # Lower them with `python3 -I -B scripts/old_forms.py --write`; a count rises only with a `# Raised:` line saying who and why.
     ```

   - **Each tree's `scripts/old_forms.py`, `HEADER`'s last string** (skein ~l.77, smith ~l.30,
     jig and temper ~l.26), `'only the director raises one, recorded in a `# Raised:` line.\n'`,
     becomes `'a count rises only with a `# Raised:` line saying who and why.\n'`. The string
     before it (`'# Lower them with ...; '`) stays.
   - **`jig/scripts/test_old_forms.py` and `temper/scripts/test_old_forms.py`:**
     - line 16, `HEADER`'s copy, becomes
       `"a count rises only with a `# Raised:` line saying who and why.\n"`;
     - line 147's fixture `"# Raised: max_out +1, director's recorded exception\nmax_out 0\n"`
       becomes `"# Raised: max_out +1, by the user, for the fixture\nmax_out 0\n"`;
     - line 149's `"# Raised: worst_case +1, director's recorded exception\nworst_case 2\n"`
       becomes `"# Raised: worst_case +1, by the user, for the fixture\nworst_case 2\n"`.
   - **`smith/scripts/test_old_forms.py`, line 65:**
     `"# Raised: worst_case by director for the fixture\n"` becomes
     `"# Raised: worst_case +1, by the user, for the fixture\n"`.
   - **`tools/old_forms/README.md`:**
     - lines 4-5, "The queue command is unchanged, including Python's isolated mode:", become
       "A landing check runs it from each tree's root, in Python's isolated mode:" (rewrap the
       paragraph of lines 3-5);
     - lines 64-67: each command loses its `heavy ` prefix
       (`python3 -I -B tools/old_forms/test_old_forms.py`, and so on).
   - **`skein/docs/foundation/testing-strategy.md`, 8.1:**
     - lines 617-619, "... through `tools/tiers/run`, or in a person's or a lead's run. Slow
       tiers never run in the gate.", become "... through `tools/tiers/run`, or in anyone's own
       run. Slow tiers never run in the gate." (rewrap those lines);
     - lines 649-650, "Take the measurements to the user through the director; the user
       decides.", become "Take the measurements to the user; the user decides."
   - **`jig/docs/design/domain/testing.md`, lines 376-378:** "taking the measurements to the
     user through the director if neither placement nor an efficiency fix settles it" becomes
     "taking the measurements to the user if neither placement nor an efficiency fix settles
     it" (rewrap those lines).

2. **Checks,** once, after the edits (`notes/common.md`), one command at a time:
   - `python3 -I -B scripts/old_forms.py` from the `skein`, `smith`, `jig` and `temper` roots;
   - `heavy python3 -I -B tools/old_forms/test_old_forms.py`, then
     `heavy python3 -I -B smith/scripts/test_old_forms.py`,
     `heavy python3 -I -B jig/scripts/test_old_forms.py` and
     `heavy python3 -I -B temper/scripts/test_old_forms.py`, from the slot's root (the lab
     runs every test through heavy; the README's commands are fabrig's own and name no tool);
   - commit, then `python3 -I -B tools/citations/citations.py --all` (0).

3. Commit (say why) and push your branch.

## Report

As your implementer's text and `notes/common.md` say, and each suite's test count.

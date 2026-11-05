+++
id = "verbs-close"
part = "world-verbs"
kind = "feature"
needs = ["core-verbs", "accounts-names", "people-names", "connector-names", "const-sweep"]
files = ["jig/tests/core/tests/suite/stories.rs", "jig/tests/core/tests/stories/note_room.rs", "jig/tests/notes/**", "jig/docs/development/debt/D035-bare-numbers-in-worlds.md"]
effort = "medium"
minutes = 20
+++

# The held files follow, and D035 goes

## Goal

Jig D035's last. `stories.rs`, `note_room.rs` and `jig/tests/notes/**` were other drafts', so they come last, touched as little as this allows: their local `ask` and `PROJECT` copies go, and nothing else.

Acceptance:

- `stories.rs`' `ask` and `ask_as` go for the kit's, if nothing reads a token by its number; otherwise its wrapper stays, as in `core-verbs`. `ask(&mut world, n, ..)`'s calls don't change.
- `note_room.rs` and `jig/tests/notes/**` import their `PROJECT`, and the Measure's three hits in `stories.rs` read a name, where that's a line or two.
- **The target.** This command reads at most ten, with each in the report and why its number is the point:

  ```sh
  rg -o -t rust '\b[a-z_]+\((&mut [a-z_]+,\s*)?[0-9]{2,}(_[0-9]+)?,\s*[0-9]+' jig/tests jig/testing \
    | rg -v ':(millis|between|randomness|check_redial|assert_replays|fixture|workers|workers_at|procedure_watch|procedure_world|reserve_fixture|world_with_rows)\(' | wc -l
  ```

  A span, a bound or a seed names itself: those twelve functions are exempt, and that's the item's own clause. The item's unfiltered Measure read 262 at this plan's base, with `jig/testing` not yet holding what moved there: report it too.
- `jig/docs/development/debt/D035-bare-numbers-in-worlds.md` is deleted. `python3 -I -B tools/citations/citations.py --all` reads 0, and `python3 -I -B tools/debt/debt.py | rg 'D035'` shows no jig row.
- `heavy cargo nextest run -p jig-core-world -p jig-notes-world` passes.

## Inputs

- `temper/docs/plan/debt-24/notes/common.md`
- `jig/docs/development/debt/D035-bare-numbers-in-worlds.md`

## Owns and may change

- **Owns:** the files in the header.
- **If the target can't be met** by a line or two in those files: a world's rows beyond them are `const-sweep`'s and the others'. Report each as left, and **keep the item if more than ten stand,** with the command in its **Measure** and what's left in its **What**. That's a `Done:`, and fabrig/debt decides.

## Steps

The copies; the search; the deletion and checks. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-core-world -p jig-notes-world --all-targets -- -D warnings`. Commit and push.

## Report

The common note's items; both counts; each number left and why; across D035's tasks, the test lines at this plan's base and now.

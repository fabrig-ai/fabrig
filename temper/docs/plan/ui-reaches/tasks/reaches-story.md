+++
id = "reaches-story"
part = "client"
kind = "feature"
needs = ["decide-root", "stop-asking"]
files = [
  "temper/tests/client/tests/suite/away.rs",
  "temper/tests/client/src/front/away.rs",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 100
+++

# The client world's story of what reaches a person

## Goal

One story in the client world (`temper/tests/client`, `temper-client-world`) walks what N1 delivers end to end through both roots, as the draft's acceptance has it (`temper/docs/design/draft/what-reaches-you.md`, section 10, the v0.1.0 story's second and third steps, and the `money` wait): on Agents review's table, with Away off, an implementation question asks the person and its card says why; with Away on, the next is decided and listed under *Decided for you* on *Needs you*; a `money` question waits though Away is on; Stop asking on the first need, applied by its owner, and the next implementation question is decided and listed with Away off (`temper/docs/design/client/README.md`, section 10, `tests/client`'s stories).

Acceptance:

1. **One new story function** in `temper/tests/client/tests/suite/away.rs`, beside `a_goal_decides_while_away_then_waits_on_home_after_overturn_and_off`, named for what it shows (`what_reaches_a_person_follows_the_classes_away_and_stop_asking`), reusing that file's `Story` and its helpers (`sequence`, `begin_goal`, `call`, `press`, `sees`, `decisions`), helpers added beside them where it needs one (a `needs` visit to `Address::Inbox` filtered by `PROJECT`, a check that a text is absent). It runs on the fixture's stock classes: the project holds no kind 8 value, so Agents review's table (`implementation` at `Ask`) judges.
2. **Its steps,** each asserting through the person's view (`sees`) and the engine's rows (`decisions`, the store's `Decided` rows by their reach), with the tool's answers checked by their first word alone (`"Ask:"`, `"Decided:"`), since `decide-root` words the rest:
   1. Home visited (To begin skipped), a goal begun; the switch shows by the classes (the rail's "Away"); with it off, a `decided` call of kind `implementation` is answered ask and writes an asked row; the goal's `message` question to its person (`"form":"question"`) shows on *Needs you* with "Reached you because it is an implementation choice, which asks you unless you’re away." and "Stop asking".
   2. Away on (the rail's "Away" pressed); the next `implementation` call is answered decided; *Needs you* shows "Decided for you · 1" and "Allowed: implementation choices are decided while you’re away."; the first question still waits.
   3. Away still on, a `decided` call of kind `money` is answered ask; its question's card says "Reached you because it spends money past a budget, which always waits for you." and offers no "Stop asking".
   4. Away off; on the first question's card "Stop asking" pressed, then "Apply" on "Let agents decide implementation choices, then tell you."; the engine's kind 8 of the project now holds `implementation` at `Decide` (read through the store's setting rows or the forge's `project_classes`), the toast "Applied: the next one is decided and listed. This one still waits for you." seen, and the first question still on *Needs you*.
   5. The next `implementation` call is answered decided with Away off; *Needs you* shows "Decided for you · 2" and "Allowed: implementation choices are decided and listed."
3. **The settings watch:** `Story::start_client` unwatches the `settings` handle to keep the engine's watch table for Away and Home; this story needs it (Apply takes the newest revision from it), so its `Story` keeps it, by a constructor or a flag beside `sequence`'s, the other stories unchanged.
4. **Runs,** counts and the story's own time (wall and CPU, from nextest's line) in the report: `heavy cargo nextest run -p temper-client-world --test suite -E 'test(=what_reaches_a_person_follows_the_classes_away_and_stop_asking)'`, then `heavy cargo nextest run -p temper-client-world`; clippy on `temper-client-world --all-targets`.
5. **Status:** delete the **Not yet** entry naming `ui-reaches/reaches-story` in `temper/docs/design/client/README.status.md`.

## Inputs

- `temper/docs/plan/ui-reaches/notes/common.md`, all of it, *Tests stay cheap* above all.
- `temper/docs/plan/ui-reaches/notes/contracts.md`, section 6 (the tool's input: `kind`, `paths?`, `under?`).
- `temper/docs/design/draft/what-reaches-you.md`, section 10.
- `temper/docs/design/client/README.md`, 3.5: *Away* (*Shown*), **Decided for you** (*The region*, *What allowed it*), *Needs you* (*Why it reached you*, *Stop asking*); section 10, `tests/client`.
- `temper/docs/design/domain/forge.md`, 5.9 (asked and decided rows) and 5.10 (Agents review's stock table; `money` fixed).
- The code: `temper/tests/client/tests/suite/away.rs` (its `Story`, `sequence`, `start_client`, `call`, and the sibling story named above), `temper/tests/client/src/front/away.rs` (`with_away_sequence`).

## Owns and may change

Owns the story and the helpers it adds, and the status entry. Not the client's or the engine's crates: a story that finds a fault there stops and reports it (`## Left`, or a `Question:` where it blocks the story), never repairs it here.

May change, each only as said:

- `temper/tests/client/src/front/away.rs`: the fixture's bounds this story needs (call records, inbox messages, people's requests, the client's watches and handles), each with a comment saying which step holds it, the other stories' bounds unchanged.

Options, each decided:

- **A watch or a read is refused for room** (the engine's per-session table or the client's): raise that bound in the story's fixture alone, as `sequence` does today, and say which and why.
- **The goal's second question** needs the first answered by the world's bounds (its inbox): answer nothing; raise `inbox_messages` for this story's fixture, as above. The first need must still wait at step 4's end.
- **`decide-root`'s answer words** differ from the steps' first words: assert the first word the tool answers for ask and for decided, whatever follows, and name them.
- **The story costs more than the client world's other Away stories together:** say so with its time; do not split it into several functions to hide it.

## Steps

1. Read the Inputs and the sibling story. Write the steps (acceptance 1, 2) and the watch's flag (acceptance 3).
2. Run it alone (acceptance 4's first run); a failure is read from its log first. Bite: comment out step 4's "Apply" press and see step 5 fail on "Decided:" (answered ask), restore.
3. The rest of acceptance 4; the status entry; `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; commit; `python3 -I -B tools/citations/citations.py --base origin/main`; push with `git push origin HEAD:refs/heads/<your branch>`.

## Report

The implementer's report: the story's steps as written, the fixture's bounds changed and why, the bite, the options taken, the story's time, and the runs' counts.

# Plainer panels: three small gaps a person meets in `task show` and the help

QA's walks found three places where `temper ctl` tells a person less than it knows. Each is small. They share a help page.

## What to change

1. **An answered person task says who answered.** `task show` on a person task that has been answered prints `taken by: none`. It should name who answered, or say nothing of it. The line is in `temper/crates/temper-client-tools/src/jig/render/document.rs`.
2. **A held task says it's held.** `task show` on a held change prints `waiting on: Nothing awaited` beside its hold, which reads as if nothing were wrong. It should say what it's waiting on: its hold. The word comes from `jig/crates/jig-client/src/words/english.rs`.
3. **The help says so.** `temper help task` says what `taken by:` and `waiting on:` print, including both cases above. `temper/docs/help/task.md` is near its size bound, so make room there. Do this with the two above, or after them, as you see fit.

## Left open

Each coding change that reaches one of these asks through its change's questions, and doesn't choose.

- **The word for a hold** (item 2): `held` alone, or `held: REASON`, which may be long on a narrow screen. The person who wrote this draft decides.

## How

- Each has a test at the lowest tier that shows it, as `task show`'s tests do today.
- Nothing else changes.

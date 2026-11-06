+++
id = "repeats"
part = "repeats"
kind = "feature"
needs = ["figures-before", "dev-profiles", "git-setup"]
files = ["hearth/docs/plan/hearth-speed/figures.md", "hearth/testing/**", "hearth/tests/*/src/**", "hearth/crates/**", "Cargo.toml"]
effort = "high"
minutes = 60
+++

# What the profile says still repeats

## Goal

`dev-profiles` and `git-setup` may already have met the aim. If they have, record that and change no code. If not, repair what `figures.md`'s profile shows, largest share first, and stop at the aim.

Acceptance:

- `figures.md` gains a "Repairs" section: the method's two figures at the base commit and at the trunk you start from, on one machine, and the ratio. That's your first commit, whichever way it comes out.
- **If the ratio is at most 85%:** you're done. No other file changes, and your report's first line says so.
- **If not:** one commit per repair, each with what it saved, measured, until the ratio is at most 85% or the options below run out. The "Repairs" section gets the final figures and what's left, by how much.
- Every test of `hearth/`'s packages passes in `default` and `fuzzy`, with every assertion, case, seed and count as it was. A golden, `fixtures::CUTS` and a recorded digest don't change: a change to one is a finding, not a new figure.

## Inputs

- `hearth/docs/plan/hearth-speed/notes/common.md`
- `hearth/docs/plan/hearth-speed/figures.md`: "What it found" and "Profile"
- `hearth/docs/plan/hearth-speed/inputs/request.md`, "Where it goes"
- `hearth/docs/design/domain/testing.md`, 9.1
- `skein/docs/foundation/programming-model.md`, 10, for anything you change in a step crate

## Owns and may change

- **Owns:** `figures.md`'s "Repairs" section, `hearth/testing/**`, `hearth/tests/*/src/**`, and the `[profile.dev.package.*]` tables of the root `Cargo.toml`.
- **May change, for the repair alone:** a hearth step crate under `hearth/crates/`. `debt-26/hearth-spot` changes `hearth-protocol-builder/src/lib.rs`, `keep.rs` and `read.rs` beside this plan, so keep any edit there to its own hunk.
- **Test files under `hearth/tests/*/tests/` and `src/tests/`:** helpers only, to compute once what they compute again from the same input. Every `assert` stays, on the same values.
- **Departs from your text:** a run that finds the aim met pushes only the "Repairs" section, and adds no test.

The options, each pre-approved, in the order the profile's shares put them:

1. **Work a world repeats, with the same result each time.** A value that doesn't depend on the seed is computed once in the test's process, and each seed gets a fresh copy of it, so no state crosses seeds and a seed still replays. What a seed draws stays per seed: `hearth/tests/builder/src/fixtures/worlds.rs`'s `Shared::build` lays the fake repository from the scenario alone, but `Universe::draw` and `Machine::new` take the seed. If a fake lacks `Clone`, adding it is in scope. Copying must cost less than building, measured.
2. **Costly code in a kit or an oracle:** the same result, computed more cheaply. An oracle stays independent of the crate's codec, as `hearth/tests/tree/src/oracle.rs` says it is: no shared code to save time.
3. **Another profile line,** if the profile shows a hearth crate or a pure dependency that `dev-profiles` left out, on the same terms as that task's: `opt-level` alone, saving and build cost in the commit.
4. **A real inefficiency in hearth's own code:** a repeated copy, a quadratic scan, a parse done twice. What hearth does is unchanged: the same answers, bytes and records, and the same refusals at the edges. The step crates' rules hold.
5. **The scratch repository without `git add` and `git commit`,** only if `figures.md` and your own figures show at least 0.5 CPU s still there. `Repository::make` writes the blobs, trees, commit and ref itself, as loose objects (`sha1`, added to `hearth-test-repository`'s `Cargo.toml`, and a zlib stored block written in the kit), with the same commit ids as today. `make_on` keeps running real git, because `a_scratch_repository_without_git_on_the_path_fails_by_name_not_by_skip` holds it. `next` and `git` stay real git. One new test in `hearth/tests/builder/tests/real/` makes one repository each way and compares `git rev-parse HEAD`, and runs `git fsck --strict` on the written one. Break the writer's tree order by hand to see it bite.

**Not options:** `RUSTFLAGS`, linker settings or anything else global; `.config/nextest.toml`; a line for a crate of another tree (`notes/common.md`). A process's floor is left as it is unless the profile shows hearth's own code in it. What these would save goes under `## Left`.

## Steps

1. The method's figures at the base commit and at your start, on your machine. Commit the "Repairs" section. Met: push, report, stop.
2. Otherwise, the largest share first: the repair, its family's figure, then the method's figure. Keep it if it saves, with a commit that says what.
3. Stop at the aim or at the end of the options. The full `default` and `fuzzy` runs of `hearth/`'s packages. The end of every task in `notes/common.md`.
4. Push.

## Report

The ratio at your start. Each repair and its saving, or none. The final figures, the machine, what's left by how much, and `## Left`.

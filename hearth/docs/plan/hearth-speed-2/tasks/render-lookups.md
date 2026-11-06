+++
id = "render-lookups"
part = "render"
kind = "feature"
needs = ["profile"]
files = ["hearth/crates/hearth-domain-render/src/revision.rs", "hearth/crates/hearth-domain-render/src/call/references.rs", "hearth/crates/hearth-domain-render/src/schema.rs", "hearth/crates/hearth-domain-render/src/tests/revision.rs"]
effort = "high"
minutes = 30
+++

# A render finds a document without classifying them all

## Goal

`Documents::declaration` runs `class()` on each document until one matches, and tasks sort last, so it walks nearly all of them, for about eight calls per task per target. It's the leading linear cost in render's production path at the worlds' sizes (by reading, about 5,000 `class()` calls per render of `Shape::SMALL`, and a property test does 12-24 renders). This is a real inefficiency in what a user runs, so it's repaired in production, with every answer the same.

Acceptance:

- `Documents::declaration` and `Documents::find` search by bisection, with the same signatures and the same answers.
- `references.rs` tests `entry.oid != oid` before it calls `class()`.
- `schema.rs` builds an enum's `names` list only when the value isn't found.
- **A new table test in `hearth/crates/hearth-domain-render/src/tests/revision.rs`:** over a listing with a fleet, an environment, a host and tasks that sort around each other (`a`, `a1`, `a_b`, `b`), `declaration` finds what a scan finds, for those names and for names that can't be a task (`A`, `a-b`, `a/b`, the empty name). `find` likewise, for a path present and one absent.
- Every test of `hearth-domain-render` and its readers passes unchanged, and every golden stands.

## Inputs

- `hearth/docs/plan/hearth-speed-2/notes/common.md` and `hearth/docs/plan/hearth-speed-2/notes/repairs.md`
- `hearth/docs/plan/hearth-speed-2/figures.md`: `### Render suite` and `## What it found`
- `hearth/crates/hearth-domain-render/src/revision.rs`: `class` and `identifier`, for what a task's name may be
- `skein/docs/foundation/programming-model.md`, 10: this is a step crate

## Owns and may change

- **Owns:** the files above. No public signature changes.
- **Why the bisection holds, which its comment says:** `Documents::new` keeps entries in a `Map` by path, so they're sorted and unique. Under `tasks/` it keeps only `Class::Declaration`, and `class` gives that only to a name `identifier` accepts: lowercase, digits and `_`, all above `/`. So path order is task order there. A name that can't be a task compares as any other and finds nothing, as before.
- **Don't bisect `Side`'s entries,** which are another list: `fixtures::x1_prod` lists them out of order. `listed_entry` and the rest of `call/` stay scans here.
- **Other costs the survey named in `hearth-domain-render`** (the exports cache's `prime`, the per-task listing scans in `call/files.rs`, `retain_spec`'s re-walk) are `last-round`'s options, because they're larger and matter less at the worlds' sizes. Leave them, and leave any other file of this crate.

## Steps

The sketch binds what's searched; the body is yours, under the step rules:

```rust
pub fn declaration(&self, name: &[u8]) -> Option<(u32, Option<&declaration::Declaration>)> {
    let first = self.entries.partition_point(|entry| entry.path.as_ref() < b"tasks/".as_slice());
    let at = self.entries.get(first..)?.binary_search_by(|entry| task(&entry.path).unwrap_or(b"").cmp(name)).ok()?;
    let index = first.checked_add(at).expect("bounded listing");
    Some((u32::try_from(index).expect("bounded listing"), self.declared.get(index)?.as_ref()))
}
```

1. The start check. Then `Ir` of the costliest test `figures.md` samples from `hearth-render-world`, and that binary's figure, twice.
2. The repairs, one commit each where each measures a saving. The table test. Break it by hand once: bisect by path without the `tasks/` partition, or by a comparison that treats `_` below `/`; see it fail; restore.
3. The same figures after.
4. **The readers' tests:** `heavy cargo nextest run -p hearth-domain-render -p hearth-render-world -p hearth-builder-world -p hearth-service-builder -p hearth-protocol-builder -p hearth`, then `--profile fuzzy -p hearth-render-world -p hearth-builder-world`: the large render reads this path, so give its figure before and after in the report.
5. The end of `notes/common.md`. Push.

## Report

What each repair saved, the large render's figure on both sides, the figures, the machine, and `## Left`.

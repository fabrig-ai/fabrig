+++
id = "decode-index"
part = "decode-index"
kind = "feature"
needs = []
files = ["skein/testing/skein-browser/src/wire/decode.rs", "skein/docs/design/browser.status.md"]
effort = "medium"
minutes = 120
+++

# CDP replies parsed once: the decoder looks fields up in an index

## Goal

`skein/testing/skein-browser/src/wire/decode.rs` validates each CDP message in `Document::parse`. Then `Value::get` and `Array::next` parse again every subtree they pass, and `get` unescapes every key it compares into a new allocation. A field three levels down in a 40 KB accessibility tree walks most of it three times. Index every value's span and every member's key once, while `parse` validates. Then make `get`, `array`, `Array::next` and `raw` lookups in that index. The module's public surface and what each call returns stay as they are, so no reader changes.

Acceptance:

1. `Document`, `Value`, `Array` and `DecodeError` keep their names, their public functions' signatures and their derives. Each call returns what it returns on the trunk for every input: `parse`'s errors and their order, `get`'s first-of-duplicates, `raw`'s exact bytes, `text`'s unescaping, the integers' and `whole_pixels`' rules.
2. `parse` builds the index as it validates: one entry per value and per object member's key, in document order, each value's entry giving where its subtree ends. It counts the entries first and allocates them once, at that count. `get`, `array`, `Array::next` and `raw` parse nothing; they read the index. A key without a backslash is compared as its bytes, with no allocation.
3. New unit tests in the module's `tests` cover the lookups (Steps, 3). The three tests there keep their assertions as they are.
4. `skein/tests/browser/tests/suite/memory.rs`'s ceilings stay as they are, and its tests pass. No assertion anywhere is removed or loosened (the request, 3).
5. The focused tests of `skein-browser`, `skein-browser-world` and `skein-fake-person-browser`, and `skein-browser-world`'s fuzzy profile, pass. `skein-ui-web-tests`, `temper-client-browser-world` and `temper-client-journeys` compile with their tests (Steps, 4).
6. The story `browser_pages::a_chat_opens_at_its_end_and_reads_back_keeping_its_place` passes, run once by name, alone, on the decoder against Chromium (Steps, 5).
7. `skein/docs/design/browser.status.md` no longer has the **Not yet** line on 4 and 8.

## Inputs

- The request: `temper/docs/plan/browser-reading/inputs/request.md`, read whole (it is short). Its `For every task of this plan` binds this task and stands in for a `notes/common.md`. Where it and this spec differ, it wins.
- `skein/docs/design/browser.md`, 4, the bullets *Decoding keeps only what it needs* and *A message is parsed once* (this cut wrote the second), and 8, the index's memory (this cut wrote that sentence).
- `skein/docs/design/browser.md`, 9, *Step tests* (bounded decoding uses synthetic CDP replies) and the memory test (section 8).
- `skein/docs/foundation/programming-model.md`, 10: the strict subset. `skein-browser` sits under `skein/clippy.toml` (the step variant): no `Vec` but under a scoped `#[expect]`, no indexing, checked arithmetic, no `as`, no trait implemented by hand.

What the cut found. The lead read this and ran nothing.

- **The readers.** `Document::parse` is called once in production, in `Browser::message` (`skein/testing/skein-browser/src/browser.rs`). Every other read goes through `Value::get`, `array`, `text`, `u64`, `i64`, `bool`, `is_null` and `raw`, through the helpers `field_text` to `opt_array`, `parse_seen`, `ax_*`, `subtree_has` and `snapshot` there. Tests call `parse`, `root`, `get` and `raw` in `skein-browser`'s `browser/*_tests.rs` and in `skein/tests/browser/src/tests/{attribute,foreground,navigation,tabs}.rs`. `Value`'s and `Array`'s `PartialEq` and `Debug` are used by no reader.
- **The direct dependents** (`cargo tree -i skein-browser`): `skein-browser-world`, `skein-fake-person-browser`, `skein-ui-web-tests`, `temper-client-browser-world`, `temper-client-journeys`.
- **Memory:** the memory test's scenarios use `message: 4096` and ceilings of a quarter above their measured peaks (`BROWSER_REPORTS_PEAK_CEILING` 21,000 over 16,764 measured, for one). An accessibility node's JSON holds about one value or key per 7 bytes. So an index of 12 bytes an entry is about 1.7 times its message, transient, while the message is handled.

Departures from the implementer's text, and why:

- **You run one browser story, by name and alone, once,** as the request allows: it is the story whose replies this decoder parses, and the plan's only check of the decoder against Chromium. Run only it, never a tier, nor skein's `browser_real` or `browser_facts`. Those are the `browser-skein` tier: compile them. The command is in Steps, 5.
- **You remove one status line** in `skein/docs/design/browser.status.md` (acceptance 7).

## Owns and may change

Owns:

- `skein/testing/skein-browser/src/wire/decode.rs`, the whole file: the index, the lookups, the parser that fills it, its tests.
- `skein/docs/design/browser.status.md`: the **Not yet** line on 4 and 8 alone.

Reader and neighbour files: none should change. If a reader outside `decode.rs` stops compiling, the public surface moved: put it back, rather than edit the reader.

The outer layer, binding. Inside it, the choices are yours:

```rust
/// One complete JSON document, without the trailing NUL, and the index of its values.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Document {
    bytes: Box<[u8]>,
    entries: Box<[Entry]>, // allocated once, at the count validation found, then filled in place
}

/// A value's or a member key's place, in document order. A member is two entries,
/// its key's (a string's) and then its value's.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Entry {
    start: u32, // the first byte
    end: u32,   // one past the last byte
    after: u32, // the first entry past this value's subtree: its next sibling's, or its parent's next
}

/// A valid JSON value of a validated document: an entry of its index.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Value<'a> {
    document: &'a Document,
    at: u32,
}

/// A borrowed iterator through a validated JSON array's entries.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Array<'a> {
    document: &'a Document,
    next: u32, // the next element's entry
    end: u32,  // the array's `after`
}
```

- `Document::root` is entry 0. `Value::raw` is `bytes[start..end]` (by `get`, never by indexing), exact, with no whitespace around it.
- `Value::get(key)`: on an object's entry `i`, walk its members from `i + 1` while below `i`'s `after`. Each member is a key entry `k`, then its value at `k + 1`, and the next member at the value's `after`. Return the first member whose key matches. A key's bytes between its quotes, with no `\`, match by comparison. A key with a `\` is unescaped as `text` does now, then compared.
- `Value::array` on an array's entry gives `Array { next: at + 1, end: after }`. `next` yields the entry at `next` and moves to its `after`.
- Offsets and counts are `u32`: `parse` refuses a message over its `u32` limit already.

Pre-approved options:

- **Filling the index:** the same `Parser`, run twice, counting then filling, or a separate counting pass. Either way, validation's errors and their order stay as they are, and nothing grows. `entries` is a `Box<[Entry]>` of the counted length, made once (for example by collecting `core::iter::repeat_n`), and its entries are set through `get_mut`. `skein_lib::List` doesn't fit: it derives only `Debug`, which would cost `Document` its derives, and it grows as it fills. If a single pass with an exact bound proves simpler, it may be used. The bound must not be the message's length times an entry's size, which would hold 12 bytes per message byte.
- **If a memory ceiling in `skein/tests/browser/tests/suite/memory.rs` breaks:** shrink the entry, never raise the ceiling. For example, a member becomes one entry whose `start` is its key's: its value starts past the key's closing quote, the colon and the space between, found by a scan of that short stretch. Then `Value` carries its value's start, as `Value { document, at, start }`. Give the peaks before and after in the report. If no entry of 12 bytes or less fits, stop and ask, with the measurements.
- **The module's `#![expect(clippy::disallowed_types, ...)]`:** keep it for `text`'s `Vec`. If the index needs another container the step lints refuse, add a scoped `#[expect]` with its reason on that item alone.
- **`Value`'s and `Array`'s derives** now compare and print through the document reference. Keep them derived, since the step variant implements no trait by hand. No reader compares or prints a `Value`.

## Steps

1. **The index.** Rewrite `Document`, `Value`, `Array` and `Parser` to the outer layer above. `text`, `u64`, `i64`, `whole_pixels`, `bool` and `is_null` read `raw` and keep their bodies. `hex4`, `unsigned`, `signed` and the byte classes stay.

2. **The narrowest tests first:** `heavy cargo nextest run -p skein-browser --lib -E 'test(/wire::decode::tests::/)'`, then the whole `skein-browser` lib.

3. **The lookups' unit tests,** in `decode.rs`'s `tests`, beside the three there. One test per rule; your names, in the module's style:
   - a member is found at its own depth and past subtrees it skips: in `{"a":{"b":1,"c":[1,[2,{"b":3}]]},"b":{"x":"y"},"d":4}`, the root's `b` is `{"x":"y"}` and its `d` is 4. The root has no `c`, and `a`'s `c` holds the nested arrays whole;
   - duplicate keys select the first, at the root and nested;
   - an escaped key matches its unescaped bytes: `{"ab":1,"q\"":2,"ab":3}` gives `ab` as 1 and `q"` as 2;
   - an array yields each element whole and then nothing: `[ 1 , "x" , {"k":[2]} , [] , null ]`, each element's `raw` exact. An empty array yields nothing;
   - a lookup on the wrong kind finds nothing: `get` on an array, a string, a number or an empty object, and `array` on an object or a string;
   - `raw` is exact around whitespace: in `{ "a" : [ 1 , 2 ] , "b" : "c" }`, `a`'s `raw` is `[ 1 , 2 ]`;
   - the deepest document `parse` takes parses, and a member past its deepest subtree is found: an object whose `a` holds 63 nested arrays, 64 containers in all, then `"b":1`. One array more is `TooDeep`, as now;
   - the index holds one entry per value and per key: `{"a":[1,2],"b":{}}` holds 7. This pins that `parse` indexes and the lookups read the index.

   For each, break the rule once by hand (return the last duplicate, skip the escape path, stop at the first nested `}`, drop the `after` jump), see the test fail for the rule's reason, and restore.

4. **Check.**
   - `cargo fmt`.
   - `python3 -I -B skein/scripts/old_forms.py`.
   - `heavy cargo clippy -p skein-browser -p skein-browser-world -p skein-fake-person-browser --all-targets -- -D warnings`.
   - `heavy cargo nextest run -p skein-browser -p skein-browser-world -p skein-fake-person-browser`: the focused tests, the memory suite with them.
   - `heavy cargo nextest run -p skein-browser-world --profile fuzzy`.
   - `heavy cargo check -p skein-ui-web-tests -p temper-client-browser-world -p temper-client-journeys --tests`. These are the dependents whose browser tests are slow tiers: they compile and don't run.
   - Remove the **Not yet** line on 4 and 8 from `skein/docs/design/browser.status.md`.
   - Commit, then `python3 -I -B tools/citations/citations.py --base origin/main` (it reads `HEAD`).

5. **The story, once, by name and alone,** through heavy, after step 4 passes:

   ```sh
   TIERS_WRAPPER=heavy tools/tiers/run temper browser -- -E 'test(=browser_pages::a_chat_opens_at_its_end_and_reads_back_keeping_its_place)'
   ```

   Check that it selected exactly one test, and record `date -u` before the run. It reads and compares no CPU figure (the request's Limits). If it fails, read its log around the failing step with `grep` and `tail` before anything reruns. Then run it once on the pristine base (the request's *For every task*). If it fails there too, it isn't yours: name it under `## Left`.

6. **Commit and push.** Use the message `browser-reading: CDP replies indexed once, fields looked up`. Its body says why: each lookup reparsed the subtrees it passed, and unescaped each key it compared.

## Report

Follow the implementer's text. Also give:

- the entry's layout and how the index is filled, and the memory suite's peaks, as it prints them, before and after;
- each new test, the rule you broke by hand and the failure you saw;
- the story's run: its `date -u` time, its outcome, how many tests it selected;
- every file you changed outside `files`, and why.

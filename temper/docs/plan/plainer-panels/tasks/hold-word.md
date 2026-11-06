+++
id = "hold-word"
part = "hold-word"
kind = "interface"
needs = []
files = ["jig/crates/jig-client/src/words.rs", "jig/crates/jig-client/src/words/english.rs", "jig/crates/jig-client/src/tests/words.rs", "jig/docs/design/client/domain/client.status.md", "temper/crates/temper-client-view/src/words.rs"]
effort = "medium"
minutes = 15
+++
# jig's words gain `held`

## Goal

`jig-client`'s words gain `held`, for what a held task waits on when it awaits
nothing else. `held-waiting` prints it. This task prints nothing to anyone.

`held` opens both forms the request leaves open, `held` alone and `held: REASON`,
so this task asks nothing and chooses nothing between them.

Acceptance:

- `Word::Waiting(WaitingWord::Held)` says `held`, as phrase number 124.
- `Word::waiting(&WaitsOn)` is unchanged, and maps no `WaitsOn` to it.
- `temper-client-view`'s counted phrases match, so its tests are green.

## Inputs

- `jig/docs/design/client/domain/client.md`, 19 (`words`).
- `temper/docs/plan/plainer-panels/notes/common.md` and `notes/contracts.md`.

## Owns and may change

Owns:

- `jig/crates/jig-client/src/words.rs`: `WaitingWord`, `ALL`, `PHRASES`.
- `jig/crates/jig-client/src/words/english.rs`: one `data()` arm.
- `jig/crates/jig-client/src/tests/words.rs`: one test.

May change, departing from your run's text:

- `jig/docs/design/client/domain/client.status.md`: remove the `Not yet` line for
  section 19's `held`, and date the file with `date -u`.
- `temper/crates/temper-client-view/src/words.rs`: only the `const PHRASES` and
  `const SEGMENTS` values, by the regeneration below. This is the one line of
  another tree a new word forces: the count `temper-client-view` holds of jig's
  words. If regeneration changes anything else there, that is a `Question:`.

Number 33 stays unused: leave it.

## Steps

```rust
// words.rs
pub enum WaitingWord { Dependencies, Party, Delegates, News, Nothing, Held }
pub const PHRASES: u32 = 124;
pub const ALL: [Word; 124] = [ /* ..., */ Word::Waiting(WaitingWord::Held) /* beside Nothing */ ];

// english.rs, Word::data
Self::Waiting(WaitingWord::Held) => (124, b"held", &[]),
```

Test in `tests/words.rs`, using the file's `book`:

```rust
#[test]
fn a_held_tasks_wait_has_its_own_word() {
    let book = book(Box::new([words::english()]));
    assert_eq!(book.say(0, Word::Waiting(WaitingWord::Held).id(), &[]).unwrap().text.as_ref(), b"held");
    assert_eq!(book.say(0, Word::waiting(&WaitsOn::Nothing).id(), &[]).unwrap().text.as_ref(), b"Nothing awaited");
}
```

Regenerate temper's count, then check it without the switch:

```
SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper-client-view --lib -E 'test(=tests::accounts::english_phrase_declarations_match_the_vocabulary)'
heavy cargo nextest run -p temper-client-view --lib -E 'test(=tests::accounts::english_phrase_declarations_match_the_vocabulary)'
```

The direct dependents your run's text asks for include `jig-client-view`,
`temper-client-domain`, `temper-client-view` and `temper-client-tools`. Commit
and push.

## Report

As your run's text says. Name the `PHRASES` and `SEGMENTS` values you wrote.

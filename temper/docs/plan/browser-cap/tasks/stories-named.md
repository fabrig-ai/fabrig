+++
id = "stories-named"
part = "steps"
kind = "interface"
needs = []
files = ["temper/tests/client-browser/src/story.rs", "temper/tests/client-browser/src/history.rs"]
effort = "medium"
minutes = 25
+++

## Goal

The surface the merged tests build on: a type that names the story a failed step stands in, and the ctl calls that make a second and third chat in one deployment. No browser test changes here.

Acceptance:

- `temper_client_browser_world::Stories` exists with the shape in `temper/docs/plan/browser-cap/notes/contracts.md`.
- `Tier::ctl_next_chat` and `Tier::ctl_messages` exist with the shape there, and `Tier::ctl_chat` keeps its signature, so `browser_pages.rs` compiles untouched.
- `heavy cargo nextest run -p temper-client-browser-world --lib` passes, with the tests sketched below among them.
- `heavy cargo check -p temper-client-browser-world --tests` passes.

## Inputs

- `temper/docs/design/client/browser.md`, 4.3: *A test of several stories*, *A chat's history*.
- `temper/docs/plan/browser-cap/notes/common.md` and `notes/contracts.md`.
- `skein/testing/skein-fake-person-browser/src/harness.rs`, `Outcome`; `harness/script.rs`, how `Script` counts `at`.

## Owns and may change

Owns `temper/tests/client-browser/src/story.rs` (new) and `temper/tests/client-browser/src/history.rs`.

May change `temper/tests/client-browser/src/lib.rs`, these two lines alone: `mod story;` and `pub use story::Stories;`.

`chat_calls` in `history.rs` may be reshaped. Its two existing tests stay and still pass; each may change only in the helper it calls. If `ctl_chat` is then one line over `ctl_next_chat`, good.

## Steps

**`src/story.rs`:** a module comment citing `temper/docs/design/client/browser.md`, 4.3, then the contract's type. It compares a failure's `person` and `step` with what was begun. It holds no state past that, and reads no step's contents: `begin` takes only `steps.len()`.

**Its tests, in that module:**

```rust
#[test]
fn a_step_stands_in_the_last_story_begun_at_or_before_it() {
    let mut stories = Stories::of_person(1);
    let mut steps = vec![Step::Back];
    stories.begin("first", &steps);      // begins at step 1
    steps.extend([Step::Back, Step::Back]);
    stories.begin("second", &steps);     // begins at step 3
    assert_eq!(stories.story(0), None);
    assert_eq!(stories.story(1), Some("first"));
    assert_eq!(stories.story(2), Some("first"));
    assert_eq!(stories.story(3), Some("second"));
    assert_eq!(stories.story(9), Some("second"));
}

#[test]
#[should_panic(expected = "second: person 1, step 3: the report")]
fn a_failure_is_named_by_its_story() { /* assert_done(Outcome::Failed { person: 1, step: 3, report: "the report".into() }) */ }

#[test]
#[should_panic(expected = "person 0, step 3: the report")]
fn another_persons_failure_has_no_story() { /* the same stories, person: 0 */ }

#[test]
fn a_done_run_passes() { /* assert_done(Outcome::Done) */ }
```

**`src/history.rs`:** `ctl_next_chat` and `ctl_messages` as in the contract. Each message's wait reads `task show {task} --until replied`, on its own chat's number.

**Its test,** beside the two there:

```rust
#[test]
fn two_chats_name_their_own_task_and_keep_distinct_keys() {
    // The same words in chats 1 and 2, then two more messages to chat 1: every key is distinct,
    // and each `task message` and `task show` names the chat it was called for.
}
```

Exercise it through the same pure functions the existing tests use (`Path::new("/test/temper")`), so that it needs no scratch deployment.

**Then:** `cargo fmt`; `heavy cargo clippy -p temper-client-browser-world --all-targets -- -D warnings`; the acceptance's two commands; commit and push.

## Report

As the implementer's text has it: the tests that ran and their count, the rule each new test was broken on and what it showed. Say that no browser test ran.

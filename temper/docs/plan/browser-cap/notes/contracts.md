# browser-cap: the run's binding calls

Everything here is in crate `temper-client-browser-world` (`temper/tests/client-browser`), or names what its tests use.

## `Stories`, from `stories-named`

`temper/tests/client-browser/src/story.rs`, exported from the crate's root as `temper_client_browser_world::Stories`.

```rust
pub struct Stories { /* person: u32, begun: Vec<(u32, &'static str)> */ }
impl Stories {
    #[must_use]
    pub fn of_person(person: u32) -> Self;
    /// The story `name` begins at the next step of `steps`, the script as built so far.
    pub fn begin(&mut self, name: &'static str, steps: &[Step]);
    /// The story step `step` stands in: the last begun at or before it.
    #[must_use]
    pub fn story(&self, step: u32) -> Option<&'static str>;
    /// Panic unless the run is done, with the failed step's story first in the message.
    #[track_caller]
    pub fn assert_done(&self, outcome: Outcome);
}
```

`assert_done`'s panic reads `{story}: person {person}, step {step}: {report}`. For a person this `Stories` isn't of, or a step before any story begins, it reads `person {person}, step {step}: {report}`.

## The history's calls, from `stories-named`

`temper/tests/client-browser/src/history.rs`, on `Tier`, beside `ctl`, `ctl_chat`, `ctl_face` and `held_setup`:

```rust
/// Start chat `task`, the next number the box gives, waiting for its answer and each following message's.
#[must_use]
pub fn ctl_next_chat(&self, task: u32, first: &str, then: &[&str]) -> Vec<Step>;
/// Send chat `task` each of `words`, each answered before the next; `from` numbers
/// the first message's key, past any this chat was sent.
#[must_use]
pub fn ctl_messages(&self, task: u32, from: u32, words: &[&str]) -> Vec<Step>;
```

`ctl_chat(first, then)` stays: the playground's start, then chat 1. Keys are `history-chat-{task}` and `history-message-{task}-{n}`, unique across a deployment's chats.

## The merged tests' names

| 4.6 | Test | File |
|---|---|---|
| 2 and 11 | `history_keeps_the_chat_focus_and_filter_across_reload_and_back`, unchanged | `browser_pages.rs` |
| 4 and 5 | `enter_starts_one_chat_and_the_composer_keeps_the_focus`, unchanged | `browser_pages.rs` |
| 9 | `the_sign_in_opens_a_tab_and_the_page_moves_on_to_ready`, unchanged | `browser_accounts.rs` |
| 14 | `a_chat_opens_at_its_end_and_reads_back_keeping_its_place`, new | `browser_pages.rs` |

`temper/docs/design/client/browser.status.md` cites `browser_pages`'s `enter_starts_one_chat_and_the_composer_keeps_the_focus` by name, so the kept names stay.

## Story labels

Reading: `short`, `latest`, `unanchored`, `long`. History: `call line`, `history`, `project across a reload`. Composer: `composer`, `model picker`, `panic`. Sign-in: `welcome`, `connect model`.

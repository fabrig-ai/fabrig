+++
id = "draft-link"
part = "goals-link"
kind = "feature"
needs = []
files = ["temper/tests/client-browser/tests/browser_all/browser_task_pages.rs", "temper/docs/design/client/browser.status.md"]
effort = "medium"
minutes = 15
+++

# Goals' draft link leaves the browser tier

## Goal

`start_goal_opens_chats_for_a_draft` presses a link and walks Back and Forward. No browser is needed for the link's target, and `history_keeps_the_chat_focus_and_filter_across_reload_and_back` holds history. The test goes, and its file's header says what the file now holds.

Acceptance:

- The test is gone, with the imports only it used. The tier lists 12 tests: `heavy cargo nextest list --profile browser-temper -p temper-client-browser-world --test browser_all`.
- `browser_task_pages.rs`'s module header describes the two step functions left in it, which `browser_pages.rs` walks, citing `temper/docs/design/client/browser.md`, 4.6, item 11. It names no test that's gone.
- `temper-client-view`'s `the_link_shows_with_a_tracked_template` passes. It holds the target: pressing the link decodes to `Act::Go { address: Address::Chats { filter: ChatsFilter::All } }`. Read its assertion before relying on it. No world test is added: this view test is the lowest tier that shows the target.

## Inputs

- `temper/docs/plan/browser-trim/notes/common.md`.
- `temper/docs/design/client/browser.md`, section 4 and 4.6, *Not in the tier*.

## Owns and may change

- Owns the two files in the header.
- **Departs from your text:** in `temper/docs/design/client/browser.status.md`, remove the one **Not yet** line for 4.6 that names `start_goal_opens_chats_for_a_draft`. No other design edit.
- `plan_and_agent_steps` and `reload_chats` stay exactly as they are: a later task of this plan decides them. `main.rs` keeps its `mod` line.
- If the view's test doesn't hold the target as above, add the assertion there (`temper/crates/temper-client-view/src/tests/goals.rs`) and say so in the report.

## Steps

1. Remove the test; correct the header.
2. `heavy cargo nextest run -p temper-client-view --lib -E 'test(the_link_shows_with_a_tracked_template)'`.
3. The listing above: 12 tests.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-browser-world --all-targets -- -D warnings`; `python3 -I -B tools/citations/citations.py --base origin/main`. Commit and push.

No browser run is needed: what's left in the file doesn't change.

## Report

As the implementer's text asks, with the listing's count.

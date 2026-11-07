+++
id = "models-read"
part = "page"
kind = "feature"
needs = ["client-facts"]
files = ["temper/crates/temper-client-domain/src/pages/models.rs", "temper/crates/temper-client-domain/src/tests/pages/models.rs", "temper/crates/temper-client-view/src/pages/models.rs", "temper/crates/temper-client-view/src/choice/picker.rs", "temper/crates/temper-client-view/src/root.rs", "temper/crates/temper-client-view/src/words.rs", "temper/crates/temper-client-view/src/tests/models.rs", "temper/crates/temper-client-view/src/tests/choice.rs", "temper/crates/temper-client-shell-web/temper.css", "temper/crates/temper-client-shell-terminal/src/tests/**", "temper/crates/temper-client-tools/src/jig/render/**", "temper/crates/temper-client-tools/src/tests/jig/accounts.rs", "temper/tools/gallery/**", "temper/docs/design/client/README.status.md"]
effort = "high"
minutes = 140
+++

# Settings, Models and All models draw it

## Goal

The states the look request `model-picker`'s 07 and message draw and step 2 left: "Read again", "Couldn't read OpenAI's list · 2 hours ago", "Read from OpenAI 2 hours ago", and "new" on Settings and All models.

Acceptance:

- `temper/docs/design/client/README.md`, 3.5, *The `Models` tab* and *All models*, as written there, in both shells, rows of `tests/models.rs:258,319,367` and `tests/choice.rs:2080`, which today says no "new", and `:2117`:
  - **The clause:** a section's second clause by its accounts' `list`s: failed newest first, "Couldn't read OpenAI's list · 2 hours ago."; else "Read from OpenAI 2 hours ago."; else "These come with temper."; a Claude subscription keeps "These come with temper."; an account with two states shows the failure. The payment clause stays in every case.
  - **Read again:** in `.temper-settings-small` where any of the section's accounts has a `list`, absent otherwise; a press sends one `ReadLists` with the section's provider, says "Reading…", off until its answer and the `accounts` watch moved; a refusal shows under the clause; a member sees it for their own accounts and not a deployment's, by the document's rows.
  - **New:** " · new" in the row's `small`, and `.temper-model-tag` "new" on All models, while `new_until` is later than now, at just before, exactly at and just after a week; a stock or a person's has none.
  - **The time** says "2 hours ago" by `words.rs:6013`'s formatter, passing `Times` in at `root.rs:276`, and it redraws by `root.rs:105,356`'s schedule.
- A row of `temper-client-domain/src/tests/pages/models.rs:66,102`'s: one act sent, its answer clears it, its refusal shows, a repeat press while held sends nothing.
- **The terminal:** the same clause as a line and "Read again" as the terminal has the switch on How work runs; `tests/golden/models.txt` regenerates.
- **`temper ctl`:** `temper ctl model list` says `new_until` in JSON and "new" in text at `jig/render/document/accounts.rs:456,531` and `account_lines.rs:7,72`. A row of `tests/jig/accounts.rs:80,353`.
- **The gallery:** new states in `temper/tools/gallery/src/{samples.rs:148,worlds.rs:5306,worlds/picker.rs:77}` and `names.txt:201`, sorted: a provider read, one failed, one new model. Rows in `each_state_holds_what_its_name_says`'s `HOLDS` at `tests.rs:559`, the row asserting the words.
- `temper.css`: a rule for the source line's button, under `/* ==== ui: settings ==== */`, from 07's `.temper-models-source`, since `.temper-settings-small` exists.
- Each commit carries the trailer `Look-request: model-picker`, directly above the other trailers (`temper/docs/design/draft/look-requests.md`, 3).
- `README.status.md`'s 3.5 line goes.
- No new browser test. No journey's recording changes here, since no exchange does; its prints reprint if the words are in them, by their tool.

## Inputs

- `temper/docs/plan/model-catalogue-3/notes/common.md`.
- `temper/docs/plan/model-catalogue-3/notes/contracts.md`, 8.
- `temper/docs/design/client/README.md`, 3.5, *The `Models` tab*, *All models*.
- The look: `git show origin/look-requests:temper/docs/design/client/journeys/model-picker/07.html` and `04.html`; `git log origin/look-requests --grep='Look-request: model-picker' -1` for its message. It's the look's source of truth for classes: `.temper-models-source`, `.temper-settings-detail`, `.temper-settings-small`, `.temper-model-tag`.

## Owns and may change

Owns the header's files.

- `pages/models.rs:86,104` (view), the clause; `:137` the row; `temper-client-domain/src/pages/models.rs:9,16,100,150,162` the act, beside the switch's, `pages.rs:599,1038` its dispatch; `frame.rs:98` a binding; `choice/picker.rs:443,472` the tag.
- `words.rs:239,1706,3025,5611`: new words `ModelsReadFrom`, `ModelsReadFailed`, `ModelsReadAgain`, `ModelsReading`, `ModelsNew`.
- `temper/crates/temper-client-domain/src/pages.rs` for the dispatch alone.

Options, each pre-approved:

- The provider's word in the clause is the section's title's: `provider_words(name).0`, and "Claude" for `anthropic`.
- A section's newest `tried`-with-failure is over its usable accounts of `pays`' filter; other people's accounts aren't in the document.
- A client's clock behind `new_until` isn't a concern: it's what the formatter does for every other time.
- A gallery world's fixed "now" sets the "2 hours".

## Steps

Then `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-domain -p temper-client-view -p temper-client-tools -p temper-client-shell-terminal -p temper-client-gallery --all-targets -- -D warnings`, naming the gallery's package as its manifest does; `heavy cargo nextest run -p temper-client-domain -p temper-client-view -p temper-client-tools -p temper-client-shell-terminal`, and the gallery's; and `temper-client-journeys`, reprinting what its check says is stale. Commit and push.

## Report

As the implementer's text says. Name what `07.html` draws that the design doesn't, each one a row for `list-close`.

+++
id = "report-in-finish"
part = "report-in-finish"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-engine-domain/src/route/forge_route/relay.rs",
  "temper/crates/temper-engine-domain/src/route/tools.rs",
  "temper/crates/temper-service/src/stock/coding.txt",
  "temper/crates/temper-service/src/tests/stock.rs",
  "temper/crates/temper-service/src/tests/stock_brief.rs",
  "temper/docs/examples/reference/settings.json",
  "temper/tools/stage-one/src/tests/fixtures/judge/brief.md",
  "temper/tests/engine/domain/src/forge_routes.rs",
  "temper/tests/engine/domain/tests/suite/forge_routes/questions.rs",
  "temper/docs/design/domain/forge.status.md",
]
effort = "high"
minutes = 60
+++

# A producer's report goes in finish, and it is told so at once

## Goal

A producer sent its report with `message` to its own change. The change answered with feedback meant for a requester; that answer arrived between the `message` and the `finish`, so smith refused the `finish`, and the run paid a turn to read words meant for someone else. Up to three times in one goal. Smith's rule stays: nothing changes in smith or jig.

Tell the producer in its charter, refuse its plain words in the `message` call itself, and fit the feedback to it where such words still reach a step.

Acceptance:

- **First a failing test:** `a_producers_words_are_refused_by_the_call_and_its_finish_goes_through` (below) runs red before the fix: today the call answers `Sent`.
- A `message` call in the words form from a change's asking delegate to that change, under an enabled route (`up`, `foreman`), answers with an error whose text is exactly the words below. Nothing enters the change's inbox and nothing reaches the producer's, and its `finish` goes through.
- Anything but a question from such an agent, found in the change's inbox at a step, is answered with those same bytes as `MessageKind::Words`, and its `finish` goes through once it has read them.
- The words are one constant, used by both:

  ```
  Your report goes in finish; this change takes no words. To ask: message target=this change, form=question.
  ```
- Requesters hear what they heard: `a_question_from_another_task_names_the_enabled_route` is not edited and passes, and so do `goal_questions.rs`'s requester's words (`Words do not answer`, about line 704) and `asks_no_one.rs`. A producer's question still goes up (`a_producers_question_goes_up_and_the_answer_comes_down`). A delegate's question past the open-queue ceiling keeps the fallback feedback.
- `temper/crates/temper-service/src/stock/coding.txt`: the sentence `Summarize the change.` becomes `Summarize the change in finish; message your change only in the question form.`, 57 bytes more. A test holds it.
- `temper/crates/temper-service/src/tests/stock_brief.rs`: row `(2, false, 33_561, 366_574)` becomes `(2, false, 33_618, 366_517)`. Nothing else should move. The test's `host-part row:` line prints the measured row: where it differs, the measure wins, and the report says what moved and by how much.
- `temper/docs/design/domain/forge.status.md`: see below.

## Inputs

- `temper/docs/design/domain/forge.md`, 8.11 (*Taken at its step*, *Its own agents' words*); 8.1, the paragraph on a change made with questions `none`.
- `smith/docs/design/domain/run.md`, section 6 (*A finish crossing a message*).
- `temper/docs/design/protocol/configuration.md`, 13.4 (charter 2) and 13.9.

## Owns and may change

Owns the `files` above, each only as this spec says.

May change, only so:

- `temper/crates/temper-engine-domain/src/route/forge_route.rs`: the one `pub(crate) use relay::{...}` line. `repair-green` and `brief-failure` work in this file's neighbours, so make no other hunk there; `takes_no_messages` stays as it is.
- `temper/docs/design/domain/forge.status.md`: **this departs from the implementer's text,** for this file alone. Delete the **Not yet** entry `8.11: an agent's own words: ...`. In the entry after it, `8.11: a repair's or a resolution's question, ...`, say that its words, refused as a producer's are, are unobserved in stories too, since the stories here run a producing task only.

`temper/docs/examples/reference/settings.json` and the judge's golden change only because they read the stock words:

- `temper/crates/temper/src/tests/reference_settings.rs` holds the reference coder's instructions to start with `coding.txt`, `You can run nothing.` replaced. So charter 2's `instructions` in `settings.json` take the same new sentence, and no other edit. Do not edit that test.
- `temper-stage-one` renders those instructions into `src/tests/fixtures/judge/brief.md`, line 193. Regenerate it: `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper-stage-one --lib -E 'test(the_brief_for_the_profile_matches_its_golden)'`, then run the test again without the variable. If heavy does not carry the variable, edit that one line with the same sentence, let the test prove the bytes, and say so in the report.

Not readers, and not edited; each has its own words: `temper/crates/temper/tests/fixtures/cluster/settings.json`, `temper/crates/temper-fake/src/tests/demo.rs`'s `CODING`.

No other story has a producer send words to its own change: the words-form senders in `temper/tests` (`root_routes.rs`, `asks_no_one.rs`, `goal_questions.rs`, `temper/tests/system`, `temper/tests/real-loop`) are parents, requesters or subowners messaging plain tasks.

Options, each settled:

- **Message tool's description and `docs/help/`:** unchanged. No protocol, schema, stored record, `tasks::Refusal` or `CallAnswer` changes: the refusal is a `SettledAnswer::Host`, the path `goal::refusal` already takes. If the fix seems to need any of those, that is a `Question:`.
- **The call's refusal is not recorded** in `call_parts`, as `goal::refusal`'s is not: `host_route::rendered` relays it. A retried call decides again. Leave that.
- **`stock_brief.rs` on a catch-up:** `note-parts`, `ui-away` and `ui-chat-draft` move these rows too. Take the trunk's row 2, and add 57 to its first figure and take 57 from its second. The test's `host-part row:` line prints the measured row to check it by.
- **If `kinds::produce` does not suit `up_world`:** end the producer as `an_asker_that_ends_drops_its_question` does, and assert the change takes that end.

## Steps

The refusal, in `relay.rs`, beside `asking_delegate`. Inside these, the shape is yours.

```rust
/// What an agent hears of its words to its own change: its report goes in `finish`
/// (temper/docs/design/domain/forge.md, 8.11).
const REPORT_IN_FINISH: &[u8] = b"Your report goes in finish; this change takes no words. To ask: message target=this change, form=question.";

/// Refuse `task`'s words to `target` where `target` is the change it works for, under an enabled route.
pub(crate) fn own_agents_words(domain: &Domain, task: u64, target: u64) -> Option<&'static [u8]>;
```

Read `domain.forge.change(target)`, its `questions.route` and `delegate`, through `asking_delegate`: a producing task, a repair or a resolution.

In `take`, where the reply is built: a `word.kind` other than `Question` from such an agent takes `REPORT_IN_FINISH`, and anything else takes `takes_no_messages` as now. The reply's `kind` stays as computed there.

In `route/tools.rs`, the `Tool::Message` arm of `relay_payload`:

```rust
let kind = match form {
    MessageForm::Words => {
        if let Some(words) = forge_route::own_agents_words(domain, key.task, target) {
            super::host_route::rendered(domain, reply_to, core::SettledAnswer::Host { error: true, body: words.into() });
            return;
        }
        tasks::MessageKind::Words
    }
    MessageForm::Question => tasks::MessageKind::Question,
    MessageForm::Answer { question } => tasks::MessageKind::Answer { question },
};
```

The world, in `temper/tests/engine/domain/src/forge_routes.rs`, beside `restore_delegate_question`:

```rust
/// Restore a delegate's plain words in its change's inbox at a committed cut.
pub fn restore_delegate_words(&mut self, delegate: u64, words: &[u8]) -> u64;
```

As `restore_delegate_question`, with `MessageKind::Words` and no credit. A private helper they share is fine, in that file. It is the only way to the step's case, now that the call refuses: a producer's answer form is refused by jig (`check_message`, no credit), and words already in an inbox across a restart are what is left.

The stories, in `forge_routes/questions.rs`, on `up_world(None)`, with one `const REPORT_IN_FINISH: &[u8]` of the test's own:

```rust
#[test]
fn a_producers_words_are_refused_by_the_call_and_its_finish_goes_through() {
    // call(.., &producer, CALL + 1, Tool::Message { target: change, form: Words, words: b"My report" });
    // world.call_errors' newest text == REPORT_IN_FINISH;
    // replies(&world, producer.task, change) is empty; the change's inbox holds nothing from the producer.
    // temper_engine_domain_world::kinds::produce(&mut world, change, &producer);
    // tick until row(&world, change).change.state has left State::Producing.
}

#[test]
fn a_producers_words_found_at_a_step_hear_the_same_words_and_its_finish_goes_through_once_read() {
    // world.restore_delegate_words(producer.task, b"My report");
    // tick until replies(&world, producer.task, change) has one: kind Words, words == REPORT_IN_FINISH.
    // protocol::turn(CHANNEL, producer.task, producer.attempt, next turn, 0, Some(that word's number), ..): it reads it.
    // kinds::produce, and tick until row(&world, change).change.state has left State::Producing.
}
```

In `temper-service/src/tests/stock.rs`, beside the other `the_coders_words_*`:

```rust
#[test]
fn the_coders_words_put_its_report_in_finish() {
    // coding.txt contains "Summarize the change in finish; message your change only in the question form."
}
```

Each bites: drop the `tools.rs` check, then the `take` arm, then the sentence, and see its test fail for that reason.

Narrow, as you go:

- `heavy cargo nextest run -p temper-engine-domain-world --test suite -E 'test(forge_routes::questions) or test(forge_routes::goal_questions) or test(forge_routes::asks_no_one)'`

This changes what `coding.txt` means to its readers, so run them too, once:

- `heavy cargo nextest run -p temper-service --lib -E 'test(stock)'`
- `heavy cargo nextest run -p temper --lib -E 'test(reference)'`
- `heavy cargo nextest run -p temper-stage-one --lib -E 'test(judge)'`
- `heavy cargo nextest run -p temper-engine-domain-world --test suite -E 'test(settings::b2)'`

Then, once: `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-engine-domain -p temper-engine-domain-world -p temper-service --all-targets -- -D warnings`; the focused tests of those packages and their direct dependents. Commit and push.

## Report

The implementer's text's report, and: what moved in `stock_brief.rs` and by how much; whether heavy carried `SKEIN_CODEGEN_WRITE`; the red the first story showed before the fix.

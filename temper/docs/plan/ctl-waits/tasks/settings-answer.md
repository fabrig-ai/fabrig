+++
id = "settings-answer"
part = "settings-answer"
kind = "feature"
needs = []
files = [
  "jig/crates/jig-protocol-engine-clients/src/translate/llm.rs",
  "jig/crates/jig-protocol-engine-clients/src/tests/llm.rs",
  "jig/tests/client-protocol/tests/suite/settings.rs",
]
effort = "high"
minutes = 40
+++

# A settings change naming an effort no model offered gets its answer

## Goal

An earlier walk's settings item named an effort no model offered, and got no answer at all: a `change-settings` kept and unanswered.
The request says this may be the same family as a kept message's wait. Reproduce it where it lives, at the clients' door; a
retained act ends in its terminal answer. Answer it in the words the door already has, or say plainly that it already is.

Acceptance:

- **A test for every place a settings item names a model and an effort,** for both `ChangeSettings` and a `Propose` of settings: each
  is answered *refused*, naming the model and `LlmWhy::Effort`.
- **A round trip through the real client and component:** the change is answered with a final refusal, and its handle ends.
- **If it reproduces,** seen red first, and fixed at the door. **If it doesn't,** the report says so plainly, and the tests stay as its
  guard.
- **No wire or stored change.** The refusal is `w::LlmWhy::Effort`, which exists.

## Inputs

- `temper/docs/plan/ctl-waits/notes/common.md`.
- `jig/docs/design/client/domain/contract.md`, 4 (unsupported model choices).
- `temper/docs/design/client/tools.md`, 7: a kept request lives until it's answered.

No design changes: the design already says this, and the code comes to it.

## Owns and may change

**Owns:** the `files` above. Nothing in `temper/` changes.

**What this cut read, and did not run:**

- `request.rs`: `w::Ask::ChangeSettings` calls `refuse_llm`, then `translate::settings::change`, whose `Err` is `not_a_record`.
- `refuse_llm` answers only on `Ok(Some(answer))`, so an `Err` from `translate::llm::refused` falls through to that `not_a_record`.
- `jig-protocol-engine-clients-http` makes `NotARecord` an HTTP 400, and `jig-client-protocol-http`'s `answered` makes any status but
  200 `Heard::Lost`. So the client retries it, and only retention ends it: **a silent route, if a change a person can write reaches it.**
- `translate/llm.rs`'s `settings_items` covers a template's `llm`, a charter's `model` and `alternatives`, and a policy's and rules'
  model names. It stops at the first finding, and it answers on bytes its `piece` and `SettingItem::decode` can read.

**Pre-approved:**

- **A person-writable change reaching `not_a_record`:** answer it at the door, with the existing refusal, in `translate/llm.rs` or
  `request.rs`, which is then open to this task.
- **A silence past the door** (core's settings dispatch, `jig/crates/jig-domain-core/src/settings/`): reduce it to a step test there
  and fix it there, which is then open; the same rule, no format change.
- **`jig-protocol-engine-clients/src/translate/**` sits in `ui-proposal-card/proposal-facts`'s glob:** keep the edit small, and in the
  files above if you can.

**Left alone, and said in the report:** any status but 200 stays `Lost`, so a kept request that's truly not a record retries until
retention. Telling a 400 apart is the protocol's, a question for its owner, and not this task's.

## Steps

**1. Step tests, failing first if they fail,** in `src/tests/llm.rs`, beside
`a_choice_naming_a_model_or_an_effort_configuration_lacks_is_refused_by_its_name`, using its `refused` and `named`: a table of each
place, over `ChangeSettings` and `Propose`:

- a template's `llm`, its main and a subagent;
- a charter's `model`, and a later alternative after a good first;
- a good item, then a bad one in a later item of the same change.

Add a row for an effort on a model no one configured, which stays `LlmWhy::Unconfigured`. A policy's and rules' model names carry no
effort: they keep their existing tests.

**2. The round trip,** in `jig/tests/client-protocol/tests/suite/settings.rs`, beside
`settings_requests_reach_the_core_and_keep_its_stale_fact`, in the shape
`a_settings_change_naming_an_effort_no_model_offered_is_answered_refused`: the real client and the real component; the answer is a
final refusal, the request's handle ends, and nothing stays retrying.

**3. The fix,** if either is red. Read the failure first: what the door answered, and which route it took.

**4. Checks:** `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p jig-protocol-engine-clients -p
jig-client-protocol-world --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-protocol-engine-clients --lib`; `heavy cargo
nextest run -p jig-client-protocol-world`. Commit and push.

## Report

**Reproduced or not,** plainly: the walk's silence, and what route explains it, or that this path answers and the walk's cause lies
elsewhere, with what you ruled out. If a test only guards, say it found nothing. Its bite: the rule broken and the failure seen.

+++
id = "read-ends"
part = "read-ends"
kind = "feature"
needs = ["line-surface"]
files = [
  "jig/crates/jig-client-protocol-link/src/component.rs",
  "jig/crates/jig-client-protocol-link/src/tests/reads.rs",
  "jig/crates/jig-client-protocol-link/src/tests/fixtures/link_reads.rs",
  "jig/crates/jig-client-domain/src/contract/reads.rs",
  "jig/crates/jig-protocol-engine-clients/src/reads.rs",
  "jig/crates/jig-protocol-engine-clients/src/tests/reads.rs",
  "jig/docs/design/client/domain/client.status.md",
]
effort = "high"
minutes = 35
+++

# An answer that can't be decoded ends its read

## Goal

A read whose answer came and can't be decoded ends `Unreachable`, which is transient, so temper asks again for as long as the page is open and nobody's told. The engine does the same by closing an exchange whose answer it can't encode. Make both end the read for good, and leave a lost exchange as it is.

Acceptance:

- A jig read whose answer doesn't decode ends `ReadNone::Malformed`, whether the body is malformed, its value past the client's limits, or of another wire version, and `Fact::UndecodableDocument` is still reported. It has one terminal.
- A family read's failures end as they did: `Readout::Family { record: None }`, with the fact.
- A lost exchange, and a read refused for want of room, still end `Unreachable`.
- The clients' component answers a read it can't encode within its wire's bounds with `ReadNone::Malformed`, where it closed the exchange, and the exchange stays usable as any answered one is.
- `ReadNone::Malformed`'s doc comment says what it now is: the answer, or the visible turn, could not be read or decoded whole.
- The clause `16: an answer that does not decode ends its read unreachable, and the clients' component closes an exchange whose answer it cannot encode.` leaves `jig/docs/design/client/domain/client.status.md`'s **Not yet**.

## What this does to the other reads

`failed` in `component.rs` is shared, and this leaves it so: only `read_answer`'s decode arm stops calling it for a read. So the change reaches every jig read whose answer doesn't decode, no read whose exchange was lost, and nothing but a read. What each reader does with `Malformed`, which it already handles:

| Reader | Before | After |
|---|---|---|
| A chat's or agent's activity (`pages/chat.rs`, `pages/task.rs`) | asked again, 1 to 30 s | ended; "Couldn't load earlier messages" |
| A task's document (`pages/task.rs`, `document_page`) | asked again | ended; "This task could not be read." |
| A chat's draft notes (`pages/chat.rs`), the chats' notes (`pages/chats.rs`) | asked again | ended |
| A forge's way (`forge/ways.rs`, `transient`) | asked again | ended |
| `temper ctl` (`temper-client-tools/src/jig/render.rs`) | "unreachable", status 1 | "malformed document", status 1 |
| jig's test kits and examples | busy | unknown |

## Inputs

- `jig/docs/design/client/domain/client.md`, section 7, *A read*, and section 16, *An answer that came and does not decode ends its read* and *The clients' component*.
- `temper/docs/plan/long-words/notes/contracts.md`, *Reads*: binding.
- `temper/docs/plan/long-words/notes/common.md`.

## Owns and may change

Owns the files in the header. `client.status.md` is shared: remove your clause alone. May also change, these edits alone:

- `jig/crates/jig-client/src/tests/**`, `jig/tests/client-protocol/tests/**`, `jig/testing/jig-client-kit/**`, `jig/testing/jig-client-scripted-engine/**`: an assertion that named `Unreachable` for an answer that doesn't decode, and nothing for a lost exchange.
- `temper/crates/temper-client-domain/src/tests/**`, `temper/crates/temper-client-tools/src/tests/**`: the same assertion's value, if the readers' run below finds one. Report each.

Options, each decided:

- `Undecodable::TooLarge` ends `Malformed` too, as the turn path's decode failure does: `ReadNone::TooLarge` carries a turn's original size, which a page has none of.
- `read_response` takes what it needs to answer (`routes::read_none(Why::Malformed, limits)`) as a new argument, its callers passing `&env.limits.wire`.
- If an engine test pins a closed exchange for an encoding failure, it now pins the answer.

Not this task: the view, and what a page says: `read-ends-page`'s.

## Steps

1. Red first, in `tests/reads.rs`: rename and turn `a_malformed_read_body_is_unreachable`, `a_value_past_the_client_s_limits_is_unreachable_and_reported`, `a_malformed_read_value_is_unreachable_and_reported` and `a_read_value_of_another_wire_version_is_unreachable_and_reported` to say malformed, with a `read_malformed` fixture beside `read_unreachable`; add `a_lost_read_stays_unreachable`. In `jig-protocol-engine-clients`' `tests/reads.rs`, `an_answer_it_cannot_encode_is_malformed_not_closed`, at a wire with a low `activity_words_words`.
2. The arm in `read_answer`; the answer in `read_response`; the comment.
3. This changes what a shared item means: besides the implementer's text's checks, run `heavy cargo nextest run -p jig-client -p jig-client-protocol-link -p jig-protocol-engine-clients -p jig-client-protocol-world -p jig-client-view -p temper-client-domain -p temper-client-view -p temper-client-tools`.
4. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p jig-client-protocol-link -p jig-client-domain -p jig-protocol-engine-clients --all-targets -- -D warnings`. Commit and push.

## Report

As the implementer's text asks, with any reader in the table above that didn't do what it says.

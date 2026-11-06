+++
id = "run-wait-status"
part = "run-wait-status"
kind = "feature"
needs = []
files = [
  "jig/crates/jig-client/src/facts.rs",
  "jig/crates/jig-client/src/words.rs",
  "jig/crates/jig-client/src/tests/facts.rs",
  "jig/crates/jig-client-view/src/words/words/status.rs",
  "jig/crates/jig-client/src/words/english.rs",
]
effort = "medium"
minutes = 45
+++

# A chat whose run can't start doesn't read as starting

## Goal

`chat_status` turns only `TaskRunWait::Refused` into a status, so a run waiting for an account, a credential
or a host reads "Starting" for ever. Give those their own status, as `jig/docs/design/client/domain/client.md`,
19.1 has it. What temper's pages say is `wait-says-why`'s.

Acceptance:

- A chat due or preparing with `run_wait` `Account`, `Credential` or `Host` is `ChatStatus::RunWaiting(kind)`, in the place
  `Refused` holds in the order: held, closing, ended, a party, a dependency all win over it.
- `Account` with `why` `Refreshing`, or `Source(Asking)` or `Source(Granted)`, stays `Starting`.
- Claimed, running or backing off, it reads its stage, whatever `run_wait` says.
- With no `run_wait`, as a summary row passes, it reads as it does today.
- `Condition::Replied`: `RunWaiting(Account)` is true, `RunWaiting(Credential | Host)` false.

## Inputs

- `temper/docs/plan/first-usable-model/notes/common.md`, `temper/docs/plan/first-usable-model/notes/contracts.md`.
- `jig/docs/design/client/domain/client.md`, section 19's status table and 19.1.

## Owns and may change

Owns, as `files` lists them, and removes `jig/docs/design/client/domain/client.status.md`'s **Not yet** entry
`19, 19.1`.

May also change, compiler-forced arms only, in the other tree:

- `temper/crates/temper-client-view/src/pages/chats.rs`, `status_words`: one arm each at about lines 456, 472 and
  482. `RunWaiting(_)` has no hole, its token is `crate::look::STATUS_WAITING`, and its name is
  `words.say_word(jig_client::words::Word::status(&status), holes.as_slice())` with the arm beside it. That token
  stops the dots (`working`, in the same file). Nothing else in that file: it's open in other drafts.
- `temper/crates/temper-client-tools/src/jig/conditions.rs`, about line 87, as the acceptance says.
- `temper/crates/temper-client-tools/src/tests/jig/conditions.rs`: its row for each.
- `temper/crates/temper-client-view/src/tests/chat.rs`, about line 1753, or any other test that pinned `Starting` with a
  wait: the expected word alone.

## Steps

1. Test first, in `jig-client/src/tests/facts.rs`, on the model of
   `refusal_status_only_replaces_an_unplaced_starting_chat`, over `rows()`; and one table of each
   `TaskWaitWhy` and `TaskSourceState` to its status. The table holds the three that stay `Starting`, and enough
   others to fail on a wrong default.
2. `facts.rs`, about line 51, per `notes/contracts.md`. Every `match` lists every variant.
3. `words.rs`: `StatusWord::{WaitingForAccount, WaitingForCredential, WaitingForHost}`, in `status`, `ALL` and the
   English table with fresh ids; `jig-client-view`'s `chat_status` arm. Its reference view draws the word and no
   more.
4. `jig/tests/client-protocol/tests/suite/client.rs`, about lines 666 to 670, if it pins the order: add the row.

Tests, through `heavy`: `-p jig-client --lib`, `-p jig-client-view --lib`, the `jig/tests/client-protocol` world,
`-p temper-client-tools --lib`, `-p temper-client-view --lib`. Then fmt, `old_forms.py` for jig and temper, clippy on the packages
you changed, commit and push.

## Report

As the implementer's text says, and each overlap file you changed.

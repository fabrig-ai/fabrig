+++
id = "remember-route"
part = "route"
kind = "feature"
needs = ["remember-surface", "core-surface"]
files = ["temper/crates/temper-engine-domain/src/route/remember.rs", "temper/crates/temper-engine-domain/src/route/remember/**", "temper/crates/temper-engine-domain/src/tests/remember.rs", "temper/crates/temper-engine-domain/src/tests.rs"]
effort = "high"
minutes = 200
+++

## Goal

Make temper's root write a `remember` call: the chat's requester's party
scope loaded, the line checked, the entry added, changed or dropped through
jig's named actions `Note` and `NoteDelete`, and the answer the whole list
after the write (`temper/docs/design/domain/agent.md`, 8.4.4, *The root
writes it*). jig admits a party's scope in task `party-runs`; this task's
own tests are the root's pure step tests, as `document`'s are.

Acceptance:

- `route/remember.rs`' `call` replaces `remember-surface`'s stub:
  - it takes the task's delegation context (`route/see.rs:403`): a
    requester other than `Party::Person(party)` answers
    `RememberRefusal::NoPerson` at once;
  - it loads `Scope::Party { party }`'s index whole, as `route/document.rs`
    loads its scope's (`call`, `loaded`, `index`), its pages joined, a
    load that fails or is cut answering `Unread`, the notes child busy
    `Busy`;
  - it checks before any write: the text trimmed of its spaces, one line
    and not empty, else `NotOneLine`; within `notes.party_bytes`, else
    `TooLong { bytes, bound }`; an `add` to a list of `notes.party_entries`,
    `Full { bound }`; a `line` past the list, `NoLine { line, lines }`;
  - it writes: `add` as `NamedAction::Note` with a new entry (name 0, the
    scope, the text as its description, an empty body, references the
    chat's own number, `recalled: None`); `change` as `NamedAction::Note`
    at that line's name and the index's revision, its references as the
    entry has them (loaded as `document` loads its entry); `drop` as
    `NamedAction::NoteDelete { scope, name, recalled }`;
  - its flight joins jig's answer as `DocumentFlight` joins `document`'s
    (`route/host_route.rs`): a write answered becomes `Remembered::Done`
    with the list after it, made from the loaded lines with the change
    applied, `line` the place written (for `add`, the last); moved
    `Moved { action, line }`; busy `Busy`; any other refusal
    `Note(refusal)`.
  - Its own refusals keep no record and change nothing, so the same call
    made again reads afresh.
- One table test, `src/tests/remember.rs` (`a_remember_call_keeps_changes_and_drops_by_place`,
  and `a_refused_remember_call_writes_nothing`), its rows driving the root
  as `src/tests/chat_document.rs` drives `document`: each row the loaded
  index, the call, the named action sent (or none) and the answer after
  jig's answer: add to none, to three, to eleven; add to twelve; change
  line 2 of three; drop line 1 of three and line 3 of three; a line of 120
  and of 121 bytes; an empty and a two-line text; line 4 of three; a moved
  answer; a busy load; a task requested by a task.
- Delete `temper/docs/design/domain/agent.status.md`'s **Not yet** entry
  `8.4.4: no call keeps, changes or drops a line`.

## Inputs

- `temper/docs/plan/ui-ask-m4/notes/common.md`; `notes/contracts.md`,
  sections 2 and 4.
- `temper/docs/design/domain/agent.md`, 8.4.4 whole; 8.4.3, *The root writes
  it*, the model.
- `jig/docs/design/domain/engine.md`, 10.1, *Its entries are preferences*
  and *Written for its party*.
- The model: `temper-engine-domain/src/route/document.rs` (`call`, `loaded`,
  `index`, `entry`, `write`), `src/route/host_route.rs:177`
  (`DocumentFlight::join`) and `:461` (`render`), `src/tests/chat_document.rs`.

## Owns and may change

- **Owns:** the files in this spec's header.
- **May change:** `route/host_route.rs` and `route/tools.rs`, the lines that
  make and join the flight for `Tool::Remember`, beside `document`'s;
  `src/archive.rs:39` and `src/restart.rs:165`, `:200`, where a load's
  completion is routed, an arm each, as `document`'s.
- **Options, pre-approved:** if the root cannot read an entry's references
  from the index alone, load the entry for `change` as `document` loads
  its note, and say so; if jig answers a `NoteDelete` with no revision,
  the list after it is the loaded one without that line.

## Steps

1. The requester, the load and the checks; the writes; the flight and its
   answer.
2. The table; break one rule (write `add` past the bound) to see its row
   fail, and restore.
3. `cargo fmt`; `heavy cargo clippy -p temper-engine-domain --all-targets --
   -D warnings`; `heavy cargo nextest run -p temper-engine-domain --lib`;
   `heavy cargo nextest run -p temper-engine-domain-world -E
   'test(/document|asking/)'`. Commit and push.

## Report

As the implementer's text asks.

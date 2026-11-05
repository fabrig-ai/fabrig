+++
id = "fake-document"
part = "fake-document"
kind = "feature"
needs = ["chat-stock", "note-room"]
files = [
  "temper/crates/temper-init/template/temper.json",
  "temper/crates/temper-init/src/tests/documents.rs",
  "temper/crates/temper-fake/src/tests/demo.rs",
  "temper/crates/temper/tests/fake_backends/walk.rs",
  "temper/crates/temper/tests/fake_backends/walk/chat_document.rs",
]
effort = "high"
minutes = 60
+++

# A scripted turn of the fake model writes a document

## Goal

On a fresh `temper box --fake-backends`, a chat asked to write something up writes its document and then changes it, through the real engine and tools, proved through `temper ctl`. This is the end of the engine's half, and what the journey `chat-draft` walks.

Acceptance:

- **Two `cued` scripts** in `temper.json`'s `application.demo.llm` (`temper/docs/design/protocol/setup.md`, 8.3; at most 8 with `asked`), which the client never suggests:
  - cue "Write this up as a draft beside us.": one turn calling `note` with `scope {"kind":"goal","project":P,"goal":T}`, description "Away mode", a Markdown body of a few sections as `chat.txt` asks (`# Away mode`, "Draft, DATE, with the user.", two `##` sections and "## Open questions", about 30 lines), then a turn saying in a sentence what it wrote.
  - cue "Add what happens to a running goal.": one turn calling `note` with the note's `name`, `recalled: 1`, the same body with a section added in the middle; then a sentence.
- `P` and `T` are the literal numbers of a fresh box's first project and first chat, established by this task's own walk, and `name` the first note's. Scripts hold literal bytes: a wrong `T` in any other walk answers `Finding::Scope`, and the script's sentence still follows. Name the numbers in a comment if the template's form lets you, or else in the walk.
- `temper-init/src/tests/documents.rs`' `demo()` mirrors them, tokens `bytes ÷ 4` rounded up.
- **The proof,** `temper/crates/temper/tests/fake_backends/walk/chat_document.rs`, on `walk.rs`' harness, `temper ctl` with `</dev/null`: `chat start` with "Please write this up. Write this up as a draft beside us."; `task activity`, then `task turn TASK A.T`, shows `Note N revision 1 written · L lines`; a message with the second cue; `Note N revision 2 written · +A −D · line F`.
- `temper/docs/design/domain/testing.md`, section 2.4's scope is unchanged; say nothing of it unless what's true there changes.

## Inputs

- `temper/docs/plan/ui-chat-draft/notes/common.md`, `notes/contracts.md`.
- `temper/docs/design/protocol/setup.md`, its `llm.cued`, `llm.asked` and a turn's spellings; `skein/docs/design/fake-llm.md`, section 2.
- `temper/docs/design/domain/agent.md`, 8.3; `temper/crates/temper/tests/fake_backends/walk/sign_in.rs` as the shape of a walk.

## Owns and may change

Owns, as `files` lists them.

May also change: `temper/docs/examples/**` if their drift test mirrors `temper.json`; `temper/crates/temper-fake/src/demo.rs` for a validated count; `temper/docs/help/**` by their renderers.

Options, each pre-approved:

- **A cue matched elsewhere first:** a script chosen by a system cue, then by the last user message's cue. If an existing `asked` or `cued` wins, change the words of neither; use a cue only this script holds, and say so; the request's "Write this up as a draft beside us." must stay what `+ Draft` sends.
- **`script_bytes`:** if the scripts past skein's fake LLM's bound (`temper-fake` sets it), shorten the body to what fits, at least 20 lines.
- **A box's payload:** the body stays well under `tool_payload`, 10,662 (`configuration.md`, 13.8).
- **A second chat in a walk,** whose number differs: it's a refused note, not an error: leave it.

## Steps

1. The scripts and `demo()`; `heavy cargo nextest run -p temper-init -p temper-fake`.
2. The walk; find the numbers by running it; `heavy cargo nextest run -p temper --test fake_backends -E 'test(chat_document)'`, by the name of its test binary as `Cargo.toml` has it.
3. Break the instructions' route once (`T` wrong) and see the walk fail for the scope's reason.
4. **Check,** the implementer's text, then push.

## Report

As `notes/common.md`, Your report, with the numbers `P`, `T` and `name`, and the test's wall time.

## Answers

2026-11-05 (fabrig/ui). Yes: make the scoped skein change in this task: an unsuggested cued script may also answer the latest user request, counting turns from that request, with system-cue matching first. Update skein/docs/design/fake-llm.md, 2.4 with it. hearth doesn't use skein's fake LLM, so no heads-up is needed. Then finish the walk.

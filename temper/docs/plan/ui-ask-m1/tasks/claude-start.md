+++
id = "claude-start"
part = "smith-start"
kind = "feature"
needs = []
files = ["smith/crates/smith-protocol-program/src/backend/start.rs", "smith/crates/smith-protocol-program/src/backend/compact.rs", "smith/crates/smith-protocol-program/src/backend/files.rs", "smith/crates/smith-protocol-program/src/backend/tests.rs", "smith/crates/smith-protocol-program/src/backend/session_tests.rs", "smith/crates/smith-protocol-program/src/backend/open_tests.rs", "smith/crates/smith-protocol-program/src/backend/redaction.rs", "smith/tests/fake-claude/**", "smith/tests/agent-process/tests/suite/program.rs", "smith/tests/agent-process/tests/suite/program_resume.rs", "smith/tests/agent-process/src/program_world.rs", "smith/docs/design/protocol/programs.status.md"]
effort = "high"
minutes = 120
+++

# ui-ask-m1: Claude Code with no workspace replaces its prompt

## Goal

A run with no workspace starts Claude Code with its prompt replaced:
`--system-prompt-file` in place of `--append-system-prompt-file`, `--tools ""`
whatever its families, and `--disable-slash-commands`, in an empty private directory
of the run's own that is no repository. A run with a workspace starts as today. The
fake program takes and records both new flags and its working directory.

Acceptance:

- `launch_spec` (`backend/start.rs`) builds, for `self.settings.workspace` `None`, the
  arguments of `notes/contracts.md`, section 4, and its spawn's directory is
  `.runs/<run>/work`, made empty and private with the run's own files
  (`backend/files.rs`) and removed with them. With a workspace, its arguments and
  directory are byte for byte today's.
- The summary session (`backend/compact.rs`) of a run with no workspace takes the same
  three changes; with a workspace, as today. A resume keeps the run's arguments
  (`--resume` in place of `--session-id`, as today).
- The fake program (`smith/tests/fake-claude`) accepts `--system-prompt-file PATH`, read
  as `--append-system-prompt-file`'s file is (cues included), and
  `--disable-slash-commands`, a flag with no value; it refuses both prompt flags at
  once, and neither, as it refuses an unknown flag (exit 2). Its observation records its
  working directory and whether that directory held any entry when it started, beside
  its arguments (`story.rs`, `Observation`; `fake.rs`).
- Rows, not new tests, but where a test sketched below has no table to join:
  - `smith/tests/fake-claude/tests/suite/turns.rs`, `refused_flags_exit_2`: both prompt
    flags at once; neither. `optional_flags_accept_values_in_any_order`: the two new
    flags among the others.
  - `smith/crates/smith-protocol-program/src/backend/tests.rs` (or `session_tests.rs`
    for the summary): one table, rows "no workspace" and "a workspace", each
    asserting the prompt flag, `--tools`' value, the presence or absence of
    `--disable-slash-commands` and the spawn's directory, for the ordinary start and
    for the summary session.
  - `smith/tests/agent-process/tests/suite/program.rs`,
    `a_program_grant_starts_claude_with_its_command`: its world has no workspace
    (`smith/tests/agent-process/src/lib.rs`, the no-workspace fixture), so its
    assertions move to the replaced prompt, `--disable-slash-commands`, and the fake's
    observed working directory `.../work`, empty at start. Its suite is in the focused
    profile (`smith-agent-process-world`, not its `live` binary): run it by name.
- `smith/docs/design/protocol/programs.status.md`: delete the bullet's first sentence
  "5.1, 5.3, 8: a run with no workspace keeps Claude Code's prompt ... nor
  `--disable-slash-commands`." and nothing else (a departure from the implementer's
  text, which closes `docs/design/`: the status must say what is built).

## Inputs

- `temper/docs/plan/ui-ask-m1/notes/common.md`; `notes/contracts.md`, section 4.
- `smith/docs/design/protocol/programs.md`, 5.1 ("A run with no workspace replaces Claude
  Code's prompt", "Where it runs", "The run's own files"), 5.3 (`--tools`), 5.8 (the
  summary session), section 8 (the fake program, its observations and stories).
- `temper/docs/design/draft/ask-temper.md`, 4.4, for why.

## Owns and may change

- Owns the files in the header.
- Readers: `smith/crates/smith-protocol-program/src/backend/fixtures.rs` and
  `smith/crates/smith-agent-service/src/service/program.rs` construct
  `backend::Settings`; touch them only if a field the compiler needs moves (none is
  expected: the change reads `workspace`, which exists).
- `backend/redaction.rs` redacts `directory + .runs/<32 hex>/instructions.md`; if the
  working directory's path must be redacted the same way where it shows, extend it there.
- Pre-approved: if the run's own files are written in a step order that cannot make
  `work` before the launch, add a step to `files.rs` that makes it (`OpenPrivate`'s
  sibling for a directory, as `.runs` is made) and say so. If `checked_init`
  (`backend/stream.rs`) refuses a `system/init` whose `slash_commands` is empty or whose
  tools are none with no workspace, relax only that check for a run with no workspace,
  and say so.

## Steps

1. Write the rows above; run them; see them fail for the rule's reason.
2. `start.rs`: decide by `self.settings.workspace.is_none()`; `arguments` gains what it
   needs to place the replaced prompt and the flag (a parameter or a small enum of your
   naming, inside the module).
3. `compact.rs`: the same for the summary session.
4. `files.rs`: make `work` with the run's files; remove it with them.
5. The fake program's flags and observation.
6. Narrow tests: `heavy cargo nextest run -p smith-protocol-program --lib`, the
   fake-claude suite (`heavy cargo nextest run -p <fake-claude's package> --test
   suite`, package `smith-fake-claude`), the agent-process test by name
   (`heavy cargo nextest run -p smith-agent-process-world -E
   'test(=a_program_grant_starts_claude_with_its_command)'`, and
   `program_resume.rs`'s compaction test), then `cargo fmt`, `python3 -I -B
   smith/scripts/old_forms.py` (if the tree has it), `heavy cargo clippy -p
   smith-protocol-program -p smith-fake-claude -p smith-agent-process-world
   --all-targets -- -D warnings`. Never the `live` binary.
7. Commit and push.

## Report

As the implementer's text says; name the argument order you chose, and where `work` is
made and removed.

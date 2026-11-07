+++
id = "tui-status"
part = "tui"
kind = "docs"
needs = ["tui-view", "tui-goldens", "tui-host", "tui-world", "tui-run", "tui-command"]
files = [
  "temper/docs/design/client/README.status.md",
  "temper/docs/design/protocol/README.status.md",
  "temper/docs/design/domain/testing.status.md",
  "temper/docs/design/draft/tui.execution.md",
]
effort = "medium"
minutes = 25
+++

# The TUI's status entries, once its code has merged

## Goal

The temper tasks of part `tui` ran side by side and touched no status
file, so that their branches never met there. Now that all have merged,
the **Not yet** entries their code closes leave the status files, and the
TUI's milestone says it is built.

This task departs from the implementer's text in two ways: the status
files and one line of a draft's execution file are yours (`docs/design/`
is otherwise closed), and it changes no code.

Acceptance:

1. `temper/docs/design/client/README.status.md`: its last entry, the
   **Not yet** beginning "2: the crates `temper-client-tui` and
   `temper-client-shell-tui`.", loses each part the trunk now builds,
   checked against the code (below); the entry goes whole if every part
   is built. The rest of the file is unchanged.
2. `temper/docs/design/protocol/README.status.md`: the **Not yet**
   beginning "4: `temper [ADDRESS]`, the TUI" goes, if
   `temper/crates/temper/src/commands.rs` has the row and bare `temper`
   runs the TUI in a terminal. The other entry for 4 is unchanged.
3. `temper/docs/design/domain/testing.status.md`: the **Not yet** for
   6.3, "the TUI's host does not run in the joint world", goes, if
   `temper/tests/client/tests/suite/tui.rs` holds the three stories.
4. `temper/docs/design/draft/tui.execution.md`, section 3: M1's state
   reads "built in `tui-connected`; its walk follows the landing".
5. `python3 -I -B tools/citations/citations.py --base origin/main`, run
   on your commit (it reads committed trees), finds nothing new broken.

## Inputs

- `temper/docs/plan/tui-connected/notes/common.md`.
- `temper/docs/plan/tui-connected/notes/contracts.md` and
  `temper/docs/plan/tui-connected/notes/tui.md`: the names the code holds.
- `DESIGN.md`, section 2 (a status file records only the gaps as they
  stand now).
- `temper/docs/design/client/README.md`, 6.11 to 6.16 and section 10's
  two TUI bullets: what each part of the entry names.
- The merged tasks' reports, where the lead gives them: a part one
  reports left (`## Left`) stays.

## Owns and may change

**Owned:** the `files` above, each only as acceptance says.

**Readers and neighbours:** none of these files is read by a test
(`.config/gate/read-by-tests.txt`).

**Options, each with its default:**

- **What each part of the client's entry needs, to go:**
  - 2: both crates are workspace members;
  - 6.11: `commands.rs`'s row with no words, `Command::Tui`, and
    `temper/crates/temper/src/tui.rs`'s `run`;
  - 6.12: `tui.rs`'s `directory`, `kept`, and its `store_file` of the
    address;
  - 6.13: `temper-client-shell-tui`'s `host`;
  - 6.14: `temper-client-tui`'s `state`, `words` and `view`;
  - 6.15: `temper-client-tui`'s `theme`, and `tui.rs`'s `scheme` and
    `depth`;
  - 6.16: `temper-client-shell-tui`'s `run`, and `tui.rs`'s raw input,
    `before_report` and `restore`;
  - 10: the goldens in `temper/crates/temper-client-tui/src/tests/golden/`,
    the host's rows, the run's machine story
    (`temper/crates/temper-client-shell-tui/tests/`), the binary's rows,
    and the joint world's three stories.
  A part whose code is absent stays, reworded only to say what is still
  absent.
- A gap you find that no entry names (a part built otherwise than the
  design says): add a **Departs** entry for it with the reason the
  task's report gives, and say so; never change the design.

## Steps

1. Read each part's code on the trunk, by `rg` for the names above.
2. The entries (acceptance 1 to 3) and M1's state (acceptance 4).
3. Commit ("temper: the TUI's status entries once its first pass is
   built"); `python3 -I -B tools/citations/citations.py --base
   origin/main` on it (fix and amend what it lists); push.

## Report

As the implementer's text says, and: each entry removed or kept, with the
code that closed it or the part still absent.

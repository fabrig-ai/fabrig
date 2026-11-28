# A project's charters: contracts between the goal's changes

2026-11-28, goal 21395 (`../../draft/ways.execution.md`, W3). The shapes
the goal's changes share and no other area reads, binding as any
delivered design section (`../../draft/design-economy.md`, 4.3). The
lasting design is `project-charters.md`,
`jig/docs/design/domain/engine/project-charters.md` and
`../../protocol/temper-dir.md`, 3.3 and 8; this document goes with the
goal's last change. Names are binding; a field's exact type is the
implementer's where this says "text" or "a name", in the type the
crate's neighbours use for the same thing.

## 1. A charter file's reader and writer

In `temper-charter-set`, step code, no generic item:

- `read_charter(text: &str, reasons: &[&str]) -> Result<CharterFile, CharterFault>`:
  one file's text read by `../../protocol/temper-dir.md`, 3.3. `reasons`
  is the names a part may be selected by, in jig's order (jig's
  `Reasons` names); a heading naming another is a fault.
  - `CharterFile`: `format: u32`; `settings: Option<…>`, the front
    matter's name or none; `instructions`, text; `parts`, each a
    `FilePart`: `line: u32`, its heading's line from 1; `reasons`, its
    names in the heading's order; `text`.
  - `CharterFault`: `line: u32` (0 for the whole file) and its words,
    3.3's, rendered by `Display`; the first fault alone.
- `write_charter(instructions: &str, parts: &[WritePart]) -> Result<String, CharterFault>`,
  `WritePart { reasons: &[&str], text: &str }`: the file 3.3's
  *Written back the same* gives, which `read_charter` reads back to the
  same instructions and parts; refused, naming the part and the line,
  where a text holds a line beginning `## part, on:`.
- `read_file` stays for an overlay's files, taking `format` alone and
  refusing `settings` (3.3).

## 2. jig's project sets

In `jig-domain-core-vocabulary` and `jig-domain-core`:

- **The input:** `ProjectCharters { project: u32, revision: u64, source, charters }`,
  `source` text at most 256 bytes, `charters` a list of
  `ProjectCharter { name, settings, instructions, parts }`, `name` and
  `settings` names (`jig-domain-people`'s `is_name`), `instructions` text,
  `parts` a list of the vocabulary's `CharterPart`. The root steps it as
  a core event, `Event::ProjectCharters`, in the decision it makes; the
  core's refusal comes back to the root as an output naming the project,
  the revision and the fault (jig's design, 6).
- **The newest:** `Core::project_charters(project: u32) -> u64`, the
  revision of the project's newest set, 0 where it has none.
- **A task's fields,** in `jig-domain-tasks`' `TaskRecord`: `charters: u64`,
  the set it records (jig's design, 4), and `scope`, the project's
  charter's name it is bound to, or none. A creation's charter scope is
  `New::charter_scope: CharterScope`, `Project` (the default) or
  `Deployment`, in `jig-domain-contract`.

## 3. A charter's read with a project

In `jig-client-domain`'s contract: `ReadSetting` gains
`project: Option<u32>`; `SettingDocument` gains `project`, none or a
`ProjectCharterDocument { name, revision: u64, source, settings, instructions, parts }`,
`parts` the contract's charter parts (jig's design, 7).

## 4. A run's guides

- **The input:** the root steps `Event::RunGuides { task: u64, attempt: u64, guides }`,
  `guides` a list of `Guide { repository, commit, path, digest: [u8; 32], bytes: u32, whole: bool }`,
  `repository`, `commit` and `path` text of at most 256 bytes each (jig's
  `domain/tasks/lifecycle.md`, 12.1).
- **The record:** `AttemptRow`'s `sources: Option<RunSources>`,
  `RunSources { build, settings: u32, charters: u64, source, guides: Option<list of Guide> }`,
  and the client contract's `AttemptLine` the same field.
- **The build:** jig's core `Config` gains `build`, text of at most 64
  bytes, the application's build name.

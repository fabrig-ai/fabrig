# configuring-p4: contracts

The shapes that cross this plan's tasks, binding on names; the design
(`temper/docs/design/domain/testing.md`, 6.5) wins on detail. One crate,
`temper/tools/stage-one`, package `temper-stage-one`, a library and one binary of the same name.

## K1. The crate's layout

```
temper/tools/stage-one/
  Cargo.toml            package temper-stage-one; [[bin]] temper-stage-one, src/main.rs, test = false
  clippy.toml           a copy of temper/tools/journeys/clippy.toml
  golden/reference.json the golden (K2)
  src/lib.rs            pub mod golden; pub mod evidence; pub mod check; pub mod judge; pub mod collect;
  src/main.rs           the commands (K5), each a call into the library
  src/golden.rs         K2's types, decode, reference(), profile()
  src/evidence.rs       K3's types and read()
  src/check.rs          K4's check
  src/judge.rs          K6's brief, the verdict's decoder, the stage's verdict
  src/collect.rs        K7's collect and assemble
  src/tests.rs          mod golden; mod evidence; mod check; mod judge; mod collect;
  src/tests/<module>.rs each module's tests
  src/tests/fixtures/   evidence directories and ctl outputs the tests read
```

Dependencies: `serde_json` (workspace), `serde` with `derive` (workspace) and `temper-settings-form`
(workspace) only; dev-dependencies `skein-scratch` (workspace) for temporary directories.

Each command is a function of its module, `pub fn main(args: &[String]) -> std::process::ExitCode`
(`golden::main` for `obligations`, `collect::main`, `check::main`, `judge::main` for `judge`,
`judge::verdict_main` for `verdict`), `args` the words after the command; `src/main.rs` only
dispatches on the first word, and prints the usage, exit 2, for anything else. `stage-one-surface`
writes `src/main.rs` whole and each of `check.rs`, `judge.rs` and `collect.rs` as a stub whose
`main` (and `verdict_main`) prints `temper-stage-one: COMMAND is not built` and exits 2; the task
that builds a module replaces its stub, file and tests file whole, and edits neither `lib.rs`,
`main.rs` nor `tests.rs`.

## K2. The golden

`golden/reference.json`, embedded with `include_bytes!` by `golden::reference()`; the profile's
`temper/docs/examples/reference/settings.json` embedded by `golden::profile()`. Unknown members are
refused naming their path.

```json
{
  "golden": "temper/docs/design/draft/reference-code-production.md, 14",
  "placeholders": {"forge": "127.0.0.1:3000", "repository": "acme/widget"},
  "roles": [ROLE, ...],
  "obligations": [OBLIGATION, ...],
  "rules": [RULE, ...]
}
```

- ROLE: `{"role": NAME, "kind": "charter"|"template"|"landing-rule", "profile": N, "anchor": ANCHOR|null}`;
  NAME is one of 6.5's roles (`coordinator`, `designer`, `coding`, `foreman`, `triage`,
  `goal-template`, `small-goal-template`, `main-rule`, `goal-rule`); `profile` the number of its
  item in the profile's settings file (a stock number where the profile holds none);
  ANCHOR one of `{"executes": TEMPLATE-ROLE}` (the charter that template's `executor.charter`
  names), `{"forge-charter": N}` (the charter kind 3's item N names), `{"number": N}` (that
  number: the foreman's 6, the stock's goal template 2), `{"resource":
  "goal-branch"|"none"}` (a template by its `resources`), `{"branch": {"exact": TEXT}}` or
  `{"branch": {"open": TEXT}}` (a deployment's landing rule by its pattern's last segment), or
  null (bound by the check's assignment, 6.5).
- OBLIGATION: `{"id": ID, "row": ROW, "kind": "setting"|"box"|"gap", ...}`; ID dotted and unique
  (`coordinator.model`), ROW section 14's first column word for word. By kind:
  - `"setting"`: `"subject"`: `{"role": NAME}` or `{"setting": SETTING}` (SETTING as a settings
    file names one, `../client/tools.md`, 5.3, a project's as `{"kind": ..., "project":
    "box"}`, the box's project); `"member"`: a dotted path in the value (`model.name`; `""` the
    value whole); `"test"`: `"equal"`, `"at-least"`, `"bits"`, `"contains"`, `"set"`, `"judged"`,
    `"unchanged"` or `"only"` (the subject a kind without a number: every current setting of it
    binds to a role);
    `"value"`: the expected value, left out where the profile's item for the role holds it. In
    a value, `{"role": NAME}` stands for that role's bound number.
  - `"box"`: `"prerequisite"`: its words; `"confirm"`: `"ctl"`, `"forge"` or `"document"`.
  - `"gap"`: `"status"`: the status entry it cites (path, section); `"names"`: a list of
    alternatives, each a list of words that must all appear, case folded, in one of the texts.
- RULE: `{"id": ID, "role": NAME, "words": TEXT, "source": TEXT}`: a rule of the reference that
  stays text (6.5), the role whose runs it must reach, and where the person wrote it.

```rust
pub struct Golden { pub placeholders: Placeholders, pub roles: Vec<Role>, pub obligations: Vec<Obligation>, pub rules: Vec<Rule> }
pub struct Placeholders { pub forge: String, pub repository: String }
pub struct Role { pub role: String, pub kind: RoleKind, pub profile: u32, pub anchor: Option<Anchor> }
pub enum RoleKind { Charter, Template, LandingRule }
pub enum Anchor { Executes(String), ForgeCharter(u32), Number(u32), Resource(ResourceAnchor), Branch(BranchAnchor) }
pub enum ResourceAnchor { GoalBranch, None }
pub enum BranchAnchor { Exact(String), Open(String) }
pub struct Obligation { pub id: String, pub row: String, pub kind: ObligationKind }
pub enum ObligationKind { Setting(SettingObligation), Box(BoxObligation), Gap(GapObligation) }
pub struct SettingObligation { pub subject: Subject, pub member: String, pub test: Test, pub value: Option<serde_json::Value> }
pub enum Subject { Role(String), Setting(SettingKey) }
pub enum Test { Equal, AtLeast, Bits, Contains, Set, Judged, Unchanged, Only }
pub struct BoxObligation { pub prerequisite: String, pub confirm: Confirm }
pub enum Confirm { Ctl, Forge, Document }
pub struct GapObligation { pub status: String, pub names: Vec<Vec<String>> }
pub struct Rule { pub id: String, pub role: String, pub words: String, pub source: String }

/// A setting as a settings file names it; `project` is `Some(..)` for a project's kinds.
pub struct SettingKey { pub kind: String, pub number: Option<u32>, pub project: Option<ProjectRef> }
pub enum ProjectRef { Box, Number(u32) }

/// A settings file (`temper/docs/design/client/tools.md`, 5.3), values kept as JSON.
pub struct SettingsFile { pub items: Vec<Item> }
pub struct Item { pub setting: SettingKey, pub edit: Edit }
pub enum Edit { Set(serde_json::Value), Removed, Reset }

pub fn decode(bytes: &[u8]) -> Result<Golden, GoldenError>;
pub fn reference() -> Result<Golden, GoldenError>;
pub fn decode_settings(bytes: &[u8]) -> Result<SettingsFile, GoldenError>;
pub fn profile() -> Result<SettingsFile, GoldenError>;
pub struct GoldenError { pub path: String, pub reason: String }   // Display: "PATH: REASON"
```

## K3. The evidence

One directory a pass writes (K7 writes all but the last four), read by `evidence::read(dir)`:

| File | Holds |
|---|---|
| `box.json` | `{"forge": "HOST:PORT", "repository": "OWNER/NAME", "project": P, "revision": R, "proposal": "tN/M", "chat": T}` |
| `revision.json` | `temper ctl DIR revision show R --json`, as printed |
| `items.json` | the accepted revision's items, a settings file (K2's `SettingsFile`), each set value whole |
| `current.json` | a settings file of every setting's current value after acceptance: each `settings show` lists, read whole with `settings show KIND [NUMBER] --json`, temper's kinds included, and the project's policy as `{"kind": "project-policy", "project": P}` |
| `proposal.txt` | `temper ctl DIR proposal show PROPOSAL`, text |
| `chat.txt` | `temper ctl DIR chat show T`, text, every page, oldest first |
| `confirmed.json` | the pass's: `{"box": [{"id": ID, "present": BOOL, "seen": TEXT}, ...]}`, one per box obligation |
| `description.txt` | the pass's: the agent's answer, after acceptance, describing the settings back as a workflow |
| `documents/` | the pass's: each document an accepted charter names, at its branch's head, as `documents/OWNER/NAME/PATH`; the person's two process documents as `person/reference-code-production.md` and `person/reference-landing.md` |
| `judge.json` | the pass's: the judge's verdict (K6) |

```rust
pub struct Evidence { pub facts: BoxFacts, pub revision: serde_json::Value, pub items: SettingsFile, pub current: SettingsFile,
                      pub proposal: String, pub chat: String, pub confirmed: Vec<Confirmation>, pub description: Option<String>,
                      pub documents: BTreeMap<String, String>, pub person: BTreeMap<String, String>, pub judge: Option<serde_json::Value> }
pub struct BoxFacts { pub forge: String, pub repository: String, pub project: u32, pub revision: u32, pub proposal: String, pub chat: u32 }
pub struct Confirmation { pub id: String, pub present: bool, pub seen: String }
pub fn read(dir: &Path) -> Result<Evidence, EvidenceError>;   // Display names the file and what fails
```

`description.txt`, `documents/`, `person/` and `judge.json` may be absent: `check` does not
read them; `judge` and `verdict` say which is missing.

## K4. The check

```rust
pub struct Report { pub binding: Vec<Binding>, pub verdicts: Vec<Verdict>, pub extras: Vec<Extra> }
pub struct Binding { pub role: String, pub number: Option<u32>, pub by: BoundBy }
pub enum BoundBy { Anchor, Assignment, Unbound }
pub struct Verdict { pub id: String, pub row: String, pub outcome: Outcome }
pub enum Outcome { Met, Wrong { found: serde_json::Value, expected: serde_json::Value }, Missing, Judged,
                   Unchanged, Named { text: TextName }, Unnamed, Confirmed, Lacking { seen: String } }
pub enum TextName { Proposal, Chat }
pub struct Extra { pub setting: SettingKey, pub why: String }
pub fn check(golden: &Golden, profile: &SettingsFile, evidence: &Evidence) -> Report;
impl Report { pub fn passed(&self) -> bool; pub fn text(&self) -> String; pub fn json(&self) -> serde_json::Value; }
```

`passed` is true when every verdict is `Met`, `Judged`, `Unchanged`, `Named` or `Confirmed` and
`extras` is empty.

## K5. The commands

```
temper-stage-one obligations [--json]
temper-stage-one collect --temper PATH --ctl DIR --project P --revision R --proposal PROPOSAL
                         --chat TASK --forge HOST:PORT --repository OWNER/NAME EVIDENCE
temper-stage-one check EVIDENCE [--json]
temper-stage-one judge EVIDENCE
temper-stage-one verdict EVIDENCE [--json]
```

Exit 0: done, or passed (`check`, `verdict`); 1: failed (`check`, `verdict`); 2: a usage error,
an unreadable golden or evidence, or a `temper ctl` call that failed (`collect`), with one line on
stderr naming it. `--json` prints one JSON object on stdout.

## K6. The judge

- `judge::brief(golden, evidence, report) -> Result<String, JudgeError>`: the judge's brief,
  Markdown, whole (6.5), naming what of the evidence it lacks as an error.
- The verdict the judge session writes, `judge.json`:

  ```json
  {"model": "MODEL",
   "rules": [{"rule": ID, "verdict": "reaches"|"missing"|"elsewhere", "where": TEXT, "quote": TEXT}],
   "description": {"verdict": "matches"|"differs", "differences": [TEXT]},
   "restart": {"verdict": "said"|"proposed", "quote": TEXT}}
  ```

  one entry per golden rule, each rule once.
- `judge::decode_verdict(bytes, golden) -> Result<JudgeVerdict, JudgeError>`, refusing an unknown
  member, a rule the golden lacks, one missing or twice.
- `judge::stage(report: &Report, verdict: &JudgeVerdict) -> Stage`, `pub struct Stage { pub passed:
  bool, pub reasons: Vec<String> }`: passed when the report passed, every rule `reaches`, the
  description `matches` and the restart `said`.

## K7. Collect

- `collect::collect(args: &CollectArgs) -> Result<(), CollectError>` runs `PATH ctl DIR ... --json`
  (and the two text reads without `--json`) with stdin null, writes K3's files it owns and
  creates the directory's `documents/` and `person/` empty.
- `collect::assemble(outputs: &CtlOutputs) -> Result<(SettingsFile, SettingsFile), CollectError>`,
  pure: from the ctl outputs read (the revision's record; each `settings show ... --json` value
  by its setting) to `items.json` and `current.json`.

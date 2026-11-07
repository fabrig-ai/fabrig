# per-wake-temper: the shapes that cross tasks

The run's binding calls. Every task builds to these names, members, fields and words;
inside them the choices are its own. The design is `temper/docs/design/client/tools.md`,
5.3 ("A charter's parts and a later choice") and `temper/docs/design/protocol/configuration.md`,
13.9 ("Charter 5 in parts", "Each charter holds what its role acts on", "Checked as it is
read"). jig's side is landed: `jig_client_domain::CharterPart { on: u32, instructions }`,
`Charter.parts`, `LlmLater { main, subagents }`, `LlmChoice.later`, `REASON_NAMES`; the
wire's `CharterParts.parts` and `LlmChoice.later` (`jig-client-protocol-wire`, v19);
`jig_domain_tasks::{Reasons, REASON_NAMES, ALWAYS_OFFERED_NAMES}`.

## 1. The form's view (`temper/crates/temper-settings-form/src/view.rs`)

```rust
/// The store names of `first` and the kinds that choose parts, in bit order: bit `i` of a
/// part's `on` is `REASON_NAMES[i]` (temper/docs/design/client/tools.md, 5.3).
pub const REASON_NAMES: [&[u8]; 14] = [b"first", b"question", b"question_answered",
    b"question_ended", b"question_owed", b"escalation", b"proposal", b"person_stalled",
    b"result", b"notice", b"nothing_awaited", b"news", b"timer", b"words"];
/// The kinds every run is offered, which a part's `on` refuses.
pub const ALWAYS_OFFERED_NAMES: [&[u8]; 7] = [b"answer", b"proposal_decision",
    b"settings_decision", b"effect_not_made", b"met", b"amendment", b"finish_requested"];

/// A charter's part (temper/docs/design/client/tools.md, 5.3).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CharterPart {
    /// Bits of `REASON_NAMES`, as jig's client `CharterPart.on`.
    pub on: u32,
    pub instructions: Text,
}
// Charter gains, after `documents`:
pub parts: List<CharterPart>,

/// The choice every run after a task's first activation's takes.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct LlmLater {
    pub main: ModelEffort,
    pub subagents: List<ModelEffort>,
}
// LlmChoice gains, after `subagents`:
pub later: Option<LlmLater>,
```

Both name lists are pinned by a test in `temper-client-tools` to jig's
(`jig_client_domain::REASON_NAMES` and `jig_domain_tasks::ALWAYS_OFFERED_NAMES`), name by
name. The decoder's bound on a charter's parts is the wire's `limits.settings.charter_parts`
(`jig-client-protocol-wire`'s v19 `Limits`, 8), as `charter.documents` takes
`charter_documents`; its words name `tasks.charter_parts`.

## 2. The settings file's members (`settings_form::file`, `values.rs`)

- A charter's `parts`, optional, left out `[]`:
  `"parts": [{"on": ["first"], "instructions": "..."}, ...]`. `on` names in any order
  in the file; encoded in bit order. Encoded only when the charter has parts.
- A template's `llm` may hold `later`, optional:
  `"later": {"main": {"model": "M", "effort": "E"}, "subagents": [{"model": ..., "effort": ...}]}`,
  `subagents` optional, left out `[]`; no `account`. Encoded only when `Some`.
- Decoder refusals, in the decoder's own form `settings file item N: PATH: WHY`, indices
  from 0 as a document's are:

  | Case | PATH | WHY |
  |---|---|---|
  | a name not in either list | `charter.parts[J].on[K]` | `"NAME" is no reason` |
  | an always-offered name | `charter.parts[J].on[K]` | `"NAME" goes to every run and chooses no part` |
  | a name twice in one `on` | `charter.parts[J].on[K]` | `"NAME" twice` |
  | an empty `on` | `charter.parts[J].on` | `names no kind` |
  | more parts than `limits.settings.charter_parts` | `charter.parts` | the decoder's bound words, naming `tasks.charter_parts` |
  | parts with `"resume": true` | `charter.parts` | `a charter that resumes takes no parts` |
  | a part's instructions past the bound | `charter.parts[J].instructions` | as `charter.instructions` past it |
  | a `later` member out of shape | `llm.later.main...`, `llm.later.subagents...` | as `llm.main...` and `llm.subagents...` |

- An engine refusal of a charter with parts prints as any charter's does
  (`out of shape; item I; charter N: shape`): jig's refusal names no part.

## 3. What `settings show` prints in text (`render.rs`)

After the value's JSON, as the design's example has it:

- each part of a charter: `parts[J]: on NAME NAME; B bytes\n`, its `on` names in bit order
  separated by one space, `B` its instructions' bytes, `J` from 0;
- a template whose choice has a `later`: `later: MODEL EFFORT\n`, or `later: MODEL\n` with no
  effort.

## 4. The reference's charter 5 and template 4 (`temper/docs/examples/reference/settings.json`)

- Charter 5's `instructions` are the shared text; its `parts`, in this order:

  | Part | `on` |
  |---|---|
  | the cut | `["first"]` |
  | questions | `["question", "question_answered", "question_ended", "question_owed"]` |
  | results and holds | `["result", "notice", "nothing_awaited", "escalation", "proposal", "person_stalled"]` |
  | news | `["news", "timer"]` |

- Template 4's `llm`:
  `{"account": null, "main": {"model": "claude-opus-5-5", "effort": "xhigh"}, "subagents": [{"model": "claude-opus-5-5", "effort": "high"}], "later": {"main": {"model": "claude-opus-5-5", "effort": "high"}, "subagents": [{"model": "claude-opus-5-5", "effort": "high"}]}}`.
- A run's instructions are the shared text, then for each part its `on` meets, `\n\n` and
  the part's instructions (jig's `jig/docs/design/domain/tasks.md`, 8.6). So the cut reads
  `shared\n\ncut`, an answer `shared\n\nquestions`.

## 5. Who reads `settings.json`

No tool generates it. Its readers, each to keep green by the task that changes what it
reads:

| Reader | What it holds |
|---|---|
| `temper/crates/temper/src/tests/reference_settings.rs` | every charter's members; phrases of charter 5's, 2's, 6's, 7's and 8's instructions; template 4's `llm.main`; the profile's counts |
| `temper/crates/temper-client-tools/src/tests/jig/settings.rs`, `reference_goal_template_resources_survive_apply_and_show` | apply and show round trips, whole |
| `temper/crates/temper-service/src/tests/root_checks.rs`, `the_reference_room_starts_and_holds_its_relations` | charter models' output, template 4's report |
| `temper/crates/temper-service/src/tests/stock_brief.rs`, `reference_landing.rs` | charter 2's instructions in a brief; the landing rules |
| `temper/crates/temper-init/src/tests/documents.rs` | charter numbers, document lines |
| `temper/tests/engine/domain/tests/suite/configuring.rs`, `a_configuring_chat_proposes_large_settings_from_a_note` | copies five charters' `instructions` onto fixtures; its note body past 23,000 bytes |
| `temper/tests/engine/domain/tests/suite/host_calls.rs`, `forge_routes/goal_branch.rs` | charter 5 and template 4 in the domain world (template `llm` nulled) |
| `temper/tools/stage-one` (`src/golden.rs`, `src/judge.rs`; `golden/reference.json`; `src/tests/fixtures/judge/brief.md`) | template 4's `llm` whole; the judge's brief copies each charter's instructions whole |

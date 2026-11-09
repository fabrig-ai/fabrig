# ui-p3f-design: the path's "design" and "close-out"

Milestone P3f of `temper/docs/design/draft/whole-story.execution.md`: a goal's
template says whether its coordinator designs first and whether it makes a
close-out, and the one derivation draws "design" in place of "plan", and a
"close-out" after the landing, from that setting. Request: `inputs/request.md`.
`notes/common.md` holds the rules every task follows; `notes/contracts.md` the
shapes that cross tasks.

## Parts and tasks

| Part | Task | Needs | What it makes run | Minutes |
|---|---|---|---|---|
| `goal` | `tasks/goal-wire.md` | | `TemplateGoal` on the client wire and in the store, `PathStep`'s `design` and `close_out`, `SettingValue`'s `goal`, regenerated, nothing filled | 45 |
| `goal` | `tasks/goal-value.md` | `goal-wire` | the forge's kind 11: `TemplateGoal`, its judgement, `template_goal`, the store's and smith's codecs | 75 |
| `path` | `tasks/goal-path.md` | `goal-wire` | the derivation's "design" and "close-out", `Start::Goal`'s two settings, the service's two steps | 90 |
| `goal` | `tasks/goal-kind.md` | `goal-value` | kind 11 declared to jig, at the wire's entrances, `template-goal` in a run's `settings`, `see` and `propose_settings` and the settings form; an owner's change read back | 120 |
| `path` | `tasks/goal-read.md` | `goal-path`, `goal-value` | the root reads the goal template's kind 11, with a decision's items laid, into the path | 60 |
| `goal` | `tasks/goal-tools.md` | `goal-kind` | `temper ctl settings show template-goal N`, ctl's name, the help; a proposal card's name for kind 11 | 50 |
| `stock` | `tasks/goal-stock.md` | `goal-kind` | the stock's template 4: neither; the reference's: both | 60 |

About 500 implementer-minutes in all.

**Critical path:** `goal-wire` (45) → `goal-value` (75) → `goal-kind` (120) →
`goal-stock` or `goal-tools` (60), about five hours of implementer time.
`goal-path` runs beside `goal-value` from `goal-wire`'s merge, and `goal-read`
once both have merged, beside `goal-kind`.

**The request's tests, by task:** the derivation's rows for a goal template that
designs and closes out and for one that doesn't (`goal-path`'s derivation table,
`goal-read`'s root table over the settings held and laid); the stock's template 4
(`goal-stock`, read as below); the fuzz over random templates, never drawing
"design" without its setting, nor "close-out" without its (`goal-path`'s 64-seed
fuzzy test).

## What was decided, and why

- **The stock's template 4 plans and makes no close-out; the reference's designs
  and closes out.** The request's tests name "the stock's template 4 giving
  'design' and 'close-out'", which the drafts settle otherwise: `stock-ways.md`,
  section 4, has "plan" in the stock's and "design" and "close-out" where a
  charter does them "as the reference's do"; section 3.3 has the stock's charter
  5 with no designers; the whole story's `how.html` draws Draft first with "plan"
  and no close-out, and "Yours", a reference-like project, with both; and the
  look request's Draft first, which `stock-ways.md`, section 8, holds exactly, is
  "plan". The request reopens no draft, so its test is read as template 4's
  setting held by the stock (neither) and by the reference (both). The stock's
  charter 5 does write its status files in a last change, but into the goal's
  branch before the landing, among the goal's changes, not after it.
- **Kind 11, a temper kind at the deployment's scope, numbered by the
  template,** rather than a field of jig's `Template` or of kind 7: a template
  is jig's and what its coordinator does is temper's words for its pages, so it
  stays temper's beside the template by number, as P3e's kind 9 sits beside a
  charter; kind 7 is under change by ui-reaches. Numbered past kinds 8 to 10,
  which the two plans beside this one take.
- **Two bools, `design` and `close_out`,** as the request says the template says
  two things; neither is the value with none, so a deployment with no kind 11
  draws today's "plan".
- **The setting describes, the engine decides nothing by it:** the charter's
  instructions say what its runs do. That the two agree is the owner's, as a
  charter's words are (`configuration.md`, 13.2).
- **"Settings" is ctl's settings,** `settings show`, `apply` and `propose`, as
  P3e's words are: no page of the client draws a template's work, and no mockup
  has a control for it.
- **The close-out opens the coordinator's charter,** the template's, which makes
  it; a close-out's charter of its own is an open question with that default
  (`forge.md`, section 20), since the reference's is a designer's change and its
  card then names the coordinator's model.
- **Its cards' words** are the whole story's `how.html`'s: "an agent · designs
  the goal, then cuts it into changes" with "Its order", and "an agent · after the
  landing, brings what the goal leaves up to date" with "Lands". "Lands" says "by
  BRANCH's rules" where the mockup says "through main's CI and queue", since the
  rules may ask no CI and the landing may be by turn.
- **No words in Ask temper's instructions:** `ask.txt` is at its 8,192-byte bound
  once ui-reaches and ui-p3e-words land; Ask temper reaches kind 11 through its
  tools' declarations and `see settings template-goal N`.
- **Status entries:** every **Not yet** this plan's design opened is written now,
  one bullet per task, each deleted by the task that builds it.

## Who hears first

- **A stored format** (for the TUI's owner): the store's `store-v11.schema` gains
  `TemplateGoal` and `SettingValue`'s `goal`, last (`goal-wire`, `goal-value`); the
  forge's settings rows hold kind 11.
- **The client wire,** shared with the terminal: `client-wire-v1.schema`'s
  `PathStep` gains `design` and `close_out` at its end, and the wire a
  `TemplateGoal` record (`goal-wire`); `settings_kind` takes 11 (`goal-kind`). A
  path read may now carry the two steps; clients draw a stop's words, so the
  terminal prints them with no change.
- **`temper-client-domain`** (for the TUI's owner): no change.
- **`temper-engine-domain`** (for the owner of coding runs, whose coordinator
  charter the setting describes): `route/ways.rs`' `start` reads the goal
  template's kind 11 (`goal-path`, `goal-read`); `route/kinds.rs`' `bounds` gains
  `templates` (`goal-value`); `route/see.rs` and `route/host_route.rs` read kind
  11 (`goal-kind`). The template's work describes charter 5: a deployment that
  gives its coordinator designers, as the reference does, says so in kind 11.
- **`forge_route.rs`:** no change.

## Overlaps with drafts beside this one

- **ui-reaches' kind 8 and ui-p3e-words' kinds 9 and 10** walk the same files a
  new kind does: the forge's `settings.rs` and `kinds.rs`, the store's schema and
  `records/settings.rs`, the wire's schema and `lib.rs`, smith's `kinds.rs`,
  `settings.rs`, `see.rs` and `calls.rs`, the service's `stock/kinds.rs`,
  `stock.rs` (P3e's `charter-lines`) and `tests/stock.rs`, the settings form, the
  root's `route/kinds.rs`, `see.rs` and `host_route.rs`, ctl's settings, the
  proposal card and the help's references. Unavoidable: it is one mechanism.
  Each list gains its own lines; the store's `SettingValue` line is edited by all
  three, `goal` staying last; generated code and goldens regenerate on a
  catch-up.
- **The design files** were placed to merge: the cut test-merged this branch
  against each sibling trunk caught up with main, and none conflicts. Kind 11 is
  a bullet of its own at 13.2's end rather than a row of its table, whose rows
  and count the other two plans change; `agent.md`'s row sits apart from
  ui-reaches' `project-ways` row.

## Questions

None: the drafts settle the one reading the request leaves open (above).

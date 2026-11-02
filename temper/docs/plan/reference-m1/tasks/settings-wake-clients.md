+++
id = "settings-wake-clients"
part = "settings-wake"
kind = "feature"
needs = ["settings-wake-charter"]
files = ["jig/crates/jig-client-protocol-wire/schema/client-v19-part1.schema", "jig/crates/jig-client-protocol-wire/src/generated/**", "jig/crates/jig-client-protocol-wire/src/tests/**", "jig/crates/jig-client-protocol-wire/golden/**", "jig/crates/jig-client-domain/src/contract/settings.rs", "jig/crates/jig-client-protocol-link/src/translate/settings.rs", "jig/crates/jig-protocol-engine-clients/src/translate/settings.rs", "jig/crates/jig-protocol-engine-clients/src/translate/settings/change.rs", "jig/crates/jig-protocol-engine-clients/src/tests/settings.rs", "jig/testing/jig-client-scripted-engine/src/codec/values/settings.rs", "temper/crates/temper-settings-form/src/**", "temper/crates/temper-client-tools/src/jig/settings_adapter.rs", "temper/docs/help/reference/settings-fields.md"]
effort = "high"
minutes = 100
+++

# settings-wake-clients: a charter's wake policy in the clients' settings

## Goal

A person reads and sets a charter's default wake policy through the clients, as any of the
charter's members: the client wire's charter value carries it, `temper ctl settings show`
prints it, and a settings file's charter value takes it (`temper ctl settings apply`), so the
reference's coordinator charter can be set with C7's policy. Until now the wire's charter has
no wake, and a client's change of a charter gives it `WakePolicy::DEFAULT`
(`settings-wake-charter`'s interim in engine-clients).

Acceptance:

- The client wire's `record Charter` (client-v19-part1, in place) ends with
  `wake: WakePolicy` (client-v19's record); `jig_client::contract` settings' `Charter` has
  `wake: WakePolicy`; the link's, engine-clients' and the scripted engine's translations carry
  it both ways, and a client's change of a charter keeps the policy it names (the interim
  `DEFAULT` goes).
- A settings file's charter value takes a `wake` member, `null` or left out for jig's
  default, in the file's own forms (C7's line: durations in nanoseconds, a rule with data an
  object with one member): rules `"never"`, `"immediate"`, `{"batch": {"count": n, "age":
  ns}}` or `{"paced": ns}`; `results` `"never"`, `"each"` or `"last_or_failure"`;
  `questions`, `answers`, `timers` booleans. `settings show --json` prints the member whole,
  never null, so a printed value applies unchanged (`temper/docs/design/client/tools.md`,
  5.3: "VALUE is exactly the `value` member `settings show … --json` prints").
- A rule of an unknown form, or a member missing or unknown in `wake`, is refused before
  anything is sent, naming its path in the file (`charter.wake.news`), as the form's other
  refusals do; a zero count, age or span is sent, and the deployment refuses it as `shape`
  (`settings-wake-charter`).
- `FIELDS` has the rows below, and `temper/docs/help/reference/settings-fields.md` is exactly
  what they render.

### The rows, and the page's new text

In `temper-settings-form/src/fields/jig.rs` (and `Shape`'s new variants `Wake`, `WakeRule`,
`WakeBatch` in `fields.rs`):

- After the charter's `spend` `Field`: `Field { shape: Shape::Charter, name: b"wake", value:
  Value::Shape(Shape::Wake), list: None, null: true, line: b"Which messages wake a task made
  under the charter whose batch names no wake policy; null, or left out, for jig's default:
  every rule `\"immediate\"`, `results` `\"last_or_failure\"`, and questions, answers and
  timers waking." }` (the line's quotes as the file's other lines write them).
- Right after the `model` `Form` row, three `Form` rows: `wake` (`Choice::Every`, line "What
  wakes a task, kind of message by kind; a proposal, an escalation and a cancel always wake
  it."); `wake_rule` (`Choice::One` with the words `never`, "It waits for the task's next
  run.", and `immediate`, "It wakes the task at once."; line "How a kind of message wakes a
  task: a word, or an object with one member."); `wake_batch` (`Choice::Every`, line "A batch
  of messages that wakes a task once.").
- Their `Field` rows, after the `model` shape's own fields, in this order: `wake`'s `words`,
  `notices`, `news` (each `Value::Shape(Shape::WakeRule)`), `results` (`Value::Words`:
  `never`, `each`, `last_or_failure`), `questions`, `answers`, `timers` (booleans);
  `wake_rule`'s `batch` (`Value::Shape(Shape::WakeBatch)`) and `paced`
  (`Value::Nanoseconds`); `wake_batch`'s `count` (`Value::Integer(Width::U32)`) and `age`
  (`Value::Nanoseconds`); every one `null: false`, its line the page's below.

The page then gains, at the end of `## charter`'s list:

```
- `wake`: `wake`, or null. Which messages wake a task made under the charter whose batch names no wake policy; null, or left out, for jig's default: every rule `"immediate"`, `results` `"last_or_failure"`, and questions, answers and timers waking.
```

and, after `## model`'s section and before `## template`:

```
## wake

What wakes a task, kind of message by kind; a proposal, an escalation and a cancel always wake it.

- `words`: `wake_rule`. Words from tasks and from the deployment; a person's words always wake it at once.
- `notices`: `wake_rule`. Notices: a task it references done or held, a proposal passing through, a hold it waits for.
- `news`: `wake_rule`. News its connectors class as waking for it.
- `results`: `"never"`, `"each"` or `"last_or_failure"`. Its delegates' results. `"never"`: None wakes it. `"each"`: Each delegate's result wakes it. `"last_or_failure"`: The last delegate's result, or any failure.
- `questions`: true or false. Whether a delegate's question wakes it.
- `answers`: true or false. Whether an answer to its own question wakes it.
- `timers`: true or false. Whether its timers wake it.

## wake_rule

How a kind of message wakes a task: a word, or an object with one member.

- `"never"`: It waits for the task's next run.
- `"immediate"`: It wakes the task at once.
- `batch`: `wake_batch`. Once enough have arrived, or the oldest has waited long enough.
- `paced`: nanoseconds. At most once in this span from the task's last run's start; the first after a quiet span wakes it at once. Above zero, else refused as `shape`.

## wake_batch

A batch of messages that wakes a task once.

- `count`: integer, 0 to 4,294,967,295. Wake once this many have arrived; above zero, else refused as `shape`.
- `age`: nanoseconds. Or once the oldest has waited this long; above zero, else refused as `shape`.
```

each section separated from the next by one blank line, as the page's others are. The
renderer's own formatting wins where it differs from this text in spacing or punctuation it
adds itself (a bound's words, the list's markers): the words of each line are these. Say
in the report any difference the renderer made.

## Inputs

- `jig/docs/design/protocol/configuration.md`, section 3 (the charters row: its default wake
  policy) and section 10 (**A charter's default wake policy**: the spelling the settings file
  shares); `jig/docs/design/domain/tasks.md`, 8.3.
- `temper/docs/design/client/tools.md`, 5.3 (**Settings FILE is JSON**; **The form's fields
  are one table**); `temper/docs/design/protocol/configuration.md`, 13.4 (**Each stock
  charter's default wake policy**).
- `temper/docs/plan/reference-m1/notes/common.md`; `notes/contracts.md`, C7 and its line.

## Owns and may change

Owns the files in the header. The edits there:

- The wire schema and its regeneration (the client wire's `drift` in
  `jig-client-wire-world`'s suite, with `SKEIN_CODEGEN_WRITE=1`), goldens
  (`record_charter_*`) and fixtures.
- `temper-settings-form`: the charter's view (`view.rs`), its wire translation (`wire.rs`),
  the settings file's decoder (`settings_file_values.rs`), the `--json` encoder
  (`values.rs`), the rows (`fields.rs`) and any text render of a charter; one decoder
  function for a wake policy and one for a rule, used by the charter's.
- `temper/docs/help/reference/settings-fields.md`: a stated departure from the
  implementer's text ("Not yours: `docs/help/`"): this page is rendered from `FIELDS`;
  its new text is the one above, and nothing else of the page changes. The tests that read
  it: `temper`'s `the_page_is_what_fields_renders`
  (`temper/crates/temper/src/tests/settings_fields.rs`, which prints the page it expects
  on a mismatch) and `temper-docs`' own tests (`temper/crates/temper-docs/src/lib.rs`
  compiles it as the help page `settings-fields`).
- Compiler-forced fields of the new member in jig's client crates (`jig-client-view`,
  `jig-client-test-records`, `jig/tests/client*`) and temper's (`temper-client-tools`,
  `temper-client-view`), each the field alone.
- Other plans: reference-m7a changes the client crates on its trunk; keep each hunk to the
  field.

## Steps

1. The wire's member and its translations; the form's decoder, encoder and rows; the help
   page.
2. Tests:
   - `jig-protocol-engine-clients/src/tests/settings.rs`:
     `a_clients_charter_change_keeps_its_wake_policy` (a paced news rule and `results: each`
     reach the core's `Charter.wake`).
   - `temper-settings-form`'s unit tests (beside the charter's existing file tests):
     `a_settings_file_charter_names_its_wake_policy` (C7's coordinator policy decodes),
     `a_settings_file_charter_without_wake_takes_the_default`,
     `an_unknown_wake_rule_is_refused_naming_its_path` (`{"news": "often"}`, path
     `charter.wake.news`),
     `a_shown_charter_with_a_wake_policy_applies_unchanged` (encode, then decode: equal).
   Narrow: `heavy cargo nextest run -p temper-settings-form --lib -E 'test(/wake/)'`,
   `-p jig-protocol-engine-clients --lib -E 'test(/wake/)'`, and `-p temper --lib -E
   'test(/settings_fields/)'` and `heavy cargo nextest run -p temper-docs --lib` (check each
   count).
3. The implementer's text's checks: fmt, old forms, clippy on every package changed, the
   focused tests of `jig-client-protocol-wire`, `jig-client-protocol-link`,
   `jig-protocol-engine-clients`, `jig-client-wire-world`, `temper-settings-form`,
   `temper-client-tools` and `temper` (its `--lib` tests; never its `live*` binaries).
4. Commit ("Carry a charter's wake policy in the clients' settings") and push.

## Report

The implementer's text's report, plus: the rows added to `FIELDS`, and each compiler-forced
field by file.

# ui-ask-drop: the shapes that cross tasks

The run's binding calls. A task builds to these; a task that finds one wrong asks.

## The stock after the drop

- **Charters:** 1 (chat), 2 (coding), 4 (reviewer), 6 (foreman), 9 (status), 10
  (trial) and 11 (Ask temper), seven, in that order. **Templates:** 1 (`chat`), 2
  (`goal`) and 5 (`ask`), three, in that order. Number 3 is left unused as a charter's
  and as a template's, and nothing takes it: charter 11 and template 5 keep their
  numbers, which the client, `ask.txt`'s readers and the reference already name.
- **Executors the stock grants** (the rules' ceiling and policy 1, each role of policy 1
  keeping the charters and procedures and its own role, as today): charters 1, 2, 4,
  6, 9 and 11; procedures 2, 3 and 4; roles 0, 1 and 2. Twelve kinds, where there
  were thirteen. Generated settings elsewhere that grant charters by number grant
  1, 2, 4, 6, 9 and 11.
- **Tool masks, unchanged:** the rules' and policy 1's tools stay 3,975 (charter 11's
  `propose_settings` is the settings family's); charters 1 1,023, 2 514, 4 and 6 512,
  9 and 10 0, 11 3,072.
- **Charter 11's models:** `ASK_FIRST_CHOICES`, `claude-opus-5-5`, `gpt-6-astra`,
  `gpt-6.1-sol`, then the rest: `CONFIGURING_FIRST_CHOICES` renamed, its values kept
  (`temper/docs/design/draft/ask-temper.md`, section 9: charter 3's models).
- **Gone from `temper-service`'s stock:** `configuring_charter`, `configuring`,
  `CONFIGURING_TOOLS`, `CONFIGURING_FIRST_CHOICES`, the charter match's arm 3,
  `Executor::Charter(3)` in `ceiling`, and `stock/configuring.txt`.

## What stays

- **The settings family's tools for a charter of a deployment's own:** `settings`,
  `read_docs` and `propose_settings`' configuring form (with its `note` source) stay
  declared, as today, to any charter whose `tools` hold the settings family and not
  the asking bit (`temper/crates/temper-engine-smith/src/calls.rs`, `tools_for`). No
  stock charter holds them now; `temper/docs/help/reference/host-tools.md`, which
  renders every host tool, does not move.
- **Their tests** run a charter of the test world's own: in the engine's domain world,
  `SETTINGS_CHARTER: u32 = 12` with template `SETTINGS_TEMPLATE: u32 = 12`, named
  `settings`, a copy of stock charter 1 whose `tools` are 1,920 (settings, read, note
  and recall, as charter 3's were) and whose instructions are
  `b"Read and propose settings."`, offered by the world's rules and policy 1 as
  `moving_config`'s removable charter is (`configuring/fixture.rs`, `settings_config`).
  Only a test of those tools' own mechanics runs it: the `settings` read's offsets and
  pages, the `note` source of a proposal, `read_docs`, and a charter with no settings
  family refused.

## What moves to charter 11

- **The configuring acceptance's scripted runs** (`temper/docs/design/domain/testing.md`,
  6.4): every story whose subject is what the configuring chat does (an owner's proposal
  and its acceptance heard, a member's waiting, stale, rejected, moved, the inventory's
  people and offer, ways and questions at their stock and as stored, a read that finds
  nothing, approvals and the box's approval identities, a charter's documents named by a
  proposal) starts template 5 and runs charter 11.
- **Its calls:** a `settings` read becomes `see` with the address `settings`,
  `settings KIND N`, `settings rules`, `settings landing-rule N [project]`,
  `settings project-ways`, `settings project-questions`, `revisions` or `revision N`
  (`temper/docs/design/domain/agent.md`, 8.4.1); a `read_docs` read `see help NAME`; a
  `read_file` read `see file OWNER/NAME PATH`; `note` on the chat's document `document`;
  a `propose_settings` call the asking form, its items inline or in parts by `more`,
  with no `note`.
- **Constants in the engine's domain world:** `ASK_CHARTER: u32 = 11` and
  `ASK_TEMPLATE: u32 = 5` in `configuring/fixture.rs`, in place of `CONFIGURING`.
  In the joined client world (`temper/tests/client`), `Joint::with_asking` and
  `fixtures::asking_world`/`asking_agent` in place of `with_configuring`,
  `configuring_world` and `configuring_agent`, each starting template 5.
- **ctl:** `chat start --project PROJECT --template 5 TEXT` starts Ask temper;
  `--template 3` is refused as any template the project isn't offered.

## Words

- "Ask temper", for the chat, where a word said the configuring chat, the configuring
  agent, Configure or Configure temper. Its tag in Chats is "Ask temper".
- Code names: a type, function, module, file or test named for the configuring chat
  or `Configure` that now serves Ask temper is named `ask`/`asking` as its spec says.

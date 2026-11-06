# model-catalogue-1: the plan

From `inputs/request.md`: section 7's step 1 of `temper/docs/design/draft/model-catalogue.md`, leaving steps 2 and 3. The
trunk is `model-catalogue-1/integration`. The user reviews on waking: what this cut chose where a choice was open is
under *For the user's review*.

## Parts and tasks

| Task | Tree | Needs | Effort, minutes | What it lands |
|---|---|---|---|---|
| `effort-xhigh` | skein | | medium, 15 | Anthropic's dialect takes `xhigh`: it refused it |
| `effort-config` | jig | | high, 50 | `engine.llm.models[].effort`; a task given no choice takes it |
| `offer-surface` | jig, literals across | `effort-config` | high, 110 | `MeModel`'s `effort`, `shown`, `accounts`, `picker`; `MeProject`'s `defaults`; wire, goldens; nothing filled |
| `offer-engine` | jig | `offer-surface` | high, 150 | `Core::offer`, the previews, a task's `llm` none, `me` marked by accounts |
| `ctl-offer` | temper | `offer-surface` | high, 70 | `--model` takes the default effort; `model list` prints the offer |
| `picker-offer` | temper | `offer-surface` | high, 150 | the chooser draws the offer; a provider's connect or fix line; `resolved_default` on the line; the wait's words |
| `configure-picker` | temper | `picker-offer` | high, 90 | the chooser on Configure temper and a configuring chat; "Models by role" |
| `stock-seven` | temper | `effort-config`, `offer-surface`, `ctl-offer`, `picker-offer` | high, 200 | seven models, the stock's orders, C the costliest; every fixture it reaches; process runs |
| `one-account-offer` | temper | `stock-seven`, `offer-engine` | high, 70 | Claude-only, ChatGPT-only, both and none, on init's settings; process runs |
| `configuring-told` | temper | `offer-engine`, `stock-seven` | high, 70 | the inventory names the offer, and `configuring.txt` keeps the agent to it |

**Critical path:** `effort-config` → `offer-surface` → `picker-offer` → `stock-seven` → `one-account-offer` or
`configuring-told`: about 10 implementer-hours end to end at these estimates, 16 in all.

**Why this order.** `stock-seven` waits for `ctl-offer` and `picker-offer` because declaring efforts breaks both: `--model X`
with no effort is refused, and pressing a model leaves an invalid draft. Each client side builds to
`notes/contracts.md` and needs no engine work, so `offer-engine` runs beside them.

## What the surveys found that changed the cut

- **Budgets.** `reserve_floor` prices a charter's main alone. With Opus first and real prices, K would be 17,436, and a run
  on Astra or Fable reserves 21,792: a ChatGPT-only box's configuring chat, default Astra, would be held on budget. So C
  is the costliest model's (`temper/docs/design/protocol/configuration.md`, 13.4).
- **`Template` can't carry the preview:** `w::Template` and `c::Template` are also a setting's value. So previews sit beside
  them, in `MeProject.defaults`.
- **A worker's restart would empty the offer**, since `model_readiness` asks a host even of a key, and every worker starts
  awaiting its logins. So a direct source's account needs no host in the offer, and `account list --until usable` means
  offered. `model_readiness` is unchanged, and a run's check still reads it.
- **The largest turn** (13.8): by reading, T doesn't move, 16,803,810, since effort names are charged at a fixed bound.
  `stock-seven` runs `startup::turn_bounds` and says the figure.
- **The fixtures that hid the last move:** `temper/docs/plan/model-catalogue-1/notes/stock-readers.md` lists them, with the fake
  box, which refuses to build under three Anthropic models.

## For the user's review

Left open by the draft, and taken here:

1. **Prices are real, so budgets grow.** Thousandths of a dollar per million tokens: C 10,896, K 43,584 ($43.58), a chat's 24K
   1,046,016 ($1,046), a week's period 84,727,296 ($84,727). They count on a Claude subscription too, which no one bills by
   the token. Existing boxes keep their documents and their budgets.
2. **Cached input is priced as input,** since section 2 gives no figure, so a budget stops early rather than late.
3. **`picker` is a configuration member until step 2,** true when left out, so that "All models" has something to mean.
4. **`shown` is section 2's:** "Opus 5.5". The look's closed line says "Claude Opus 5.5". `fabrig/ui`'s landing settles which:
   the facts are `shown` and the provider.
5. **The configuring agent's charters still name every model** when it reads them, since a charter goes back whole in a
   proposal. The inventory's list and its instructions keep it to the offer, and a proposal isn't refused for naming
   another: stock charters' "rest" name them.
6. **A task funded by a project or another person** shows the person's own offer in its chooser, as section 3 has it; the
   run's check decides.
7. **`temper ctl --model` refuses a model the offer lacks** (`model not offered: X`), so a script can't start a chat on a model
   before its account is usable. A chat with no model named still starts, and waits.
8. **A model's `resolved_default` is each template's,** not one a project's: `temper/docs/design/draft/model-catalogue.status.md`.

## fabrig/qa's comparison

**No stock change reaches a stored charter:** a start skips a changed setting (`jig-domain-core`'s `settings/restart.rs`), and
`stock-seven` holds that with a story. **But a charter copies its model:** its value holds the model's prices and efforts, and
`settings/check.rs`'s `model_valid` requires them equal to `engine.llm`. So a charter written against the old figures, `"efforts":
[]` and 1/1/1 per 1,000 for `gpt-6.1-sol`, is **refused on a box from the new `temper init`**, at a settings check and at a start.
An existing box keeps its document and is unaffected. A comparison that applies saved charters to a fresh box must rewrite them
from that box's `settings` reads.

## For `fabrig/ui`'s landing of `model-picker`

`me` gives what its pages need: the offer by `accounts`, a heading per account; `efforts` and `effort`; `picker`, and how many
it leaves out; `defaults`. The heading's "your subscription" comes from the accounts document, which holds the person's own
accounts alone: a number it doesn't hold is someone's key, and takes its provider's name. Not in this step, so with nothing
to call: page 07's "Read again", "Add a model by name", the picker's switch and "new" (steps 2 and 3). Today's look keeps "Change
model" until the landing.

## Overlaps it could not avoid

Each hunk small, and named in its task's report:

- `temper-client-view` and `temper-client-domain`, `words.rs`, `pages/chat.rs`, `limits.rs`: `ui-walk-3/fix-flow`, `ui-walk-3/fix-look`,
  `look-round-12/ready-pane` (`picker-offer`, `configure-picker`). `fabrig/ui` has priority.
- `temper/tools/gallery/src/worlds.rs`: `ui-walk-3/fix-look`, `look-round-12/ready-pane` (`offer-surface`'s literal, `picker-offer`).
- `temper-fake/src/backends.rs`: `ui-walk-3/fix-flow` (`stock-seven`).
- `temper/docs/design/client/README.md` and its status: `chat-reading-c/reading-close` (this cut's design).
- `temper/crates/temper/tests/fake_backends/walk/`: `test-speed/walk-waits`: read by `one-account-offer`, edited only if a run needs it.
- `temper/crates/temper/src/tests/help.rs`: `test-speed/tier-threads`' `temper/crates/temper/src/tests/**` (`stock-seven`).

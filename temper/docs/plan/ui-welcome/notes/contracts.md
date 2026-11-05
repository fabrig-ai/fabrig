# ui-welcome: the shapes that cross tasks

Binding for every task of this plan. A task that creates one of these builds it exactly so; a task that
reads one reads it so.

## The welcome's domain (`temper_client_domain::pages::welcome`)

`frame` creates: `Step::WelcomeAsked` (`/welcome?at=welcome`), `StepMark`, `step_marks`, `forge_kept`,
`go_on`. `go_on` is every "Continue" and "Go on setting up": Ready with a usable model, else Model.

## The welcome's parts, as `temper-` names

| The mock's | `look` const | Class | First built by |
|---|---|---|---|
| `.forge-status`, `.done` | `WELCOME_FORGE_STATUS`, with `FLOW_DONE` | `temper-welcome-forge-status` | `forge-step` |
| `.spin` | `WELCOME_SPIN` | `temper-welcome-spin` | `forge-step` |
| `.acts` | `WELCOME_ACTS` | `temper-welcome-acts` | `forge-step` |
| `.panel` | `WELCOME_PANEL` | `temper-welcome-panel` | `model-step` |
| `.away-note` | `WELCOME_AWAY` | `temper-welcome-away` | `model-step` |
| `.numbered` | `WELCOME_NUMBERED` | `temper-welcome-numbered` | `model-step` |
| `.field-v` | `WELCOME_FIELD` | `temper-welcome-field` | `model-step` |
| `.btn.quiet` for "All ways" | `FLOW_QUIET`, under `.temper-welcome-acts` | | `model-step` |
| `.facts` | `FLOW_FACTS` | as it stands | `model-step` |
| `.cmd` | `WELCOME_CMD` | `temper-welcome-cmd` | `claude-way` |
| `.checklist .current .mark` | `WELCOME_CURRENT` on the item | `temper-welcome-current` | `ready-step` |

Copy each rule's declarations from `mock.css`, with the welcome's `--gf-*` tokens, under
`/* ==== ui: welcome ==== */`.

# serve-without-box: the plan

The welcome is the box's (`temper/docs/design/draft/serve-without-box.md`).

## Parts

- **document** (draft, 2): `application.welcome`, `temper init`'s bare document, `--box`, the help.
- **fact** (draft, 4): the deployment's read on the wire, served and read by the domain.
- **pages** (draft, 3): with no welcome, the inbox; the chats page; the Forge choices.

## Tasks

| Task | Part | Needs | What |
|---|---|---|---|
| `welcome-member` | document | | the member read at a start |
| `deployment-wire` | fact | | `Read::Deployment`, `DeploymentAnswer` |
| `init-box` | document | `welcome-member` | init bare, `--box`, the box's `welcome` |
| `help-box` | document | `init-box` | the help pages |
| `deployment-read` | fact | `welcome-member`, `deployment-wire`, `init-box` | `owned`, the read served, the playground's offer |
| `client-reads-deployment` | fact | `deployment-read` | the domain's read, every world answering |
| `welcome-inbox` | pages | `client-reads-deployment` | the inbox; the plan's meeting point |
| `no-project-page` | pages | `client-reads-deployment` | the chats page's words |
| `forge-choices` | pages | `client-reads-deployment` | Forgejo on this machine |

## Critical path

`welcome-member`, `init-box`, `deployment-read`, `client-reads-deployment`, `welcome-inbox`: about
five and a half hours of implementer time; `deployment-wire`, `help-box`, `no-project-page` and
`forge-choices` run beside it.

## The order's reason

`client-reads-deployment` makes the welcome wait for the fact with every world answering
`welcome: true`, so that nothing a person sees moves until `welcome-inbox`. `init-box` gives the
three documents that show the welcome their member before anything reads it, and `deployment-read`
gates the playground only once they have it.

## Overlaps not avoided

- `temper-service`'s `family.rs`, `clients.rs`, `system.rs` beside `debt-7`'s host moves
  (`temper-service/src/**`): `deployment-read` keeps to contract K4's lines. It waits on `init-box`,
  which keeps it later than the first tasks.
- `temper/crates/temper/tests/{process.rs,suite/startup.rs}` beside `live-login-guard/guard`: one
  `--box` word each in `init-box`.
- `temper/docs/help/box.md` and `reference/**` beside `configuring-walks` and `reference-walks`
  (queued): `help-box` stays small and goes last in its part.

## A question put to the user

`forge-offer` (`plan.toml`): the draft's section 4 cannot offer Forgejo on this machine from the
rows on a fresh box, whose store has none. Default taken; nothing waits.

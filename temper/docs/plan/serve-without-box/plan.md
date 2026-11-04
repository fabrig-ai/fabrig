# serve-without-box: the plan

The welcome is the box's (`temper/docs/design/draft/serve-without-box.md`). Released in batches.

## Parts

- **document** (draft, 2): `application.welcome`, `temper init`'s bare document, `--box`.
- **fact** (draft, 4): the deployment's read on the wire, served and read by the domain.

## Tasks so far

- `welcome-member`: the member read at a start, to `settings::Engine`.
- `deployment-wire`: `Read::Deployment` and `DeploymentAnswer` on the wire.
- `init-box`: `temper init` bare, `--box`, the box's `welcome`, the readers' `--box`.

## Critical path, as it stands

`welcome-member`, then `init-box`; `deployment-wire`; then the batches still being cut: the read
served, the domain's decision, the pages.

## Overlaps

- `temper-service/src/family.rs` beside `debt-7`'s moves of `temper-service/src/**`: one arm here.
- `temper/crates/temper/tests/{process.rs,suite/startup.rs}` beside `live-login-guard/guard`: one
  `--box` word each in `init-box`.

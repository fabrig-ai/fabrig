# serve-without-box: contracts

The shapes that cross tasks, binding. Numbered; never renumbered.

## K1. The document's member

`application.welcome`, a boolean or absent (`temper/docs/design/protocol/configuration.md`, 3):
`true` shows the welcome; `false` or absent, none.

- `temper_service::startup::application::Members.welcome: bool`, and
  `temper_service::settings::Engine.welcome: bool`, its value as the start read it.
- Its row in `MEMBERS`: shape `application`, name `welcome`, kind `boolean`, not required, line
  `Whether the document shows the welcome and offers the playground.`

## K2. The wire

`temper/crates/temper-client-protocol-wire/schema/client-wire-v1.schema`, in place
(`temper/docs/design/client/README.md`, 4.6):

```
enum Read { playground, setting: SettingRead, proposal_items: ProposalItemsRead, deployment }
versioned record DeploymentAnswer { welcome: bool }
```

`deployment` is `Read`'s last variant, so that no tag moves. In `temper-client-protocol-wire`'s
`lib.rs`, beside the playground's pair:

```rust
pub fn deployment_answer(record: &v1::DeploymentAnswer) -> Box<[u8]>;
pub fn decode_deployment_answer(bytes: &[u8], limits: &v1::Limits) -> Option<v1::DeploymentAnswer>;
```

The read always answers a record, on every deployment: never the read's empty answer.

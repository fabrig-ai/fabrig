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

## K3. init's two documents

`temper_init::files`, `Facts` and `Ports` stay exactly as they are, the box's: `temper-init`'s
tests and every other caller keep reading them, as they keep reading `template/temper.json` and
`template/agent.json`. The box's template gains `"welcome": true` in `application`.

Beside them, in `temper-init`:

```rust
/// What a bare document needs: the facts of `Facts` that are not the box's parts.
pub struct Bare<'a> {
    pub binary: &'a [u8],
    pub git: Option<&'a [u8]>,
    pub claude: Option<&'a [u8]>,
    pub clients: NonZeroU16,
    pub webhooks: NonZeroU16,
}
/// `worker.secret`, `worker.digest`, `agent.json`, `temper.json`, in that order.
pub fn bare(facts: &Bare<'_>, directories: &Directories<'_>, secret: &[u8; 32]) -> Result<Box<[File]>, Refusal>;
```

from two templates, `template/bare.json` and `template/bare-agent.json`. The bare document is the
box's less the box's parts (`temper/docs/design/protocol/setup.md`, 8.3): `application.machine` is
`{"github": null}`, `application.demo` is `null` (the member is required), `application.welcome`
is absent, `engine.acts` is absent, and provider 3, its endpoint and its model `demo` are gone from
`engine.llm`; the agent's configuration loses the scripted provider's endpoint and model.

In the binary (`temper/crates/temper/src/init.rs`):

```rust
pub(crate) fn core(directories: &Directories) -> Result<Written, String>; // the box's, unchanged
pub(crate) fn bare(directories: &Directories) -> Result<Written, String>;
pub(crate) fn write_bare(directories: &Directories, environment: &[EnvironmentEntry], binary: &Path, preferred: [u16; 2]) -> Result<Written, String>;
pub(crate) fn run(directories: &Directories, boxed: bool) -> Result<u8, String>;
```

`[8367, 8371]` are bare's preferred ports, the clients' and the webhooks'. `Command::Init {
directories, boxed }`: `temper init --box` is `boxed: true`. `temper box`'s first run and
`temper box --fake-backends` keep calling `core` and `write_with_program`: neither changes.

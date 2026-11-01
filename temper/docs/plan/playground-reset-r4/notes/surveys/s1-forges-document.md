# 1. Deployment document: parsing, types, members, and readers

Read-only survey; no builds or tests run. Paths below are relative to the checkout root. `path:line` identifies a declaration or a representative direct use. Reverse-dependency lists distinguish production readers, constructors, and document fixtures.

**Important current-state findings:**

- Forge numbers come from `application.forges[].number`; no runtime forge-number allocator was found.
- A fake is `kind: "forgejo"` plus a fake setup mode; `settings::Kind::Fake` does not exist.
- A durable forge configuration record does not exist.
- `forge.forges` does not exist. The current connection limit is `forge_component.forges`, and several domain tables instead use `forge.client.repositories`.
- Legacy credential-file configuration remains accepted for Forgejo.

## Parsing and typed representations

| Layer | Exact location and API |
|---|---|
| JSON parsing | `temper/crates/temper-service/src/startup.rs:429`: `fn load_with(input: &[u8], directories: &config::Directories, check_start: bool, given: &crate::stock::Given) -> Result<(Shape, Option<Parsed>), Problems>`; calls `config::document`, then `load_document`. |
| Jig engine member | `temper/crates/temper-service/src/startup.rs:384`: `pub fn engine_with(document: &skein_json::Document, at: u32, directories: &config::Directories, given: &crate::stock::Given) -> Result<config::Engine, config::Problems>`; calls `jig_protocol_config::engine(..., Defaults::Passed)`. |
| Temper application member | `temper/crates/temper-service/src/startup/application.rs:770`: `pub fn parse(document: &Document, at: u32, jig: &config::Engine, directories: &config::Directories, problems: &mut Problems) -> Option<Members>`. Called at `startup.rs:560`. |
| Parsed application aggregate | `temper/crates/temper-service/src/startup/application.rs:18`: `Members`, including `forges: Box<[Forge]>`, `identities: Box<[Identity]>`, `demo: Option<settings::Demo>`, `asked_owned: Box<[u16]>`, listeners and limits. |
| Pre-file-read projection | `temper/crates/temper-service/src/startup.rs:222`: `pub struct Named`; `named_with` at line 683 and `named` at line 719. |
| Checked runtime aggregate | `temper/crates/temper-service/src/settings.rs:139`: `pub struct Engine`; fields include `jig`, `clients`, `webhooks`, `forges`, `demo`, `setup`, `records_only`, `identities`, and `limits`. |

The individual forge parser is:

```rust
// temper/crates/temper-service/src/startup/application.rs:355
fn forge(
    document: &Document,
    at: u32,
    path: &Path,
    problems: &mut Problems,
) -> Option<Forge>
```

### Forge and identity fields

Parsed forge: `temper/crates/temper-service/src/startup/application.rs:96`.  
Checked runtime forge: `temper/crates/temper-service/src/settings.rs:55`.

| Member | Parsed field | Checked runtime field |
|---|---|---|
| Number | `number: u16` | `number: u16` |
| Kind | `kind: settings::Kind` | `kind: Kind` |
| API address | `api: config::Address` | `api: jig_protocol_config::Address` |
| Transport | `transport: startup::application::Transport` | `transport: settings::Transport` |
| Git base | `git: Box<[u8]>` | same |
| Git executable | `git_program: Option<Box<[u8]>>` | same |
| Browser base | `web: Option<Box<[u8]>>` | same |
| App ownership restriction | `app_read_only: bool` | same |
| Legacy API-token filename | `api_token: Box<[u8]>` | `api_token: Option<Secret>` |
| Legacy webhook-secret filename | `webhook_secret: Option<Box<[u8]>>` | `webhook_secret: Option<Secret>` |
| Bot’s forge account | `user: User` | `user: u64` |
| Git identity join | `identity: u32` | same |
| Credential-source distinction | `records_only: bool` | absent from runtime `Forge`; runtime `Engine.records_only: Box<[u16]>` holds the distinction |
| Setup mode | derived later | `mode: temper_engine_domain_forge::Mode` |

Exact supporting enums:

```rust
// temper/crates/temper-service/src/settings.rs:80
pub enum Kind { Forgejo, GitHub }

// temper/crates/temper-service/src/settings.rs:87
pub enum Transport {
    Plaintext,
    Tls { server_name: Box<[u8]>, trust: Option<Box<[u8]>> },
}

// temper/crates/temper-service/src/startup/application.rs:134
pub enum User {
    Id(u64),
    File(Box<[u8]>),
}
```

GitHub’s null `api`, `transport`, `git`, and `web` defaults are applied in `startup/application.rs:374–418`. Forgejo’s records-based form is detected at lines 424–440. The legacy inverse `forges[].identity` join is read at lines 449–461.

Git identity is separate from the forge’s numeric bot account:

```rust
// Parsed: startup/application.rs:148
pub struct Identity {
    pub number: u32,
    pub forge: Option<u16>,
    pub token: Box<[u8]>,
    pub token_configured: bool,
    pub login: Box<[u8]>,
    pub author: settings::Author,
}

// Checked: settings.rs:96
pub struct Identity {
    pub number: u32,
    pub forge: u16,
    pub token: Secret,
    pub login: Box<[u8]>,
    pub author: Author,
}

// settings.rs:106
pub struct Author {
    pub name: Box<[u8]>,
    pub email: Box<[u8]>,
}
```

`fn identity(...) -> Option<Identity>` is at `startup/application.rs:502`. `references` at line 572 validates uniqueness and joins forge/identity records. `resolve` at line 1134 constructs the checked runtime forms.

### Setup entries

Declared and parsed by crate **`temper-forge-setup`**, `temper/crates/temper-forge-setup/src/setup.rs`:

```rust
// :78
pub struct Entry {
    pub forge: u16,
    pub directory: Box<[u8]>,
    pub bot: Account,
    pub orgs: Box<[Box<[u8]>]>,
    pub mode: Mode,
    pub asked: bool,
}

// :71
pub enum Mode {
    Managed { admin_token: Option<Box<[u8]>> },
    Owned(Owned),
    Fake(Fake),
}

// :14
pub struct Account {
    pub login: Box<[u8]>,
    pub email: Box<[u8]>,
}
```

Nested fields:

| Type | Location | Fields |
|---|---|---|
| `Owned` | `setup.rs:28` | `public: Option<Origin>`, `fetch: Fetch`, `forgejo: Box<[u8]>`, `runner: Option<Runner>`, `listen: SocketAddr`, `person: Account` |
| `Runner` | `setup.rs:20` | `binary`, `labels: Box<[Box<[u8]>]>`, `capacity: u32`, `path` |
| `Fetch` | `setup.rs:38` | `forgejo: bool`, `runner: bool`, `mirror: Option<Mirror>` |
| `Origin` | `setup.rs:51` | `host: Box<[u8]>`, `port: u16` |
| `Fake` | `setup.rs:63` | `listen: SocketAddr`, `git`, `owner`, `repositories: Box<[Repository]>` |
| Setup `Repository` | `setup.rs:57` | `name`, `landing` |
| `Setup` | `setup.rs:88` | `entries: Box<[Entry]>` |

APIs:

```rust
// setup.rs:93
pub fn read(
    d: &Document,
    at: u32,
    limits: &Limits,
    directories: &config::Directories,
    problems: &mut Problems,
) -> Option<Self>

// setup.rs:161
pub fn entry(&self, number: u16) -> Option<&Entry>
```

`start: "asked"` becomes `Entry.asked`. The checked setup wrapper is `temper/crates/temper-forge-setup/src/check.rs:40`, `Checked { fetch_bytes, records_only, setup, limits }`.

### Demo, LLM, acts, listeners, ports, limits

| Item | Declaration / parser |
|---|---|
| Service demo projection | `temper/crates/temper-service/src/settings.rs:7`: `Demo { project: u32, forge: u16, provider: u32, owner, name, cues, landing, key_file, state: Option<DemoState> }`. Parsed by `startup/application/demo.rs:26`, `parse(...) -> Option<settings::Demo>`. |
| Whole demo configuration | `temper/crates/temper-fake/src/demo.rs:47`: private fields `llm: llm::Document`, `provider: u32`, `forge: u16`, `project: u32`. `pub fn read(document: &Json, at: u32, facts: &Facts<'_>) -> Result<Option<Demo>, Refusal>` at line 82. Reads `llm.listen`, `provider`, `key`, `asked`, optional `cued`, `otherwise` at lines 94–109. |
| Fake LLM document | `temper/crates/temper-fake/src/llm.rs:20`: `Document`, including `listen: SocketAddr`, key pathname, models, scripts and rules. |
| `engine.llm` | `jig/crates/jig-protocol-config/src/lib.rs:69`, `Engine.llm: Llm`. `llm.rs:11`: `Llm { endpoints: Box<[Endpoint]>, models: Box<[Model]>, descriptions, providers: Box<[Provider]> }`; parser at line 44. `Endpoint` is at line 22. |
| `engine.acts` | `jig/crates/jig-protocol-config/src/lib.rs:58`, `acts: Box<[jig_domain_people::ActGrant]>`. Parser `acts.rs:43`, `fn grant(...) -> Option<ActGrant>`. |
| Act grants | `jig/crates/jig-domain-people/src/boundary.rs:688`: `ActGrant { connector: u16, may: Box<[ActKind]> }`. `ActKind` at line 673: `Close`, `AddKey`, `RemoveAccount`, `Adopt`, `MakeProject`. |
| Clients listener | `temper/crates/temper-service/src/startup/application.rs:36`: `Clients { listener: config::Address, bundle: Option<Box<[u8]>>, public: Option<Origin> }`; parser `clients(...) -> Option<Clients>` at line 51. |
| Webhook listener | same file, line 46: `Webhooks { listener: config::Address }`; parser at line 761. |
| Address | `jig/crates/jig-protocol-config/src/address.rs:8`: `Address::Addr(skein_io::Addr)` or `Address::Host { name: Box<[u8]>, port: u16 }`; `read(...) -> Option<Address>` at line 19. |
| Temper limits | `temper/crates/temper-service/src/limits.rs:45`: `Limits`; profile at line 331; `apply(limits: &mut Limits, name: &[u8], value: u64) -> Result<(), Refused>` at line 358. |
| Memory | `jig/crates/jig-protocol-config/src/lib.rs:66`, `Engine.memory: u64`; JSON name `memory_bytes` parsed at line 146. See question 5. |

There are **no independent machine-owned Forgejo/fake port fields today**. Those ports are embedded in forge API addresses, setup `Owned.listen`/`Fake.listen`, and `application.demo.llm.listen`.

## Forge-member reverse dependencies

The table identifies production field readers beyond parsing/conversion. Shared aggregate forwarding is listed afterward.

| Members | Direct readers |
|---|---|
| `number` | `temper/crates/temper/src/lib.rs:343`; `temper/crates/temper/src/setup.rs:30`; service `forge.rs:53`, `forge/records.rs:163`, `forge/tokens.rs:38`, `family.rs:197,453`, `translate.rs:49`, `setup/settings.rs:48,268`, `setup/lifecycle.rs:544`, `startup/application.rs:1294` |
| `kind` | service `forge.rs:86`, `forge/records.rs:164`, `forge/tokens.rs:39`, `family.rs:460`, `translate.rs:85`, `setup/settings.rs:37`, `startup/checks.rs:1045`, `startup/application.rs:1141,1327`; binary `setup.rs:34` |
| `api` | binary `lib.rs:350`, `setup.rs:39`; service `forge.rs:69,229`, `forge/records.rs:166`, `startup/checks.rs:1042`, `startup/application.rs:1173`; setup’s origin comparison uses it through `setup/settings.rs:106` |
| `transport` | binary `lib.rs:355`, `setup.rs:40`; service `startup/application.rs:1165`, `forge.rs:77`; resolved endpoint transport reaches `setup/settings.rs:18` |
| `git` | binary `setup.rs:41`; service `settings.rs:29`, `family.rs:199,464`, `translate.rs:56`, `startup.rs:617`, `startup/checks.rs:328`, `startup/application.rs:1300,1342`, `setup/settings.rs:109` |
| `git_program` | service `translate.rs:81`, `startup/application.rs:1045,1176,1251` |
| `web` | service `family.rs:464`, `setup/settings.rs:54`, `startup/application.rs:1177,1346` |
| `app_read_only` | service `setup/settings.rs:52`, `startup/application.rs:1178,1347` |
| `api_token`, `user`, `webhook_secret` | binary `setup.rs:45–48`; service `forge.rs:114–120`, `startup/application.rs:1139–1181`, `setup/settings.rs:269`; old startup access handling in `setup/settings.rs:198–235` |
| `identity` | binary `setup.rs:32`; service `forge.rs:462`, `forge/tokens.rs:44`, `setup/settings.rs:272`; validation/join in `startup/application.rs:572` |
| Identity `number`, `forge`, `token`, `login`, `author` | service `values.rs:138–150`, `translate.rs:167–174`, `forge/tokens.rs:43–56`, `setup/settings.rs:271–314`, `setup/lifecycle.rs:613–622`; binary `setup.rs:32` |
| `mode`, `asked`, setup bot | service `startup/application.rs:1045,1291`, `family.rs:457`, `setup/settings.rs:277–307`; domain readers listed in question 3 |

**Additional aggregate readers/forwarders of `settings::Forge` / identities:**

```text
temper/crates/temper-service/src/assignment.rs:76
temper/crates/temper-service/src/clients.rs:453,545,677
temper/crates/temper-service/src/clients/bundle.rs:341
temper/crates/temper-service/src/engine.rs:124,344,427
temper/crates/temper-service/src/route.rs:143,159
temper/crates/temper-service/src/startup.rs:797,858,908,916
temper/crates/temper-service/src/system.rs:197,215,217,221,227
temper/crates/temper-service/src/workers.rs:575
```

**Typed constructors/readers in tests and worlds:**

```text
temper/crates/temper-service/src/tests/fixtures.rs:80,125,401
temper/crates/temper-service/src/tests/family.rs:13,596
temper/crates/temper-service/src/tests/engine.rs:31
temper/crates/temper-service/src/tests/setup_admin.rs:19
temper/crates/temper-service/src/tests/setup_adoption.rs:260,435
temper/crates/temper-service/src/tests/setup_lifecycle.rs:700
temper/crates/temper-service/src/tests/startup.rs:1872,1890,1908
temper/tests/client/src/fixtures.rs:21
temper/tests/system/src/composed/link.rs:30
```

Document constructors and mutations are catalogued in question 7; they are additional reverse dependencies of the JSON members.

### Setup reverse dependencies

Production readers/constructors:

```text
temper/crates/temper-forge-setup/src/check.rs:64,222
temper/crates/temper-forge-setup/src/fetch.rs:82
temper/crates/temper-forge-setup/src/given.rs:104
temper/crates/temper-forge-setup/src/ini.rs:59
temper/crates/temper-forge-setup/src/prepare.rs:149
temper/crates/temper-forge-setup/src/run.rs:33
temper/crates/temper-forge-setup/src/runner.rs:21
temper/crates/temper-forge-setup/src/spec.rs:49
temper/crates/temper-forge-setup/src/state.rs:16
temper/crates/temper-forge-setup/src/step.rs:144
temper/crates/temper-fake/src/demo.rs:74
temper/crates/temper-fake/src/forge.rs:53
temper/crates/temper-fake/src/steps.rs:21
temper/crates/temper-fake/src/backends.rs:467
temper/crates/temper-service/src/settings.rs:130
temper/crates/temper-service/src/startup.rs:21,77,81
temper/crates/temper-service/src/startup/application.rs:1045,1291
temper/crates/temper-service/src/startup/application/sign_in/forges.rs:53
temper/crates/temper-service/src/clients/bundle.rs:342
temper/crates/temper-service/src/setup/settings.rs:160
temper/crates/temper/src/setup.rs:17
temper/crates/temper/src/forge.rs:201,262,318
temper/crates/temper/src/forge_sign_in.rs:9
temper/crates/temper/src/fake.rs:351
temper/crates/temper/src/fake/check.rs:23
temper/crates/temper/src/fake_backends.rs:352,559
temper/crates/temper/src/box_command.rs:51,189
```

Setup-specific tests:

```text
temper/crates/temper-forge-setup/src/tests/read.rs:17
temper/crates/temper-forge-setup/src/tests/given.rs:69
temper/crates/temper-forge-setup/src/tests/managed.rs:28
temper/crates/temper-forge-setup/src/tests/members.rs:57
temper/crates/temper-forge-setup/src/tests/fixtures.rs:98
temper/crates/temper-forge-setup/src/tests/fetch.rs:10
temper/crates/temper-forge-setup/src/tests/steps.rs:68
temper/crates/temper-fake/src/tests/demo.rs:8
temper/crates/temper-fake/src/tests/steps.rs:6
temper/crates/temper-fake/src/tests.rs:595
temper/crates/temper-fake/src/backends/tests.rs:107
temper/crates/temper-service/src/tests/setup.rs:764
temper/crates/temper-service/src/tests/setup_create.rs:333
temper/crates/temper-service/src/tests/setup_scopes.rs:5
temper/crates/temper-service/src/tests/setup_adoption.rs:3
temper/crates/temper-service/src/tests/setup_admin.rs:19
temper/crates/temper-service/src/tests/sign_in_configuration.rs:273
temper/crates/temper-service/src/tests/startup.rs:464
temper/crates/temper-service/src/tests/fixtures.rs:1031
temper/crates/temper/src/init/tests.rs:324
temper/crates/temper/src/fake_backends/tests.rs:231
temper/crates/temper-forge-setup-shell/tests/support/mod.rs:8
temper/crates/temper-forge-setup-shell/tests/suite/prepare.rs:6
temper/crates/temper-forge-setup-shell/tests/suite/files.rs:6
temper/crates/temper-forge-setup-shell/tests/suite/locks.rs:3
temper/crates/temper-forge-setup-shell/tests/suite/commands.rs:2
temper/crates/temper-forge-setup-shell/tests/suite/fetch.rs:16
temper/crates/temper-forge-setup-shell/tests/suite/exchange.rs:2
temper/crates/temper-forge-setup-shell/tests/suite/reconcile.rs:5
temper/crates/temper/tests/support/owned.rs:172
temper/crates/temper/tests/suite/forge.rs:3
temper/crates/temper/tests/qa_all/qa_adoption.rs:58
```

The shell also consumes the derived setup plans/specifications in:

```text
temper/crates/temper-forge-setup-shell/src/prepare.rs:5
temper/crates/temper-forge-setup-shell/src/process.rs:8
temper/crates/temper-forge-setup-shell/src/run.rs:4
temper/crates/temper-forge-setup-shell/src/files.rs:8
temper/crates/temper-forge-setup-shell/src/command.rs:11
temper/crates/temper-forge-setup-shell/src/fetch.rs:20
temper/crates/temper-forge-setup-shell/src/parent.rs:14
temper/crates/temper-forge-setup-shell/src/reconcile.rs:11
```

### Demo reverse dependencies

In addition to the document fixtures in question 7:

```text
temper/crates/temper-service/src/settings.rs:19
temper/crates/temper-service/src/startup/application/demo.rs:26
temper/crates/temper-service/src/startup/application.rs:23,847,1299
temper/crates/temper-service/src/startup/checks.rs:667
temper/crates/temper-service/src/startup.rs:909
temper/crates/temper-service/src/system.rs:223
temper/crates/temper-service/src/engine.rs:18
temper/crates/temper-service/src/clients.rs:454
temper/crates/temper-service/src/family.rs:30
temper/crates/temper-service/src/forge.rs:12
temper/crates/temper-service/src/route.rs:10
temper/crates/temper-service/src/setup/settings.rs:255
temper/crates/temper-service/src/clients/demo_tests.rs:28
temper/crates/temper-service/src/tests/family.rs:170
temper/crates/temper-service/src/tests/checks.rs:367
temper/crates/temper-service/src/tests/stock.rs:49
temper/crates/temper-service/src/tests/setup.rs:1631
temper/crates/temper-service/src/tests/setup_admin.rs:617
temper/crates/temper-service/src/tests/setup_github.rs:121
temper/crates/temper-service/src/tests/clients.rs:256
temper/crates/temper-service/src/tests/fixtures/http.rs:112
temper/crates/temper-service/src/tests/startup.rs:297
temper/crates/temper-service/src/tests/welcome_policy.rs:65
temper/crates/temper-fake/src/demo.rs:82
temper/crates/temper-fake/src/steps.rs:21
temper/crates/temper-fake/src/tests/demo.rs:86
temper/crates/temper-fake/src/tests.rs:698
temper/crates/temper/src/fake.rs:391
temper/crates/temper/src/fake_backends.rs:283
temper/crates/temper/src/fake_backends/tests.rs:232
temper/crates/temper/src/init/tests.rs:327
temper/crates/temper/src/tests.rs:190
```

### Help-document dependencies

These contain configuration semantics, fixed forge numbers, or commands that depend on existing slots:

```text
temper/docs/help/forge.md:5,10
temper/docs/help/playground.md:9
temper/docs/help/forgejo.md:30,36
temper/docs/help/github.md:10,34,84
temper/docs/help/box.md:144,148
temper/docs/help/settings.md:90
temper/docs/help/reference/ctl-forge.md:14,52,59
temper/docs/help/reference/commands.md:104
temper/crates/temper-docs/src/lib.rs:143
```

Broader explanatory references are `temper/docs/help/deployment.md:5`, `index.md:9`, and `project.md:11`. `server.md:32` contains a worker-listener port example, not a forge-number declaration.

**References from skein, smith, or jig to temper’s application configuration types: not found.** Same-named upstream `Forge`, `Configured`, and `ActKind` types are separate types.

# 2. `temper-init`

## What it writes and asks

The current template is `temper/crates/temper-init/template/temper.json`:

| Lines | Current content |
|---|---|
| 3 | `engine.acts = [{"connector":1,"may":["close","remove_account"]}]` |
| 31 | `engine.llm` |
| 60–64 | Demo provider, number **3** |
| 84–94 | Demo endpoint, number **3**, socket placeholder `$llm` |
| 164 | `engine.memory_bytes = 1073741824` |
| 211–239 | Forge **1** fake Forgejo endpoint, **2** GitHub, **3** owned Forgejo endpoint |
| 241–265 | Identity **1 → forge 2**, identity **2 → forge 1**, identity **3 → forge 3** |
| 291–301 | Client and webhook listeners |
| 303–325 | Forge 1’s fake setup |
| 327–354 | Forge 3’s owned setup |
| 334 | Forge 3’s `"start": "asked"` |
| 358 onward | `application.demo`, including fake LLM scripts |

“Asked” means **dormant until setup is requested**, not an interactive question from init. Init derives the owned Forgejo person’s login from the machine login; `temper/crates/temper-init/src/person.rs` validates/falls back. No interactive forge-configuration prompt was found.

Generation:

```rust
// temper/crates/temper-init/src/lib.rs:119
pub fn files(
    facts: &Facts<'_>,
    directories: &Directories<'_>,
    secret: &[u8; 32],
    demo_key: &[u8; 32],
) -> Result<Answer, Refusal>
```

It returns, in order, `worker.secret`, `worker.digest`, `demo/llm.key`, `demo/bin/git`, `agent.json`, `temper.json` (`lib.rs:141–171`).

- Template embedding: `temper/crates/temper-init/src/template.rs:12`.
- `documents(facts, directories, login) -> (Box<[u8]>, Box<[u8]>)`: line 15.
- Port/address substitutions: lines 18–47.
- File creation: `temper/crates/temper/src/init.rs:180`, `write_with_program(...) -> Result<Written, String>`, with writes at lines 243 onward.
- Public command path: `init.rs:272`, `core`, then `run` at line 276.

## Ports

```rust
// temper/crates/temper-init/src/lib.rs:33
pub struct Ports {
    pub clients: NonZeroU16,
    pub owned: NonZeroU16,
    pub forge: NonZeroU16,
    pub llm: NonZeroU16,
    pub webhooks: NonZeroU16,
}
```

Default order and preferences, `temper/crates/temper/src/init.rs:274`:

```rust
[8367, 8368, 8369, 8370, 8371]
```

All bind `127.0.0.1`. `Io::port` at `init.rs:78` falls back to a kernel-selected port when the preferred port is taken; `listened` at line 98 handles the bind/listen race. All reservations remain held while files are written. Port-to-field assignment is at lines 209–214.

The OpenAI sign-in redirect port **1455** is separately declared in the template at line 55.

## Tests and reverse dependencies

Template-specific tests:

```text
temper/crates/temper-init/src/tests/documents.rs:15
  the_owned_example_offers_openai_keys_and_chatgpts_sign_in
temper/crates/temper-init/src/tests/documents.rs:58
  the_default_is_the_owned_example_so_edited
temper/crates/temper-init/src/tests/documents.rs:343
  the_agent_configuration_is_the_owned_examples_but_its_programs
temper/crates/temper-init/src/tests/documents.rs:360
  facts_are_written_as_json_text
temper/crates/temper-init/src/tests/documents.rs:394
  no_placeholder_is_left
temper/crates/temper-init/src/tests/documents.rs:435
  the_person_keeps_a_forgejo_login_or_falls_back
temper/crates/temper-init/src/tests/documents.rs:474
  the_template_keeps_project_one_without_settings_members
temper/crates/temper-init/src/tests/files.rs:5,35
temper/crates/temper-init/src/tests/paths.rs:14
temper/crates/temper-init/src/tests/fixtures.rs:9
```

Filesystem/port integration tests:

```text
temper/crates/temper/src/init/tests.rs:52
  taken_default_owned_and_webhooks_ports_get_distinct_kernel_ports
temper/crates/temper/src/init/tests.rs:73
  a_port_another_init_listens_on_between_bind_and_listen_gets_a_kernel_port
temper/crates/temper/src/init/tests.rs:94
  init_finds_ports_and_programs_and_writes_exact_modes_in_order
temper/crates/temper/src/init/tests.rs:271
  init_refusals_leave_every_file_and_directory_untouched
temper/crates/temper/src/init/tests.rs:317
  generated_configuration_passes_the_same_checks_as_serve
temper/crates/temper/src/init/tests.rs:336
  the_person_diagnostic_names_the_login_or_the_uid
temper/crates/temper/src/init/tests.rs:352
  explicit_program_fact_precedes_path_discovery
```

Process tests:

```text
temper/crates/temper/tests/suite/startup.rs:409
  a_box_on_a_fresh_init_reaches_serves_link_with_forge_3_dormant
temper/crates/temper/tests/suite/startup.rs:1316
  init_prints_written_files_account_guidance_and_explicit_start_flags_on_stderr
temper/crates/temper/tests/suite/startup.rs:1364
  box_initializes_only_absent_or_empty_configuration_directories
```

Additional direct template readers:

```text
temper/tests/engine/domain/tests/fixtures/stock.rs:4
temper/tests/engine/domain/src/forge_routes.rs:1921
```

# 3. Engine startup, forge state, acts, and setup host

## Startup wiring

1. `startup/application.rs:1134`, `resolve`, creates `settings::Forge` and identities.
2. `startup/application.rs:1045`, `git_programs`, derives setup mode and fake git executable.
3. `temper/crates/temper/src/lib.rs:343`, `forges`, resolves configured endpoints into `ForgeRuntime`.
4. `temper/crates/temper-service/src/forge.rs:42`, `settings`, joins runtime endpoints by forge number and creates `skein_forge_connection::settings::Settings`.
5. `system.rs:197` calls `forge::config`; at `forge.rs:454` this constructs the `GitIdentity` array. Its initial writer array is empty.
6. `system.rs:215` prepares setup; line 221 calls `startup::application::configured`.
7. `system.rs:228` constructs `temper_engine_domain::Config`.
8. `temper-engine-domain/src/domain.rs:275` constructs the forge domain.
9. `temper-service/src/engine.rs:137` supplies connection/installation limits.
10. `temper-service/src/workers.rs:575` moves runtime forge settings into `Workers`; assignments and client rendering later read `engine.workers.forges`.

The root configuration is `temper/crates/temper-engine-domain/src/translate.rs:16`:

```rust
pub struct Config {
    // other root/core fields...
    pub forge: skein_forge_client::Config,
    pub identities: Box<[forge::GitIdentity]>,
    pub forge_charter: u32,
    pub forges: Box<[forge::Configured]>,
    pub under_box: bool,
    pub demo: Option<forge::DemoConfig>,
}
```

The domain receives only this public configuration:

```rust
// temper/crates/temper-engine-domain-forge/src/boundary.rs:298
pub struct Configured {
    pub admin_connect: bool,
    pub bot: Box<[u8]>,
    pub owned: Option<Owned>,
    pub git: Box<[u8]>,
    pub web: Option<Box<[u8]>>,
    pub read_only: bool,
    pub number: u16,
    pub kind: Kind,
    pub mode: Mode,
    pub asked: bool,
}
```

**API socket, TLS configuration, setup entry and credentials are not held in `Configured`.** They remain in protocol/service configuration.

Constructor:

```rust
// temper/crates/temper-engine-domain-forge/src/domain.rs:492
pub fn new(
    l: &Limits,
    seed: u64,
    config: skein_forge_client::Config,
    identities: &[crate::GitIdentity],
    forges: &[crate::Configured],
    under_box: bool,
) -> Result<Domain, skein_forge::api::Error>
```

`configured.insert(forge.number, forge.clone())` occurs at line 512. Its getter explicitly says “never restored from records”:

```rust
// domain.rs:208
pub fn configured(&self, forge: u16) -> Option<&crate::Configured>
```

## Forge-keyed state

All fields below are in `temper/crates/temper-engine-domain-forge/src/domain.rs:119`:

| Field | Exact type / line |
|---|---|
| `configured` | `Map<u16, crate::Configured>` — 120 |
| `connections` | `Map<u16, ConnectionState>` — 128 |
| `first` | `Map<u16, crate::First>` — 129 |
| `ended` | `Map<u16, (Box<[u8]>, Option<crate::AdminStop>)>` — 130 |
| `apps` | `Map<u16, crate::AppWay>` — 131 |
| `installations` | `Map<(u16, u64), crate::Installation>` — 135 |
| `repositories` | `Map<skein_forge::api::Repository, Repository>` — 136 |
| `flights` | `Map<(u64, u64), ()>` — 148; key is **task, entry**, not forge |
| `writers` | `Map<u16, u64>` — 165 |
| `identities` | `Map<u16, u32>` — 168 |

`ConnectionState = (crate::Connection, Option<crate::FailedStep>)` is at line 115.

Repository/provider identity:

```rust
// skein/crates/skein-forge/src/api.rs:58
pub struct Repository {
    pub forge: u16,
    pub repository: u32,
}

// temper-engine-domain-forge/src/boundary.rs:120
pub struct GitIdentity {
    pub forge: u16,
    pub credential: u32,
}

// skein/crates/skein-forge-client/src/identity.rs:37
pub struct Writer {
    pub forge: u16,
    pub author: u64,
}
```

Writer installation is `domain.rs:1487`, `pub fn writer(...) -> bool`. A successful connection supplies its bot ID at `domain.rs:1116–1122`.

Other forge-associated state:

- Secret rows: `temper-protocol-records/src/forge.rs:53`, `Rows.forges: Map<u16, ForgeRows>`; details in question 6.
- GitHub installation-account tokens: `temper-service/src/forge/tokens.rs:28`, `Tokens.forges: Map<u16, Forge>`. `Forge` at line 18 holds identity number, login, author, `entries: Map<u64, Entry>`, and `made: List<(Box<[u8]>, u64)>`. `Entry.account: u64` is at line 12.
- These installation accounts are distinct from jig’s LLM accounts.
- Hints contain `Hint.repository: api::Repository`; intake is `temper-engine-domain-forge/src/domain.rs:1371`, dependency checks are `domain/hints.rs:8`, and CI hint handling is `domain.rs:4308`.
- The current “forge ended” test is demo-specific: `temper-engine-domain-forge/src/demo.rs:42`, `pub fn forge_ended(&self, forge: u16) -> bool`.

The protocol component is also fixed from startup settings:

- `skein/crates/skein-forge-connection/src/settings.rs:21`, `Forge { number, kind, address, transport, access, log_origins }`.
- `component.rs:102`, `Component` retains `settings: Settings`, `starts: List<Start>`, and per-index GitHub state.
- `component.rs:124`, `pub fn new(settings: Settings, limits: &Limits, shared: &Shared) -> Result<Self, Unusable>`.
- Dynamic creation/removal of configured forge entries: **not found**.

### `Configured` reverse dependencies

Production readers, constructors, and forwarding:

```text
temper/crates/temper-service/src/startup/application.rs:1291
temper/crates/temper-service/src/system.rs:221
temper/crates/temper-service/src/startup/checks.rs:676
temper/crates/temper-engine-domain/src/translate.rs:34
temper/crates/temper-engine-domain/src/domain.rs:275,1220
temper/crates/temper-engine-domain/src/route/forge_route.rs:140,167,1571,2647
temper/crates/temper-engine-domain-forge/src/domain.rs:120,208,238,306,492
temper/crates/temper-engine-domain-forge/src/connect.rs:73,163,363
temper/crates/temper-engine-domain-forge/src/demo.rs:140
temper/crates/temper-engine-domain-forge/src/brief.rs:156
temper/crates/temper-engine-domain-forge/src/landing.rs:120
temper/crates/temper-engine-domain-forge/src/reconcile.rs:292
```

Tests/worlds:

```text
temper/crates/temper-engine-domain-forge/src/tests.rs:1223
temper/crates/temper-engine-domain-forge/src/tests/connect.rs:16
temper/crates/temper-engine-domain-forge/src/tests/demo.rs:146
temper/crates/temper-engine-domain/src/tests/setup.rs:19,625
temper/crates/temper-engine-domain/src/route/forge_route/tests.rs:728
temper/crates/temper-service/src/tests/setup_adoption.rs:260,435
temper/crates/temper-service/src/tests/setup_lifecycle.rs:700
temper/crates/temper-service/src/tests/startup.rs:1872,1890,1908
temper/tests/engine/domain/tests/qa_all/qa_first_repository.rs:16,212
temper/tests/engine/domain/tests/suite/demo.rs:31,43
temper/tests/client/src/front.rs:161
temper/tests/client/src/front/account_acts.rs:190
temper/tests/client/tests/suite/forges.rs:32,44,56,68
temper/tests/client/tests/qa_all/qa_forges.rs:60,72,137,149,167,179,398,410,482,494,1117,1129
temper/tests/client/tests/qa_all/qa_first_repository.rs:24
temper/tests/client/tests/qa_all/qa_welcome_m3_owned.rs:26
```

`GitIdentity` additionally appears in:

```text
temper/crates/temper-engine-domain-forge/src/tests.rs:132,1179
temper/crates/temper-engine-domain-forge/src/tests/adoption.rs:305,411
temper/crates/temper-engine-domain/src/tests/resume_fixture.rs:349
temper/tests/engine/domain/src/walking.rs:358
temper/tests/engine/forge/src/world.rs:297,511
temper/tests/engine/forge/tests/suite/memory.rs:58
temper/tests/engine/protocol-forge/src/protocol.rs:138
```

## Current `Connect` and `SetUp`

Domain declarations, `temper/crates/temper-engine-domain-forge/src/boundary.rs:152`:

```rust
pub enum Ask {
    Adopt(Adoption),
    SetUp {
        forge: u16,
        org: Box<[u8]>,
        repository: Box<[u8]>,
        project: SetUpProject,
        playground: bool,
    },
    StartDemo,
    Connect { forge: u16, how: How },
    // Install, Accept, Reachable...
}

pub enum How {
    ForgejoAdmin { first: Option<FirstAsk> },
    GitHubApp { app: core::num::NonZeroU64 },
    GitHubManifest { back: Option<Box<[u8]>> },
}
```

`FirstAsk` is at line 163: `org`, `repository`, `project: SetUpProject`, `label`.

The current acts **name an existing configured forge**. They carry no new forge kind, API address, transport, bot identity, or setup entry.

Wire source, `temper/crates/temper-client-protocol-wire/schema/client-wire-v1.schema`:

```text
15 enum SetUpProject { existing: ExistingProject, new: NewProject }
16 record SetUp { forge: u16 org: bytes 1024 repository: bytes 1024 project: SetUpProject playground: bool }
21 record FirstAsk { org: bytes 1024 repository: bytes 1024 project: SetUpProject label: bytes 64 }
22 record ForgejoAdmin { first: option FirstAsk }
23 enum How { github_app: GitHubApp, github_manifest: GitHubManifest, forgejo_admin: ForgejoAdmin }
24 record Connect { forge: u16 how: How }
```

Generated declarations: `src/generated/v1.rs:1083` (`SetUpParts`), `1098` (`SetUp`), `1259` (`FirstAskParts`), `1357` (`How`), `1385` (`ConnectParts`), `1394` (`Connect`).

Decision/routing:

| Step | Location |
|---|---|
| Decode wire into domain ask | `temper-service/src/family.rs:46,84` |
| Force deployment scope and ask jig admission | `temper-engine-domain/src/route/applications.rs:19–47` |
| Decide `SetUp` | `temper-engine-domain-forge/src/domain.rs:1178–1240` |
| Decide offer | same file, `pub fn offer(&self, forge: u16) -> Offer`, line 306 |
| Decide `Connect` | `temper-engine-domain-forge/src/connect.rs:35`, `start`; Forgejo-admin admission at line 152 |
| Root output translation | `temper-engine-domain/src/route/forge_route.rs:2969–3002` |
| Service execution | `temper-service/src/route.rs:32–48` |

### Act reverse dependencies

Beyond the declaration/routing files above:

```text
temper/crates/temper-engine-domain-forge/src/demo.rs:110,128
temper/crates/temper-engine-domain-forge/src/tests.rs:1343,1504,1681,1750
temper/crates/temper-engine-domain-forge/src/tests/connect.rs:33,622,708
temper/crates/temper-engine-domain-forge/src/tests/demo.rs:277,383
temper/crates/temper-engine-domain/src/tests/setup.rs:124
temper/crates/temper-service/src/setup/public.rs:50
temper/crates/temper-service/src/tests/family.rs:846,1043,1406
temper/crates/temper-service/src/tests/setup.rs:246,733
temper/crates/temper-service/src/tests/setup_admin.rs:67,91
temper/crates/temper-service/src/tests/setup_github.rs:98,203
temper/crates/temper-service/src/tests/setup_lifecycle.rs:389,558,724
temper/crates/temper-service/src/tests/qa_box_link.rs:21,73,132
temper/crates/temper-service/src/tests/startup.rs:2127
temper/crates/temper-service/src/tests/fixtures.rs:746
temper/crates/temper-client-domain/src/forge/family.rs:192,216
temper/crates/temper-client-domain/src/forge/github.rs:126,156
temper/crates/temper-client-domain/src/forge/admin.rs:152
temper/crates/temper-client-domain/src/tests/connect.rs:1060
temper/crates/temper-client-domain/src/tests/form.rs:90
temper/crates/temper-client-domain/src/tests/forges.rs:35
temper/crates/temper-client-domain/src/tests/admin.rs:24
temper/crates/temper-client-domain/src/tests/pages/welcome.rs:106
temper/crates/temper-client-tools/src/forges.rs:502
temper/crates/temper-client-tools/src/forges/connection.rs:45,125
temper/crates/temper-client-tools/src/tests/adopt.rs:315
temper/crates/temper-client-tools/src/tests/forges.rs:32
temper/crates/temper-client-tools/src/tests/forge_connection.rs:66
temper/crates/temper-client-tools/src/tests/fixtures/adopt.rs:84
temper/crates/temper-client-protocol-wire/src/lib.rs:256
temper/crates/temper-client-protocol-wire/src/tests.rs:294
temper/crates/temper-client-protocol-wire/src/tests/forgejo_admin.rs:7
temper/crates/temper-client-protocol-wire/src/tests/fixtures.rs:83
temper/crates/temper-client-view/src/tests/connect_forge.rs:14
temper/crates/temper-client-view/src/tests/forges.rs:9
temper/crates/temper-client-view/src/tests/welcome.rs:19
temper/crates/temper-client-view/src/tests/choice.rs:1258
temper/crates/temper-client-shell-web/src/host_tests.rs:437
temper/crates/temper-client-shell-terminal/src/tests/family.rs:22
temper/crates/temper-client-shell-terminal/src/tests/fixtures/chat.rs:330
temper/tests/client/src/front.rs:1130,1359
temper/tests/client/src/translate.rs:243
temper/tests/client/src/translate/forge.rs:158,237,418
temper/tests/client/tests/suite/forges.rs:290
temper/tests/codecs/tests/suite/client_wire.rs:49,75,129
temper/tests/engine/domain/tests/qa_all/qa_first_repository.rs:51
temper/tests/engine/domain/tests/suite/demo.rs:199
temper/tests/engine/domain/tests/suite/reconcile.rs:44
temper/tests/engine/domain/src/walking.rs:787
temper/tests/engine/domain/src/direct.rs:208
temper/tests/engine/domain/src/protocol.rs:295
temper/tests/engine/domain/src/roles.rs:533
temper/tests/engine/domain/src/forge_routes.rs:1133
temper/tests/engine/domain/src/escalation.rs:674
temper/tests/engine/forge/src/world.rs:782
temper/tests/engine/forge/tests/suite/simulation.rs:822
temper/tests/engine/protocol-forge/src/protocol.rs:508
temper/tests/engine/protocol-forge/tests/suite/protocol.rs:511
temper/tests/system/src/forge.rs:422
temper/tests/system/src/composed.rs:741
temper/tools/gallery/src/worlds.rs:298
temper/tools/gallery/src/worlds/github.rs:13
```

## Setup host and box

There are two distinct mechanisms:

| Responsibility | Code |
|---|---|
| Network setup/reconcile host | `skein-forge-setup::host::Host`, wrapped by `temper/crates/temper-service/src/setup.rs:18`, `Layer` |
| Host settings from configured forges | `temper/crates/temper-service/src/setup/settings.rs:8`, `fixed(config, forges) -> Result<(host::Settings, host::Limits), Problems>` |
| Lifecycle and durable credential writes | `temper/crates/temper-service/src/setup/lifecycle.rs` |
| Public Connect/manifest handling | `temper/crates/temper-service/src/setup/public.rs` |
| HTTP setup routes | `temper/crates/temper-service/src/setup/routes.rs` |
| Owned Forgejo preparation | `temper/crates/temper-forge-setup/src/prepare.rs`, `ini.rs`, `spec.rs`, `runner.rs` |
| Owned process running/fetching | same crate, `run.rs`, `fetch.rs`; OS execution in `temper-forge-setup-shell` |
| Box command composition | `temper/crates/temper-forge-setup/src/step.rs:144`, `pub fn steps(setup: &Setup, flags: &[Box<[u8]>], listeners: Box<[SocketAddr]>, forgejo_listens: bool) -> Steps` |
| Fake/demo composition | `temper/crates/temper-fake/src/steps.rs:21`, `pub fn steps(...) -> Steps` |
| Binary wiring | `temper/crates/temper/src/box_command.rs:47`, `run_configuration` |
| Group shape | `temper-forge-setup/src/step.rs:27`, `Group { forge, label, recorded, before, steps }`; `Steps { always, asked }` at line 38 |
| Pipe | `temper/crates/temper-box/src/pipe.rs:46`: `Command::{Start(u16), Stop(u16), Unknown}` |
| Started records | `temper/crates/temper-box/src/records.rs:191`: `box/started/<forge>` |

`<data>/box/groups/N`, persisted group settings, and pipe `remove N`: **not found**.

Box group/pipe reverse dependencies:

```text
temper/crates/temper-box/src/layout.rs:4
temper/crates/temper-box/src/machine.rs:8
temper/crates/temper-box/src/host.rs:23
temper/crates/temper-box/src/records.rs:38
temper/crates/temper-box/src/pipe.rs:46
temper/crates/temper-box/src/tests/pipe.rs:5
temper/crates/temper-box/src/tests/boundary.rs:50
temper/crates/temper-box/src/tests/host.rs:476
temper/crates/temper-box/src/tests/logs.rs:9
temper/crates/temper-box/src/tests/fixtures.rs:8
temper/crates/temper-box/src/tests/terminal.rs:36
temper/crates/temper-box/src/tests/machine.rs:8
temper/crates/temper-box/tests/real_loop.rs:19
temper/crates/temper-service/src/route/box_link.rs:304
temper/tests/one-process/src/restart_control.rs:37
```

# 4. First-repository intent in memory

There are **two copies**, both volatile.

## Domain copy

`temper/crates/temper-engine-domain-forge/src/domain.rs:129`:

```rust
pub(crate) first: Map<u16, crate::First>,
```

`temper/crates/temper-engine-domain-forge/src/boundary.rs:360`:

```rust
pub struct First {
    pub playground: bool,
    pub org: Box<[u8]>,
    pub repository: Box<[u8]>,
    pub project: SetUpProject,
    pub state: FirstState,
    pub label: Box<[u8]>,
}

pub enum SetUpProject { Existing(u32), New(Box<[u8]>) } // :323
pub enum FirstState {
    Making,
    Made { id: u64 },
    Failed { step: CreateStep },
} // :335
```

- Inserted by admitted `SetUp`: `domain.rs:1226`.
- Inserted by Forgejo-admin `Connect`: `connect.rs:214–228`.
- `pub fn first_made(&mut self, forge: u16, id: u64) -> bool`: `domain.rs:222`.
- `pub fn first_failed(&mut self, forge: u16, step: CreateStep) -> bool`: line 230.
- Exposed as `Row.first: Option<First>`: `boundary.rs:387`.

No `Stored::First` or first-intent record family was found.

## Service copy

`temper/crates/temper-service/src/setup/lifecycle.rs:53`:

```rust
pub(crate) struct Intent {
    pub(crate) playground: bool,
    pub(crate) label: Box<[u8]>,
    pub(crate) org: Box<[u8]>,
    pub(crate) repository: Box<[u8]>,
}
```

Holders:

- `Local.first: Option<Intent>` — `lifecycle.rs:67`.
- `Run.first: Option<Intent>` — line 35.
- `Lifecycle.reconcile: Map<u16, Reconcile>` — line 117.

Flow:

- `pub(crate) fn first(engine, forge, org, repository, playground)` — line 1330; assigns `local.first` at line 1341.
- `fn create_local(engine: &mut Engine, forge: u16) -> bool` — line 1000; takes the intent at line 1002 and makes `host::Plan::Create` at line 1018.
- Managed admin path: `fn admin_create(...)` — line 1416.
- Create success/failure: lines 335–338 and 1042–1080.

**Existing test explicitly requires volatility:**

`temper/tests/engine/domain/tests/qa_all/qa_first_repository.rs:199`:

```rust
fn first_repository_row_is_volatile_across_restart()
```

It forks the store and recreates the configuration; the first-repository row disappears. This test must change when intent becomes durable.

## Reverse dependencies

Besides the domain/service files above:

```text
temper/crates/temper-service/src/family.rs:61,816
temper/crates/temper-service/src/setup/public.rs:50
temper/crates/temper-service/src/setup/settings.rs:279
temper/crates/temper-service/src/tests/setup_create.rs:280,302,341
temper/crates/temper-service/src/tests/setup_admin.rs:67
temper/crates/temper-service/src/tests/family.rs:854
temper/crates/temper-service/src/tests/system.rs:349
temper/crates/temper-engine-domain-forge/src/tests.rs:1509,1803,1842
temper/crates/temper-engine-domain-forge/src/tests/connect.rs:634,760
temper/crates/temper-engine-domain-forge/src/tests/demo.rs:388
temper/crates/temper-engine-domain/src/tests/setup.rs:124,459,669
temper/crates/temper-client-domain/src/forge/forges.rs:55,67,91
temper/crates/temper-client-domain/src/forge/family.rs:186
temper/crates/temper-client-domain/src/forge/admin.rs:151
temper/crates/temper-client-domain/src/forge/listed.rs:62
temper/crates/temper-client-domain/src/forge/connect.rs:173
temper/crates/temper-client-domain/src/forge/words.rs:989
temper/crates/temper-client-domain/src/tests/connect.rs:451
temper/crates/temper-client-domain/src/tests/forges.rs:26
temper/crates/temper-client-domain/src/tests/admin.rs:26
temper/crates/temper-client-view/src/forge/connect.rs:17
temper/crates/temper-client-view/src/forge/forges.rs:117
temper/crates/temper-client-view/src/pages/welcome.rs:230
temper/crates/temper-client-view/src/tests/connect_forge.rs:525
temper/crates/temper-client-view/src/tests/forges.rs:583
temper/crates/temper-client-view/src/tests/welcome.rs:176
temper/crates/temper-client-tools/src/forges.rs:198
temper/crates/temper-client-tools/src/forges/connection.rs:125
temper/crates/temper-client-tools/src/tests/adopt.rs:323
temper/crates/temper-client-tools/src/tests/forges.rs:35
temper/crates/temper-client-tools/src/tests/forge_connection.rs:340
temper/crates/temper-client-shell-cli/src/tests/fixtures/setup.rs:64
temper/tests/client/src/translate/forge.rs:243
temper/tests/client/tests/suite/connect_forge.rs:120
temper/tests/client/tests/suite/forges.rs:453
temper/tests/engine/domain/tests/qa_all/qa_first_repository.rs:56,199
temper/tests/codecs/tests/suite/client_wire.rs:49
temper/tools/gallery/src/worlds.rs:417
```

Wire schema/generated dependencies are the `FirstAsk`, `FirstRepository`, `FirstState`, and `SetUpProject` declarations in question 3.

# 5. Forge-derived limits and `memory_bytes`

## Sum over configured forges

A deployment-limit calculation summing per-forge capacities was **not found in current code**.

Current behavior:

- `temper-service/src/limits.rs:145`: deployment-wide `FORGE: forge::Limits`, with `repositories: 2`.
- `limits.rs:131`: child client’s `repositories: 2`.
- `limits/names.rs:70`: `"forge.repositories"` override.
- `limits/names.rs:119`: `"forge.client.repositories"` override.
- `limits/names.rs:210`: `"forge_component.forges"` override.
- `forge.rs:167`: default connection component `forges: 4`.
- `temper-engine-domain-forge/src/limits.rs:17`: `Limits`; **no `forges` field**.

Couplings the new `forge.forges` count would replace or separate:

| Current use | Location |
|---|---|
| Maximum configured-forge JSON list | `temper-service/src/startup/application.rs:788`, bounded by `bounds.forge.client.repositories` |
| Empty forge list rejected | same file, lines 812–813 |
| Exactly one identity per configured forge | same file, lines 815–816 |
| `configured` map capacity | `temper-engine-domain-forge/src/domain.rs:507`, `l.client.repositories` |
| Writer map capacity | same file, line 522 |
| Initial `connections`, `first`, `ended`, `apps` capacities | lines 537–540 |
| Later connection-table capacities | `connection_limits(&mut self, forges: u32, installations: u32)`, line 197 |
| Service passes connection bounds | `temper-service/src/engine.rs:137` |
| Setup host forge capacity | `temper-service/src/setup/settings.rs:15,86` |

There are still configuration-dependent counts, but they are not a heap-budget sum:

- `skein/crates/skein-forge-connection/src/limits.rs:94`, `Forges::of(settings: &Settings) -> Option<Forges>` counts Forgejo and GitHub entries.
- `temper-service/src/startup/checks.rs:1017`, `record_writes`, collects maximum encoded secret-row sizes, including one origin per configured forge.
- `temper-service/src/system.rs:1098` sizes diagnostic room from configured forge count plus demo-act capacity.
- `startup/checks.rs:945` computes socket requirements.

## Memory budget

Current implementation corresponding to the draft’s reference to commit `272c945cc`:

```rust
// temper/crates/temper-engine-domain/src/limits.rs:35
pub memory_bytes: u64,

// temper/crates/temper-engine-domain/src/domain.rs:612
pub(crate) fn within_budget(domain: &Domain, limits: &Limits) -> bool {
    domain.held_bytes() < limits.memory_bytes
}
```

Wiring/checks:

- `jig-protocol-config/src/lib.rs:146`: parses `engine.memory_bytes`.
- `temper-service/src/startup.rs:285`: `engine_limits(...)`; sets `root.memory_bytes` at line 293.
- `startup.rs:305`: `check_memory_bytes`; rejects zero.
- `temper-service/src/system.rs:176`: `limits.memory_bytes = settings.jig.memory`.
- `temper-engine-domain/src/limits.rs:405`: zero invalid.
- `temper-engine-domain/src/domain.rs:882`: activation checks `within_budget`.

**The current meter is narrower than “every held allocation.”** `Domain::held_bytes()` at `domain.rs:100` adds root-owned charged bytes to `core.transcript_bytes()`. `budget.rs:1` identifies transcript bodies and completed brief text. It does not meter the whole configured-forge table.

Budget ownership helpers are `temper-engine-domain/src/budget.rs:4,12,20,32`; root charge/release methods are `domain.rs:104,108`.

Reverse dependencies relevant to changing this budget:

```text
temper/crates/temper-service/src/limits.rs:372
temper/crates/temper-service/src/startup.rs:285
temper/crates/temper-service/src/system.rs:174
temper/crates/temper-service/src/tests/engine.rs:575,589
temper/crates/temper-service/src/tests/startup.rs:884
temper/crates/temper-engine-domain/src/limits.rs:35,405
temper/crates/temper-engine-domain/src/domain.rs:100,104,108,612,882
temper/crates/temper-engine-domain/src/budget.rs:4
temper/crates/temper-engine-domain/src/assemble.rs:90
temper/crates/temper-engine-domain/src/loads.rs
temper/crates/temper-engine-domain/src/tests/budget.rs:26
temper/crates/temper-engine-domain/src/tests/resume_fixture.rs:161
temper/tests/engine/domain/src/walking.rs:173
temper/tests/engine/domain/src/direct.rs:69
temper/tests/engine/domain/tests/suite/budget.rs:110,150,220,238
temper/tests/engine/domain/tests/suite/root_routes.rs:1202
temper/crates/temper-init/template/temper.json:164
temper/crates/temper-init/src/tests/documents.rs:60
temper/docs/examples/owned.json:128
```

Documentation confirming the current budget rule: `temper/docs/design/protocol/configuration.md:884,1879`. The historical commit diff was not inspected in this code/docs-only survey.

# 6. Connector records, families, keys, and secrets

## Ordinary connector records

Domain declarations:

| Item | Location |
|---|---|
| `pub enum Stored` | `temper/crates/temper-engine-domain-forge/src/boundary.rs:802` |
| `pub enum Key` | same file, line 883 |
| `pub enum RecordKind` | `temper/crates/temper-engine-domain-forge/src/reconcile.rs:8` |
| `pub fn stored_key(row: &Stored) -> Key` | `temper-crates/temper-engine-domain-forge/src/lib.rs:222` — actual path is `temper/crates/temper-engine-domain-forge/src/lib.rs` |
| `pub fn stored_bytes(record: &Stored) -> Option<u64>` | same file, line 258 |
| Restore/admission | `temper-engine-domain-forge/src/restore.rs:53,233`; `domain.rs:4785` |
| Reference reconstruction | `temper-engine-domain-forge/src/reconcile.rs:136,146` |

Current encoded family bytes, from `temper/crates/temper-protocol-store/src/keys.rs:81`:

```text
0 Call             1 ProposedEffect    2 Repository
3 Hold             4 Names             5 Subscription
6 BranchHead       7 PullState         8 Ci
9 Landed          10 Entry            11 Client
12 Change         13 Issue            14 Release
15 Application    16 Writer           17 Demo
18 Binding        19 DemoAct          21 Flight
23 Landing        24 Receipt
```

`Setting` family 20 and `Delivery` family 22 appear in the design, but matching `Stored`/`Key` variants were **not found in current code**. The design also reserves later families; a new byte should not be inferred merely from gaps in the implementation.

Repository record:

```rust
// temper-engine-domain-forge/src/boundary.rs:98
pub struct Repository {
    pub project: u32,
    pub home: bool,
    pub provider: skein_forge::api::Repository,
    pub host: Box<[u8]>,
    pub owner: Box<[u8]>,
    pub name: Box<[u8]>,
    pub prefix: Box<[u8]>,
    pub role: Role,
    pub ci: bool,
    pub checks: Box<[u32]>,
    pub kinds: Kinds,
    pub protection: Protection,
    pub settings: skein_forge::api::Settings,
}
```

- `Stored::Repository(Repository)` — `boundary.rs:811`.
- `Key::Repository(api::Repository)` — line 886.
- Encoding: `temper-protocol-store/src/keys.rs:111`.
- Provider-key encoding: `keys.rs:244`, forge and repository both widened to key-writer `u64`.
- Value schema: `temper-protocol-store/schema/store-v10.schema:25`, `record Adopted`.

Codec/routing APIs:

```rust
// temper/crates/temper-protocol-store/src/keys.rs:81,185
pub fn encode(key: &forge::Key, limit: u32) -> Option<Box<[u8]>>
pub fn decode(bytes: &[u8], limits: &forge::Limits) -> Option<forge::Key>

// temper/crates/temper-protocol-store/src/records.rs:22
pub fn encode(value: &d::Stored) -> Box<[u8]>
```

`STORE_VERSION: u32 = 10` is `temper-protocol-store/src/lib.rs:15`. Schema: `schema/store-v10.schema`; generated code: `src/generated/v10.rs`.

Root wrapping/routing:

- `temper-engine-domain/src/store.rs:533`: `Record::Forge { row: Box<Stored> }`.
- `temper-service/src/store/route.rs:107–112`: encode root saves/erases into application puts/removes.
- Same file, line 404: decode application bytes.
- `temper-service/src/store/ranges.rs:81`: `Range::Forge` restores the complete application interval.

**Stored forge configuration, first intent, and next-forge-number records: not found.**

## Adding a family

The existing architecture requires these concrete touchpoints:

1. Add domain payload, `Stored` and `Key` variants in `boundary.rs`.
2. Add `stored_key` and `stored_bytes` cases in `lib.rs`.
3. Add domain storage, save/erase behavior and restoration in `domain.rs`.
4. Add capacity/field checks in `restore.rs`; add `RecordKind` and references in `reconcile.rs`.
5. Add record and `Value` variant in `temper-protocol-store/schema/store-v10.schema`.
6. Add key encode/decode cases in `temper-protocol-store/src/keys.rs`, preserving assigned family bytes and ordering.
7. Add total value translations in `temper-protocol-store/src/records.rs` or a child module; update bounds in `lib.rs`/`records.rs` if variable fields require them.
8. Regenerate `src/generated/v10.rs` and goldens.
9. Extend codec, key-order, restore-fit, reconciliation, restart and domain/world coverage.
10. Update `temper/docs/design/protocol/store.md` and its status file.

Generation/drift verification is implemented in `temper/tests/codecs/tests/suite/drift.rs:5`; store and secrets checks are at lines 42 and 47. It uses `skein_codegen::{parse, generate}`. Its documented write mode is at lines 12–17. Nothing was generated during this survey.

### Recent implemented example: `Flight`

`temper/docs/design/protocol/store.status.md:6` explicitly records addition of family **21 Flight**. Its implementation footprint is:

```text
temper/crates/temper-engine-domain-forge/src/boundary.rs:863,911
temper/crates/temper-engine-domain-forge/src/domain.rs:148,2274,4148
temper/crates/temper-engine-domain-forge/src/lib.rs:252,413
temper/crates/temper-engine-domain-forge/src/reconcile.rs:29,245
temper/crates/temper-engine-domain-forge/src/restore.rs:233
temper/crates/temper-engine-domain-forge/src/tests/flight.rs:10
temper/crates/temper-protocol-store/schema/store-v10.schema:218,244
temper/crates/temper-protocol-store/src/generated/v10.rs
temper/crates/temper-protocol-store/src/keys.rs:12,167,210
temper/crates/temper-protocol-store/src/records.rs:45,107
temper/crates/temper-protocol-store/src/tests/keys.rs:183
temper/crates/temper-protocol-store/src/tests/records.rs:907
temper/crates/temper-protocol-store/src/tests/fixtures.rs:177
temper/crates/temper-protocol-store/golden/record_flight_smallest.bin
temper/crates/temper-protocol-store/golden/record_flight_full.bin
temper/crates/temper-protocol-store/golden/enum_value_flight_smallest.bin
temper/crates/temper-protocol-store/golden/enum_value_flight_full.bin
temper/tests/engine/forge/tests/suite/memory.rs:80
temper/tests/engine/domain/tests/suite/forge_routes/flights.rs:18
temper/docs/design/protocol/store.md:91
temper/docs/design/protocol/store.status.md:6
temper/docs/design/domain/forge.status.md:5
```

This is the current implementation file list, **not a verified historical commit’s changed-file list**; that cannot be established from source/docs alone.

## Credential records: separate store and namespace

`temper-protocol-records` is not the ordinary connector-family codec. It holds forge credential records in jig’s separate secrets store.

`temper/crates/temper-protocol-records/src/lib.rs:18–54`:

```rust
pub const FAMILY_FORGE: u16 = 1;
pub const VERSION: u16 = 1;
pub const CARRIED: u64 = 1 << 63;

pub enum Item {
    AdminToken = 1,
    ApiToken, GitToken, User, WebhookSecret,
    AppId, AppSlug, AppKey, Accepted, Origin,
}

pub fn number(forge: u16, item: Item) -> u64 {
    (u64::from(FAMILY_FORGE) << 48)
        | (u64::from(forge) << 16)
        | u64::from(item.code())
}
```

`parse(number: u64) -> Option<(u16, u16, Item)>` is at line 57.

Schema: `temper/crates/temper-protocol-records/schema/secrets-v1.schema:7–30`. It declares each item and `versioned record Stored { item: Item }`.

Runtime grouping, `temper-protocol-records/src/forge.rs`:

```rust
// :53
pub struct Rows { forges: Map<u16, ForgeRows> }

// :57
pub struct ForgeRows {
    values: [Option<Secret>; 10],
    unreadable: bool,
    unreadable_items: u16,
}
```

- `Rows::push(&mut self, number: u64, bytes: Secret, value_bytes: u32)` — line 69.
- `Rows::forge(&self, forge: u16) -> Option<&ForgeRows>` — line 103.
- `ForgeRows::get(&self, item: Item) -> Result<&[u8], NotConnected>` — line 109.
- `access(kind, expected, rows, bounds, webhooks) -> Result<Start, NotConnected>` — line 165.
- Origin includes kind and exact API address; mismatch refuses connection, without erasure.
- Credential loading: `temper-service/src/forge/records.rs:50`, `Loader`; configured forge iteration at line 155.
- Credential writes: `temper-service/src/setup/lifecycle.rs:267`, `keep`.

Credential namespace reverse dependencies:

```text
temper/crates/temper-protocol-records/src/codec.rs:19,36
temper/crates/temper-protocol-records/src/forge.rs:2
temper/crates/temper-service/src/forge/records.rs:5
temper/crates/temper-service/src/forge/tokens.rs:151
temper/crates/temper-service/src/forge.rs:471
temper/crates/temper-service/src/setup.rs:15
temper/crates/temper-service/src/setup/lifecycle.rs:12
temper/crates/temper-service/src/setup/public.rs:188
temper/crates/temper-service/src/startup/checks.rs:1052
temper/crates/temper-service/src/tests/setup.rs:125
temper/crates/temper-service/src/tests/setup_lifecycle.rs:14
temper/crates/temper-service/src/tests/setup_bound_webhook.rs:17
temper/crates/temper-service/src/tests/setup_create.rs:238
temper/crates/temper-service/src/tests/setup_adoption.rs:118
temper/crates/temper-service/src/tests/setup_admin.rs:28
temper/crates/temper-service/src/tests/setup_github.rs:329
temper/crates/temper-service/src/tests/forge.rs:594
temper/crates/temper-service/src/tests/limits.rs:296
temper/crates/temper-service/src/tests/fixtures.rs:1059
temper/crates/temper-client-domain/src/forge/forges.rs:803
temper/tests/codecs/tests/suite/records.rs:4
temper/tests/codecs/tests/suite/fuzzy/records.rs:3
temper/tests/codecs/tests/suite/drift.rs:47
temper/testing/temper-github-app/src/lib.rs:4
```

### Ordinary record-enum reverse dependencies

Production readers/constructors requiring attention when extending `Stored`/`Key`/`RecordKind`:

```text
temper/crates/temper-engine-domain-forge/src/domain.rs:10
temper/crates/temper-engine-domain-forge/src/lib.rs:222
temper/crates/temper-engine-domain-forge/src/adoption.rs:4
temper/crates/temper-engine-domain-forge/src/capabilities.rs:3
temper/crates/temper-engine-domain-forge/src/demo.rs:3
temper/crates/temper-engine-domain-forge/src/landing.rs:4
temper/crates/temper-engine-domain-forge/src/reconcile.rs:2
temper/crates/temper-engine-domain-forge/src/restore.rs:3
temper/crates/temper-engine-domain/src/store.rs:303,533
temper/crates/temper-engine-domain/src/route.rs:282,555
temper/crates/temper-engine-domain/src/route/forge_route.rs:3088
temper/crates/temper-protocol-store/src/keys.rs:81
temper/crates/temper-protocol-store/src/records.rs:22
temper/crates/temper-protocol-store/src/records/application.rs:40
temper/crates/temper-protocol-store/src/records/entries.rs:120
temper/crates/temper-protocol-store/src/records/landing.rs:320
temper/crates/temper-service/src/store/route.rs:107,404
temper/crates/temper-service/src/store/ranges.rs:56,81
temper/crates/temper-fake/src/check.rs:94
temper/crates/temper/src/host.rs:686
```

Direct domain/codec tests:

```text
temper/crates/temper-engine-domain-forge/src/tests.rs:217
temper/crates/temper-engine-domain-forge/src/tests/adoption.rs:10
temper/crates/temper-engine-domain-forge/src/tests/binding.rs:7
temper/crates/temper-engine-domain-forge/src/tests/brief.rs:48
temper/crates/temper-engine-domain-forge/src/tests/change.rs:17
temper/crates/temper-engine-domain-forge/src/tests/demo.rs:79
temper/crates/temper-engine-domain-forge/src/tests/fixtures.rs:120
temper/crates/temper-engine-domain-forge/src/tests/flight.rs:10
temper/crates/temper-engine-domain-forge/src/tests/landing.rs:6
temper/crates/temper-engine-domain-forge/src/tests/limits.rs:70
temper/crates/temper-engine-domain-forge/src/tests/panel.rs:10
temper/crates/temper-engine-domain-forge/src/tests/projection.rs:106
temper/crates/temper-engine-domain-forge/src/tests/reconcile.rs:13
temper/crates/temper-engine-domain-forge/src/tests/restore_fit.rs:14
temper/crates/temper-engine-domain-forge/src/tests/stopping.rs:6
temper/crates/temper-engine-domain-forge/src/tests/views.rs:35
temper/crates/temper-engine-domain/src/tests/setup.rs:427
temper/crates/temper-engine-domain/src/tests/restore_fit.rs:23
temper/crates/temper-engine-domain/src/route/forge_route/tests.rs:195
temper/crates/temper-protocol-store/src/tests/fixtures.rs:82
temper/crates/temper-protocol-store/src/tests/holds.rs:26
temper/crates/temper-protocol-store/src/tests/keys.rs:4
temper/crates/temper-protocol-store/src/tests/records.rs:30
temper/crates/temper-protocol-store/src/tests/sizes.rs:34
temper/crates/temper-fake/src/tests/check.rs:98
temper/crates/temper/src/fake/check/tests.rs:225
temper/crates/temper-service/src/tests/fixtures.rs:304
temper/crates/temper-service/src/tests/store.rs:124
temper/crates/temper-service/src/tests/store_route.rs:25
temper/crates/temper-service/src/tests/setup.rs:806
temper/crates/temper-service/src/tests/setup_adoption.rs:220
temper/crates/temper-service/src/tests/setup_admin.rs:758
temper/crates/temper-service/src/tests/engine.rs:440
temper/crates/temper-service/src/tests/stop.rs:196
temper/crates/temper-service/src/tests/sign_in.rs:264
temper/crates/temper-service/src/clients/google/tests.rs:139
temper/tests/codecs/tests/support/protection.rs:18
temper/tests/codecs/tests/suite/fuzzy/store.rs:32
temper/tests/codecs/tests/suite/store_queue.rs:5
temper/tests/codecs/tests/suite/store_protection.rs:33
```

Additional direct record readers in worlds/referees:

```text
temper/tests/engine/domain/src/forge_routes.rs:967
temper/tests/engine/domain/tests/qa_all/qa_github_land.rs:25
temper/tests/engine/domain/tests/qa_all/qa_change_panel.rs:119
temper/tests/engine/domain/tests/suite/root_routes.rs:973
temper/tests/engine/domain/tests/suite/issue_projection.rs:86
temper/tests/engine/domain/tests/suite/demo.rs:137
temper/tests/engine/domain/tests/suite/mark.rs:50
temper/tests/engine/domain/tests/suite/forge_routes.rs:192
temper/tests/engine/domain/tests/suite/restore_fit.rs:8
temper/tests/engine/domain/tests/suite/loads.rs:320
temper/tests/engine/domain/tests/suite/reconcile.rs:12
temper/tests/engine/domain/tests/suite/settings/b2.rs:304
temper/tests/engine/domain/tests/suite/forge_routes/plain_change.rs:27
temper/tests/engine/domain/tests/suite/forge_routes/reviews.rs:255
temper/tests/engine/domain/tests/suite/forge_routes/hints.rs:19
temper/tests/engine/domain/tests/suite/forge_routes/result.rs:96
temper/tests/engine/domain/tests/suite/forge_routes/change_tool.rs:712
temper/tests/engine/domain/tests/suite/forge_routes/asks_no_one.rs:116
temper/tests/engine/domain/tests/suite/forge_routes/flights.rs:18
temper/tests/engine/domain/tests/suite/forge_routes/landing.rs:182
temper/tests/engine/domain/tests/suite/forge_routes/saved_retry.rs:280
temper/tests/engine/domain/tests/suite/forge_routes/producer_failure.rs:65
temper/tests/engine/domain/tests/suite/forge_routes/b2_tree.rs:330
temper/tests/engine/forge/src/world.rs:102
temper/tests/engine/forge/tests/qa_kept_change.rs:34
temper/tests/engine/forge/tests/suite/memory.rs:80
temper/tests/engine/forge/tests/suite/writer.rs:15
temper/tests/engine/forge/tests/suite/fuzzy/simulation.rs:29
temper/tests/engine/forge/tests/suite/replay.rs:27
temper/tests/engine/forge/tests/suite/simulation.rs:29
temper/tests/engine/forge/tests/suite/landing.rs:34
temper/tests/engine/forge/tests/suite/protection.rs:33
temper/tests/engine/protocol-forge/src/protocol.rs:21
temper/tests/engine/protocol-forge/tests/qa_all/qa_github_land.rs:62
temper/tests/engine/protocol-forge/tests/qa_all/qa_rate.rs:300
temper/tests/engine/protocol-forge/tests/suite/protocol.rs:130
temper/tests/engine/protocol-forge/tests/suite/fuzzy/protocol.rs:17
temper/tests/system/src/composed.rs:322
temper/tests/system/src/forge/stories.rs:215
temper/tests/one-process/src/world.rs:647
temper/tests/one-process/src/stories.rs:261
temper/tests/one-process/src/fixtures.rs:18
temper/tests/one-process/src/referee.rs:167
temper/tests/one-process/src/referee/restart.rs:4
temper/tests/one-process/src/m15/observer.rs:102
temper/tests/one-process/tests/suite/one_process.rs:34
temper/tests/conformance/src/scenarios.rs:39
temper/tests/conformance/src/stories.rs:47
temper/tests/conformance/src/systems.rs:128
temper/tests/conformance/src/lib.rs:339
temper/tests/conformance/src/observations.rs:228
temper/tests/client/src/front.rs:295
temper/tests/client/tests/qa_all/qa_first_repository.rs:315
temper/tests/client/tests/suite/change_panel.rs:116
temper/tools/gallery/src/samples.rs:97
```

# 7. Engine worlds and document fixtures

## World layout

| Crate | Location | How forges are supplied |
|---|---|---|
| `temper-engine-domain-world` | `temper/tests/engine/domain/Cargo.toml:2` | Direct root `Config`, fake ordered/paged store, scripted workers/person; `src/direct.rs:14`, `Driver`. |
| `temper-engine-forge-world` | `temper/tests/engine/forge/Cargo.toml:2` | Connector domain plus fake forge; `src/world.rs:231`, configured constructor. |
| `temper-engine-protocol-forge-world` | `temper/tests/engine/protocol-forge/Cargo.toml:2` | Connector and real forge protocol machinery against fakes; `src/protocol.rs:138`. |
| `temper-system-world` | `temper/tests/system/Cargo.toml:2` | Composed engine/worker domains; `src/world.rs:63`, `config`; `src/composed/link.rs:30` builds service forge settings. |
| One-process world | `temper/tests/one-process/` | Parsed deployment document plus simulated services/machine and restart referee. |
| Real-loop world | `temper/tests/real-loop/` | Parsed deployment document, real loop and fake peers. |
| Shared testing kits | `temper/testing/temper-fake-git-hosted/`, `temper/testing/temper-github-app/` | Fake git/backend and GitHub App helpers; not the main engine world. |

The standard direct root fixture is:

`temper/tests/engine/domain/src/walking.rs:269`, `pub fn config(seed: u64) -> engine::Config`.

Its forge portion at lines 352–362 is:

```rust
forge: skein_forge_client::Config {
    namespace: Box::from(&b"root"[..]),
    writers: Box::new([skein_forge_client::Writer { forge: 1, author: 1 }]),
},
identities: Box::new([GitIdentity { forge: 1, credential: 1 }]),
forge_charter: 1,
forges: Box::new([]),
under_box: false,
demo: None,
```

Thus ordinary direct domain tests can have adopted repositories/writers without a public configured-forge list.

Setup-specific worlds override `config.forges` with `Configured` entries:

- `temper/tests/engine/domain/tests/qa_all/qa_first_repository.rs:16`.
- `temper/tests/engine/domain/tests/suite/demo.rs:31`.
- `Driver::with_repositories` is at `temper/tests/engine/domain/src/direct.rs:100`.
- Adoption fixture: `temper/tests/engine/domain/src/fixtures.rs:19`, `pub fn adoption(forge: u16) -> forge::Adoption`.

## Document fixtures naming forges

Representative legacy shape:

`temper/tests/one-process/src/fixtures.rs:132–140`:

```json
{
  "forges": [{
    "number": 1, "kind": "forgejo",
    "api": {"socket": "..."},
    "transport": "plaintext",
    "git": "...",
    "webhook_secret": "secrets/forge-1.webhook-secret",
    "api_token": "secrets/forge.api-token",
    "user": {"file": "secrets/forge-1.user"},
    "identity": 1
  }],
  "identities": [{
    "number": 1,
    "token": "secrets/forge.git-token",
    "login": "worker",
    "author": {"name": "Worker", "email": "worker@example.invalid"}
  }],
  "setup": [], "demo": null
}
```

This differs from init’s current `identities[].forge` and records-based access. Both forms must be accounted for.

### Fixture/deployment constructors and mutations

These are reverse dependencies of the deployment forge/identity/setup/demo members:

```text
temper/docs/examples/owned.json:175,186
temper/crates/temper-init/template/temper.json:211,241,303,358
temper/crates/temper-init/src/tests/documents.rs:84
temper/crates/temper-service/src/tests/fixtures/startup.rs:12,61,296
temper/crates/temper-service/src/tests/startup.rs:336,528,717,991,1845
temper/crates/temper-service/src/tests/checks.rs:282,326,366
temper/crates/temper-service/src/tests/root_checks.rs:90,102,220
temper/crates/temper-service/src/tests/stock.rs:49,58,103
temper/crates/temper-service/src/tests/forge.rs:403,422,479
temper/crates/temper-service/src/tests/family.rs:1394
temper/crates/temper-service/src/tests/welcome_policy.rs:65
temper/crates/temper-forge-setup/src/tests/read.rs:77
temper/crates/temper-forge-setup/src/tests/given.rs:8
temper/crates/temper-forge-setup/src/tests/managed.rs:44
temper/crates/temper-forge-setup/src/tests/members.rs:19
temper/crates/temper-forge-setup/src/tests/fetch.rs:95
temper/crates/temper-fake/src/tests/demo.rs:86
temper/crates/temper-fake/src/tests.rs:698,972
temper/crates/temper-fake/src/backends/tests.rs:125,195,250
temper/crates/temper/src/tests.rs:190,318,388
temper/crates/temper/src/forge/tests.rs:49
temper/crates/temper/src/init/tests.rs:146,161,175,186
temper/crates/temper/src/fake/check/tests.rs:400
temper/crates/temper/src/fake_backends.rs:354
temper/crates/temper-forge-setup-shell/tests/suite/reconcile.rs:274
temper/tests/engine/domain/tests/fixtures/stock.rs:4
temper/tests/engine/domain/src/forge_routes.rs:1921
temper/tests/one-process/src/fixtures.rs:105,135
temper/tests/one-process/src/stories.rs:311
temper/tests/one-process/src/lower/github.rs:40
temper/tests/real-loop/src/fixtures.rs:16,55,148
temper/tests/real-loop/src/forge.rs:49
temper/tests/real-loop/src/github.rs:152
temper/tests/real-loop/tests/suite/github.rs:39
temper/tests/client-browser/src/configuration.rs:84,164
temper/tests/client-browser/src/lib.rs:204,338
temper/tests/client/tests/suite/connect_forge.rs:302
temper/tests/client/tests/suite/welcome.rs:209
temper/crates/temper-client-view/src/tests/connect_forge.rs:236
temper/crates/temper-client-tools/src/tests/forges.rs:149
temper/crates/temper-client-shell-web/src/host_tests.rs:457
temper/tools/gallery/src/samples.rs:308
```

Binary/process fixtures and tests:

```text
temper/crates/temper/tests/fixtures/cluster/engine.json:187,198
temper/crates/temper/tests/support/github/engine.json:176,188
temper/crates/temper/tests/support/configuration.rs:65,124
temper/crates/temper/tests/support/owned.rs:27,37,45
temper/crates/temper/tests/support/case.rs:204
temper/crates/temper/tests/support/webhooks.rs:32
temper/crates/temper/tests/support/forge_sign_in.rs:27
temper/crates/temper/tests/support/forgejo_elsewhere.rs:150
temper/crates/temper/tests/support/first_repository.rs:457,546,732
temper/crates/temper/tests/support/forge_land.rs:670,766,1572
temper/crates/temper/tests/support/forge_land/rung.rs:827
temper/crates/temper/tests/support/github.rs:227
temper/crates/temper/tests/support/github/existing.rs:166
temper/crates/temper/tests/support/github_land.rs:212,815
temper/crates/temper/tests/support/live_github.rs:155,323
temper/crates/temper/tests/support/csv_repair.rs:106
temper/crates/temper/tests/support/landing_by_turn.rs:32
temper/crates/temper/tests/support/account_file_qa.rs:31
temper/crates/temper/tests/support/account_move.rs:46
temper/crates/temper/tests/support/account_move_qa.rs:51
temper/crates/temper/tests/qa_all/qa_adoption.rs:30,97,244
temper/crates/temper/tests/qa_all/qa_first_run.rs:21,167
temper/crates/temper/tests/qa_all/qa_first_repository.rs:106
temper/crates/temper/tests/qa_all/qa_forge_handover.rs:42,217,481
temper/crates/temper/tests/qa_all/qa_forge_setup.rs:44,131
temper/crates/temper/tests/process.rs:823,2403,2601,3903,4600
temper/crates/temper/tests/suite/startup.rs:327,409,509,1503
temper/crates/temper/tests/suite/forge.rs:325,450,556
temper/crates/temper/tests/suite/fake.rs:448,554,753
temper/crates/temper/tests/fake_backends/walk.rs:117
temper/crates/temper/tests/fake_backends/qa.rs:270,466
```

A world exercising setup interruption and recovery **with durable first intent** was not found. The existing domain-world test at `qa_first_repository.rs:199` explicitly tests that the intent is lost on restart.
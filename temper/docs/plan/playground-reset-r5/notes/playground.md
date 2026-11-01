# Part note: `playground` and `client` (playground-reset-r5)

The shapes inside parts `playground` (`pg-*`) and `client` (`cl-*`) that
several of their tasks share. It adds to `notes/contracts.md` and never
changes it. Draft: `temper/docs/design/draft/playground-reset.md`, 6.1 to
6.3 and section 9's first default (a start resets a running playground).

## 1. The `Playground` row (store family 17, `Key::Playground`)

In `temper_engine_domain_forge::boundary`, replacing `Demo`/`DemoState`:

```rust
pub enum PlaygroundState { Running, Resetting }
pub struct Playground {
    pub state: PlaygroundState,
    /// The person whose start made it: `Act::MakeProject`'s maker, so a
    /// resend is the same act.
    pub maker: u64,
    /// Each set in the decision that learns it, cleared when its reset step answers.
    pub forge: Option<u16>,
    pub project: Option<u32>,
    pub account: Option<u32>,
}
```

- No row: no playground. `Running` from the start's first decision until
  `EndPlayground` (or a start, section 4 below) turns it `Resetting`;
  the row is erased when the reset's last step answers. There is no
  ended state: `DemoState::Ended` and what reads it go.
- Its codec: `temper-protocol-store`'s record `Playground` (family 17,
  the key the family byte alone), in the store version the trunk is at
  when `pg-store` starts if this run already bumped it, else the next.

## 2. Keys of the playground's acts

Every deployment act the playground asks is keyed so a resend is the same
act and a later playground's never meets an earlier one's answer: the
16-byte key is `sha256("temper playground\0" || step || forge as 2
bytes BE || subject as 4 bytes BE)`, first 16 bytes, where `forge` is the
playground's forge number (never reused) and `step` and `subject`:

| Step | byte | subject | Act |
|---|---|---|---|
| make the project | 1 | 0 | `Act::MakeProject { maker, name: "Playground", policy }` |
| add the account | 2 | 0 | `Act::AddKey { provider, join: None, digest }` (the key file's value carried) |
| its uses | 3 | account | `Act::SetUses { account, uses: Some([Use::Project(project)]) }` (task `pg-jig-uses`) |
| adopt the repository | 4 | project | `Act::Adopt { project, adoption }` (`playground/widget`) |
| delete the project | 5 | project | `Act::RemoveProject { project }` (kept: `KeptAct::RemoveProject`) |
| remove the account | 6 | account | `Act::RemoveAccount { account }` (kept: `KeptAct::RemoveAccount`) |

The fake forge is not an act: `Domain::make_forge(NewForge::Playground, ..)`
at the start, `Domain::remove_forge(forge, ..)` at the reset.

## 3. The start (`StartPlayground`)

- **Offered** while no forge record other than the playground's is kept
  (any `ForgeRecord` whose `mode` is not `Mode::Fake`, removal or not),
  and no row reads `Resetting`; else answered *not offered*.
- **First decision** (no row): `make_forge(NewForge::Playground)`, then the
  row saved `Running` with `maker` the start's sign-in's person and
  `forge: Some(n)`. The box is asked for its group by `make_forge`.
- **Then, in order, each once** (a `Running` row with a field still `None`
  asks its step again with its key): the project (step 1), the account
  (step 2), its uses (step 3), and once the forge connects the adoption
  (step 4). Each answer's number is saved into the row in the decision
  that hears it. The start's answer is `Running` once the adoption is
  answered (adopted, or already adopted).
- **The project's policy:** the stock's playground policy, whose charters
  allow the scripted model alone; no other stock charter allows it
  (draft 6.1). Its number is the stock's (`pg-start` names it).
- **Grants** (`temper init`'s template, `engine.acts`, connector 1):
  `make_project`, `add_key`, `set_uses`, `adopt`, `cancel`,
  `remove_project`, `remove_account`; never `close`.

## 4. The reset (`EndPlayground`, and every start with a row)

- **One decision:** the row saved `Resetting`; the first step's kept row
  saved with it. Steps in order, each skipped when its field is `None`:
  1. `KeptAct::RemoveProject { project }` (step 5), sent; on its answer
     (`ProjectRemoved`, or unknown: the project is gone) the kept row
     erased and the row's `project` cleared, in one decision;
  2. in that same decision (or the first, with no project),
     `remove_forge(forge)`, and the row's `forge` cleared: the forge is
     ended in that decision, and the forge connector keeps the rest of its
     removal itself;
  3. `KeptAct::RemoveAccount { account }` (step 6) saved and sent in that
     decision; on its answer (removed, or unknown) erased with the row's
     `account` cleared;
  4. with all three `None`, the row erased; every waiting `EndPlayground`
     answered `Reset`.
- **At a start** (draft, section 9's default), a `Running` row is reset as
  `EndPlayground` resets it, after its unknown numbers are learned: a
  `project: None` resends step 1 (same key, same act: jig answers its
  retained answer, or makes it now; *unknown* means none was made); an
  `account: None` takes the deployment-held account of the scripted
  provider, if any, as today's end selection does
  (`temper-engine-domain/src/route/demo.rs`, `list_end_acts`). A
  `Resetting` row's kept rows are resent as every kept row is.
- `EndPlayground` with no row answers `Reset` at once (idempotent); with
  a `Resetting` row it joins the reset under way.

## 5. The client wire (`temper-client-protocol-wire`, task `pg-wire`)

Beside the forge family's requests, replacing `start_demo`, `Read::demo`
and `DemoAnswer` (the old forms go in `pg-demo-out`):

```
enum PlaygroundState { offered, not_offered, running, resetting }
versioned record PlaygroundAnswer { state: PlaygroundState forge: option u16 project: option u32 provider: u32 owner: bytes 1024 name: bytes 1024 cues: list 8 text 256 }
enum Read { demo, playground }            # demo leaves in pg-demo-out
enum Request { ..., start_playground, end_playground }
enum Outcome { ..., reset }               # EndPlayground's answer: the row is none
```

- The read always answers a record (no empty answer): `offered` (no row,
  offered), `not_offered` (no row, a real forge kept), `running`,
  `resetting`. `forge`/`project` as the row has them; `provider`, `owner`,
  `name` (the playground's repository, `playground`/`widget`) and `cues`
  from the playground's stock.
- `StartPlayground` answers as `StartDemo` does today (`running`,
  `start_refused`, `start_failed`); `EndPlayground` answers `reset`, or
  `unknown` past retention. The client sends `EndPlayground` again with a
  new key when it reads `resetting` (a reset a restart cut), and so learns
  its end.
- The welcome's three states (draft 6.3) are the client's, from what the
  engine reports: the read's state (offered or not), the Forges watch's
  rows (records, their connection and first repository) and the accounts
  (a usable model of a real provider). Empty: offered and no account of a
  real provider. Unfinished: a real forge kept but not connected with its
  first repository, or no real account usable. Whole: neither.

## 6. The client (`temper-client-domain`, `temper-client-view`)

- `temper_client_domain::forge::Playground` replaces `Demo`:
  `{ state: PlaygroundState, forge: Option<u16>, project: Option<u32>,
  provider: u32, owner, name, cues }`, `PlaygroundState { Offered,
  NotOffered, Running, Resetting }`. `forge/playground.rs` replaces
  `forge/demo.rs`: the read once per live link, the start (`Starting`,
  its failure, retry with a new key) and the end (`Leaving { request }`).
  No halves, no retired identity, no deployment-account watch, no
  automatic account removal: the reset is the engine's.
- Acts: `Act::StartPlayground`, `Act::EndPlayground` (the banner's link).
  While `Leaving` or the read says `resetting`, every page shows "Leaving
  the playground…"; on `reset` the client reads the playground and the
  forges again and opens `/welcome?at=forge`.
- The banner while the playground runs, on every page but the welcome:
  one line, "This is the playground: a scripted model and in-memory
  forge. Use the wizard to [set up temper]", the bracketed words the link
  sending `EndPlayground`. The setup strip, the demo banner's Connect
  links and the Playground sections of Forges and Accounts go; Forges
  lists the playground forge's row, Accounts the Scripted model, neither
  with anything to add while it runs.

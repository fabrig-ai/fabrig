# look-round-3: the shapes that cross tasks

The run's binding calls. A task builds to these; a change is a question. Design is
`temper/docs/design/client/README.md`, 3.5 (*The welcome*, *Home*, *Settings*) and
`temper/docs/design/client/browser.md`, 2.3.

## Addresses (`temper-client-domain`, `address.rs`)

```rust
pub enum Address {
    ...
    Home { project: u32 },       // /projects/<project>
    Settings { project: u32 },   // /projects/<project>/settings
}
```

- The design settles the routes, and the request's differing hrefs go in the account: "+ New chat"
  is `Address::Chats { filter: ChatsFilter::All, start: false }`'s start, `start: true`, printing
  `/chats?at=start` where the request drew `/chats/new`; "Open Forgejo ↗" is
  `forge::forges::open_link`'s external address where it drew `/forges/1/open`.
- The brand's link and Home's rail entry go to `Home { project }`, the open project; "Needs you" to
  `Inbox { filter: Filter::ALL }`, `/`; "Settings" to `Settings { project }`.
- On `Home` or `Settings`, the address's project is the open project once `me` lists it. The
  switcher's `Act::Project { project }`, pressed on either, shows that project's same page.

## The first repository (`temper-client-domain`, `pages/welcome.rs`)

```rust
/// What Ready adopts (client/README.md, 3.5): a first repository made and adopted nowhere.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct First {
    pub forge: u16,
    pub org: Box<[u8]>,
    pub name: Box<[u8]>,
}
```

`Pages` holds `pub(crate) welcome_first: Option<welcome::First>`, set in `route.rs`'s
`playground_facts` beside `welcome_decision`, from `forge::listed::unadopted_first(root.forges())`.
`Pages::welcome_first()` reads it.

## Ready's name (`pages/welcome.rs`)

```rust
pub struct DefaultProject {
    /// What the person wrote in the field; none until they write.
    pub written: Option<Box<[u8]>>,
    ...
}
impl DefaultProject {
    /// The field's value: what was written, else the offered name.
    pub fn shown<'a>(&'a self, offered: &'a [u8]) -> &'a [u8];
    /// What Create asks for: `shown` without its outer blanks.
    pub fn asked<'a>(&'a self, offered: &'a [u8]) -> &'a [u8];
}
/// The offered name (client/README.md, 3.5).
pub fn offered(first: Option<&First>) -> Box<[u8]>;
pub const OFFERED_NAME: &[u8] = b"My project";
```

`DEFAULT_NAME` and `DefaultProject::name` go. The field's written count stays
`temper_client_domain::pages::shown(root.pages())`.

## Ready's adoption (`pages/welcome.rs`, `pages.rs`, `forge.rs`)

`Pages` owns it, as it owns Ready's request: the forge side drops any answer with no open form.

```rust
/// Ready's adoption, or Home's again (client/README.md, 3.5).
#[derive(Debug, PartialEq, Eq)]
pub struct Adopting {
    pub project: u32,
    pub first: First,
    /// The request out, if one is.
    pub out: Option<jig_client::Key>,
    /// Each attempt after a refusal names a key of its own.
    pub attempt: u64,
    pub ended: Option<AdoptionEnded>,
}
#[derive(Debug, PartialEq, Eq)]
pub enum AdoptionEnded {
    Refused(crate::forge::Answer),
    Unknown,
}
```

- `Pages::adopting() -> Option<&Adopting>`; `pages::Act::AdoptFirst`, Home's "Adopt ORG/NAME".
- The request, ctl's (`temper-client-tools/src/forges.rs`, 1341): `forge::Adoption { project, forge,
  owner: org, name, prefix: b"temper/", role: Role::Owned, home: true, landing: b"main", ci: true,
  checks: [] }`, through `forge::ask`, sent as `pages::Request::Client(jig_client::Event::Send {
  key, name: Some(..), request })` with a `Pages` key.
- Its name: `forge::first_repository_key(forge, false)` while no refusal has answered, then
  `forge::first_repository_again_key(forge, attempt)`, `first-repository-again-<forge>-<attempt>`.
- The answer goes through `forge::answer`. `Answer::Adopted` clears `ended`; anything else ends it.

## Home's facts (`temper-client-domain`, `pages/home.rs`)

```rust
/// The facts Home draws (client/README.md, 3.5, Home).
pub struct Facts {
    pub configured: bool,
    pub repository: Repository,
    pub chat_asked: bool,
}
pub enum Repository {
    /// Not known: the `forges` watch has delivered nothing yet.
    Unknown,
    In { owner: Box<[u8]>, name: Box<[u8]> },
    Adopting(welcome::First),
    /// A first repository to adopt, with the last refusal's answer.
    Offer { first: welcome::First, refused: Option<crate::forge::Answer> },
    /// GitHub connected, with no first repository; its first accepted account.
    GitHub { account: Option<Box<[u8]>> },
    None,
}
impl Facts {
    /// The steps left: step 2 counts unless it is `In`.
    pub fn left(&self) -> u8;
}
pub fn facts(root: &Domain, project: u32) -> Facts;
```

- `Home` holds the `Subject::Settings` watch for as long as `Address::Home` is open, as `Inbox` holds
  its own. `configured`: its `SettingsDocument.settings` has a line whose `follows` is
  `SettingFollows::Change`; refused, ended or not delivered, `false`.
- `Pages` keeps its `chats::Chats` on `Address::Home` too, filter `All`, so that `Act::FirstChat`
  and its answer work and `chat_asked` is one reading: `chats.live` or the ended page holds a row.

## A project's repositories (`forge/listed.rs`)

```rust
/// A project's repositories in the switcher's small type (client/README.md, 5.3).
pub enum Held<'a> {
    Unknown,
    None,
    One { owner: &'a [u8], name: &'a [u8] },
    Several { owner: &'a [u8], name: &'a [u8], more: u32 },
}
pub fn held<'a>(forges: &'a State, playground: Option<&'a Playground>, project: u32) -> Held<'a>;
```

The first listed stands in for the home: rows by forge number, repositories in their row's order,
which is ascending by the forge's id for the repository, not by when it was adopted. `Home`'s `In`
and the Connected lines read the same one.

## Numbers, so that tasks side by side don't meet

| Task | `words.rs`, its numbers | `look.rs`, its tokens |
|---|---|---|
| `ready-adopts` | 3060 to 3069 | |
| `settings-page` | 3070 to 3079 | 230 to 234 |
| `rail` | 3080 to 3099 | 175 to 189 |
| `switcher-rows` | 3100 to 3109 | 190 to 199 |
| `home-page` | 3110 to 3169 | 200 to 229 |

## The shells

`w.shell` (`crate::words::Shell`) is the view's one way to tell them. Browser alone: the tab bar,
"Behind" outside the link's region, a switcher row's two spans. Terminal alone: "Behind" in the
region, a switcher row's name and one text.

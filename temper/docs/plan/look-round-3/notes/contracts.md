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

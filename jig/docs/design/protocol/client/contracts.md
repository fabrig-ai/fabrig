# The client's protocol: contracts while client layouts are built

The shapes the changes of the goal that freezes the client families'
versions share (`temper/docs/design/draft/design-economy.md`, 4.3), which no other
area reads. They bind as any design section does, and this document
goes once the goal's changes have landed. The lasting rules are
`versions.md`, section 1.

## 1. Each family's layout by name

- **Generated:** each family's generated module holds
  `pub const LAYOUT: [u8; 32]`, beside `VERSION` (skein's
  `data/codec/README.md`, section 4).
- **Re-exported** beside each crate's version constants, under these
  names, each the generated `LAYOUT` of its family:
  - `jig_client_protocol_wire::LAYOUT`, `jig-client-wire`'s, and
    `jig_client_protocol_wire::PUBLIC_LAYOUT`, `jig-public-wire`'s;
  - `temper_client_protocol_wire::CLIENT_LAYOUT`, `temper-client-wire`'s,
    and `temper_client_protocol_wire::PUBLIC_LAYOUT`,
    `temper-public-wire`'s;
  - the cursor's and the view records' `LAYOUT` in their codec crates'
    generated modules, named where their own crate reads them.

## 2. Owners declared with their layouts

- **To the client:** `jig_client_domain::Spoken { family: u16,
  version: u32, layout: [u8; 32] }`, `layout` the owner's client wire's
  `LAYOUT`. The link and the domain read it to judge the hello and to
  name a kept request's owner layout.
- **To the clients' component:** `jig_protocol_engine_clients::Served
  { family: Option<u16>, first: u32, last: u32, layout: [u8; 32] }`, and
  each settings path that carries it from the application (temper's
  `Clients::new`, jig's `config::clients::Application`): jig's entry
  with `jig_client_protocol_wire::LAYOUT`, an application owner's with
  its client wire's.

## 3. Later

None: the document goes with the goal.

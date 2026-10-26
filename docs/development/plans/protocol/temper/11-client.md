# Session 11: the client root and view, joined to the engine's root; the web crates retired

temper's first client is jig's client domain and view under a client
root and a view root of temper's, copied from jig's `ops` reference
roots with the forge's client side in place of ops's two connector
sides. The forge's side shows the forge's parts that need no document
in words, says "Shown on the forge." for the rest, and has no form yet
(15.1 adds it). `tests/client` joins the client to temper's engine root
from its first story, on jig's client world kit: a `Temper` app after
ops's `client/app.rs` and a `Joint` front after ops's `joint/`, over the
engine domain world's direct driver, with the translation between jig's
contract and the root's party vocabulary written once, exhaustive both
ways. The session ends with the stories that replace the web crates and
those four crates gone, in the same increment. Four increments: two of
step code proven by unit tests, two of world code proven in the joint
world.

## Read first

- `docs/design/client.md`, sections 1 to 5, 9 and 10; `docs/design/domain/testing.md`,
  6.1 and 6.3.
- jig at the pin (`~/.cargo/git/checkouts/jig-aba39083bf77fefe/d91a89c`):
  `docs/design/client/domain/root.md` (all: sections 3 to 10 are the
  shape copied), `docs/design/client/domain/view.md`, sections 4, 5, 7
  and 10.
- `findings/jig-client.md`, all (sections 1, 2, 5, 6 and Discrepancies
  6, 8 to 14, 16 bind this session); `findings/temper-misc.md`, sections
  1 and 2 (the retiring crates and their stories).
- `decisions.md`, 16, 18 and 20; `code-organization.md`, sections 2, 3,
  5 and 6.
- The code to copy, at the pin:
  - ops's client root, `examples/ops/domain-client/src/{lib.rs, owner.rs,
    address.rs, boundary.rs, domain.rs, route.rs, parts.rs, store.rs,
    limits.rs}` (810 lines; each file's "Keep: … Replace: …" comment
    says what to keep), and the shape of a side,
    `observability.rs:1-140`;
  - ops's view root, `examples/ops/view/src/{root.rs, limits.rs,
    words.rs:500-695, observability.rs:1-40, 216-259}`;
  - ops's client app, `tests/ops/src/client/app.rs` (393 lines), and its
    joint front, `tests/ops/src/joint/{front.rs:167-240, 502-640,
    800-1016, fixture.rs}`;
  - jig's client world kit, `tests/client/src/harness/{mod.rs:13-90,
    world.rs:13-70, faults.rs}` and `tests/client/src/joint/{transport.rs,
    packets.rs, requests.rs}` (public, reused, not copied);
  - jig's phrases, `crates/jig-view/src/phrases/english.rs:714-772,
    1157-1258` (the words the stories find).
- temper's code: `tests/engine/domain/src/{direct.rs, walking.rs:204-260,
  protocol.rs}` (the engine the front drives);
  `crates/temper-engine-domain/src/{boundary.rs:300-470, decision.rs:53-140}`
  (the party vocabulary); `tests/engine/domain/Cargo.toml` (a world's
  lints table, copied); `tests/web/domain/tests/{slice.rs, faults.rs}`
  (the stories that carry over).

## What the design asks

- **Roots copied from ops's** (client.md, 3): "`temper-client-domain` is
  copied from jig's reference client root ... with `ops`'s two connector
  sides replaced by the forge's one. It keeps jig's one shape ..., the
  request table ..., its routing ..., the small store composed ..., and
  never does what section 10 there forbids." The modules of 3.2, and
  "The reference's comments say what to keep and what to replace; the
  copy keeps them".
- **The forge's number given at construction** (3.1): "the shell passes
  `temper-protocol-client-wire`'s `FORGE` (4.6)"; parts by connector
  (3.3): "a part whose connector is the forge's goes to the forge's
  side; any other is a fact (`UnknownConnector`) ... The match is on the
  forge's number given at construction."
- **temper's navigation** (3.1): "home is the inbox; jig's entries are
  the inbox, the conversations across projects, and the deployment".
- **The request table's owner** (3.1): "`Jig(token)`, `Forge(token)`,
  and `Kept { scope }`" (15.1 adds `Forge`, with the form).
- **Parts not served** (4.2): every part but a connector, a resource or
  a kind "is not served by the engine yet. Its slot holds one paragraph,
  exactly: > Shown on the forge." "The slot never shows loading for
  these parts, and never an error."
- **Words that need no document** (4.1): the connector "forge"; a
  resource by its path's shape; a kind by the forge's numbering 1 to 15,
  "any other N | kind N". "Names ... are copied in, never translated"
  (5.2).
- **The view root** (5.1): `build` "when the client root's shown count
  moved": the page node, jig's frame and page, then "fills each slot in
  two passes ...: a part of the forge's from the forge's view, the
  navigation slot from the forge's view ... A slot no owner fills says,
  in the forge's words, that it cannot be shown." `act`: "resolved in
  the last tree to its binding, and decoded by its owner's bindings".
- **Joined from the start** (domain/testing.md, 6.3): "`tests/client` is that
  joint world itself: there is no client world of temper's on a scripted
  engine before it or beside it"; "The translation ... is written with
  the first types on both sides, exhaustive both ways"; "One thin story
  end to end, such as signing in and seeing the inbox".
- **What goes** (client.md, 9): "Retired in the plan that lands
  `temper-client-domain` and `tests/client`, in the same change:
  `crates/temper-web-domain`, `crates/temper-web-view`,
  `testing/temper-fake-person` and `tests/web/domain`. Their stories that
  still apply (signing in, a new chat answered, a held chat's escalation
  decided) are written again as stories of `tests/client`, on jig's
  pages."
- **Budgets** (domain/testing.md, 6.1): the worlds below the domain "together
  at most 4 s of the default suite and 25 s of the fuzzy suite";
  `tests/client`'s share is 1.0 s and 5 s (code-organization.md, 5).

## Choices this plan makes

- **11.1 lays the shape the form fills.** The root's enums have jig's
  variants only where the forge's side asks, answers or acts nothing
  yet: `Owner { Jig, Kept }`, `Ask::Jig`, `Answered::Jig`, `Action::Jig`,
  `Address::Jig`. 15.1 adds each `Forge` variant (and `Event::Forge`),
  and the compiler lists the arms to fill. The forge's side in 11.1 holds
  only the parts shown for the page's generation (`Show`, `PartsLeft`),
  as 4.5's state names them; it reads and watches nothing, so it takes no
  `Link`, `Confirmed` or `FrameIs` (15.1 adds what its form needs).
- **The forge's number** is `new(limits, seed, forge: u16)`'s third
  argument, kept by the root (`forge_number()`) and matched in
  `parts.rs` (findings/jig-client.md, Discrepancy 8: the references
  match a shared enum; the design asks for the number).
- **`navigation(&Limits)`** in `domain.rs` returns temper's navigation;
  the shell (12.1, the world's `App::new`) passes it to
  `configure_navigation` before the start. `Navigation.home` is not read
  by jig (Discrepancy 16): the shell starts at the inbox.
- **No sign-in provider.** temper registers none (client.md, 8);
  `provider` stays in `domain.rs` (client.md, 3.2 lists it) and nothing
  calls it. The joint front signs a client in when it binds it, as
  ops's `Joint::bind` does (`front.rs:800-819`), standing in for the
  printed link.
- **The view root's entry is `act`,** as client.md, 5.1 names it (ops
  names it `decode`, Discrepancy 9): it resolves the node and delegates
  to each owner's bindings' `decode` (jig's `jig_view::decode`, the
  forge's from 15.1), as ops's `decode` does.
- **Resources reach the forge's view two ways:** jig's `Part::Resource`
  in a part slot (`JigBindings::part`) and a project page's
  `OwnerPart { kind: Resource, key: Path }` (`JigBindings::owner_part`,
  Discrepancy 14). `forge::part` and `forge::owner_part` say the same
  words for the same path.
- **The navigation slot is spliced empty** in 11.2 (an empty fragment:
  the slot keeps no child), never left loading; 15.1 fills it.
- **The unshowable words** are "Cannot show this part." (the design asks
  for "the forge's words" without wording them; ops says "Cannot show
  this part").
- **The engine behind the front** is the engine domain world's
  `direct::Driver` (`tests/engine/domain/src/direct.rs:14`) on
  `walking::config` unchanged; a restart rebuilds the driver on its
  store and sends `Event::Restart`. The client root is given the
  engine's forge connector number, `walking::config`'s
  `core.connectors[0]` (0 there; 1, `FORGE`, in a deployment), as the
  world's `fixtures::FORGE`, so both roots name the connector alike
  (client.md, 3.1). 18.1 moves that world onto jig's
  real store underneath; the front does not change.
- **The person signs in as the deployment's first owner,**
  `walking::config`'s identity (provider 0, subject 7, login `person`,
  name `Person`), party 1.
- **11.4 adds one fuzzy sweep** to `tests/client` (the web world's
  `fuzzy_web.rs` retires in the same increment).

## Rules that bind it

- "**`temper-client-domain`** links `skein-lib` and `jig-domain-client`;
  **`temper-client-view`** adds `jig-view` and `temper-client-domain`.
  Neither links a wire, the link, smith or an engine crate (client.md,
  section 2)." (code-organization.md, 3) The check: `grep -E
  '^(temper|jig|smith)-' crates/temper-client-{domain,view}/Cargo.toml`
  prints only those names.
- "**Keeping the pins.** Adding to any `Cargo.toml` a crate of jig, smith
  or skein that `Cargo.lock` does not hold yet moves that repository's
  git source to its newest commit ... The increment that adds one then
  runs, in this order, and commits the lock" the three `cargo
  update -p … --precise …` commands (code-organization.md, 3).
  `jig-domain-client` (11.1), `jig-view` (11.2), `jig-client-world` and
  `jig-fake-person` (11.3) are not in temper's lock today: each of those
  increments runs them, and `grep -oE
  'forgetic/(jig|smith|skein)\.git#[0-9a-f]+' Cargo.lock | sort -u`
  prints exactly the three pinned revisions before it lands.
- "**The new worlds' share of the suites** ... `tests/client` 1.0 s and
  5 s ... A world past its share is made cheaper before more stories land
  in it; past twice it, a question." (code-organization.md, 5) and "No
  thread in a test" (5, 6).

## Increments

### 11.1 `temper-client-domain`: ops's root, the forge's side in place of ops's two

Scope: the crate `crates/temper-client-domain` (step code), copied from
ops's client root module by module (client.md, 3.2), the
infrastructure and observability sides replaced by `forge.rs`: the
forge's side holding the forge's parts shown for the page's generation
(client.md, 4.2, 4.5). The root routes jig's `ShowPart` by the forge's
number given at construction (3.3), `PartsLeft` to the side, `Link` and
`Confirmed` to the table, and everything else as ops's `route.rs` does;
temper's navigation (3.1).
Needs: nothing.
Files: `crates/temper-client-domain/{Cargo.toml, src/{lib.rs,
boundary.rs, owner.rs, address.rs, domain.rs, route.rs, parts.rs,
store.rs, limits.rs, forge.rs, tests.rs, tests/fixtures.rs}}`; the
workspace `Cargo.toml` (member; `[workspace.dependencies]`
`temper-client-domain`, `jig-domain-client`); `Cargo.lock` (the pins).
Shared hotspots: the workspace `Cargo.toml`, `Cargo.lock`. No existing
type changes.
Shape: as the interface sketch, module for module. Copied from ops:
`domain.rs:12-196` (`Domain`, `new`, `step`, `fire`, `take`,
`next_deadline`, `shown`, `answered`, `ask`, `ask_size` with `Ask::Jig`
sized by `limits.jig.document_bytes`, `provider`,
`configure_navigation`), `route.rs` (`event`, `went`, `route_jig`,
`progress`; `route_forge` in place of the two sides' routers; jig's
`FrameIs` answers no child in 11.1 and its arm pushes
`Fact::InvalidAnswer`), `parts.rs` (`show(root, env, part, generation,
scope, out)`: `connector(&part) == root.forge_number()` steps the side's
`Show { part, generation }`, any other pushes `Fact::UnknownConnector`;
`connector(part)` kept), `store.rs`, `limits.rs` (the forge's
`List<Held>` of `limits.forge.parts` in place of the two sides' sums).
Each file keeps ops's "Keep: … Replace: …" header, its "Replace:" line
naming the forge's side. `navigation(&Limits) -> Navigation`: capacity
`limits.jig.navigation_entries`, entries `Inbox { project: None }`,
`Conversations { project: None }`, `Deployment`, in that order.
Test shape: `src/tests.rs` after ops's `tests.rs`; fixtures in
`src/tests/fixtures.rs`: `limits()` (jig's numbers of
`tests/ops/src/client/app.rs:26-58`, `forge: forge::Limits { parts: 8 }`,
the table's of `app.rs:80-87`), `env(&Limits)`, `FORGE: u16 = 1`,
`OTHER: u16 = 2`, `PROJECT: u32 = 1`, `SEED: u64 = 11`, `connector()`
and `resource()` parts. Tests:
`a_part_of_the_forges_number_is_held_by_its_side` (rows: the root made
with forge 1 and with forge 9, each given a part of its own number),
`a_part_of_another_connector_is_reported_unknown`,
`parts_left_forget_the_parts_of_that_generation`,
`a_part_past_the_sides_bound_is_reported_parts_full`,
`kept_requests_are_restored_parked_and_counted_per_scope`,
`a_table_record_of_another_version_is_dropped_as_invalid`,
`temper_navigation_is_the_inbox_conversations_and_deployment`. Each
under 20 ms. Reference test, in full:

```rust
#[test]
fn a_part_of_another_connector_is_reported_unknown() {
    let limits = fixtures::limits();
    let env = fixtures::env(&limits);
    let mut root = crate::new(&limits, fixtures::SEED, fixtures::FORGE);
    let mut out = Queue::with_capacity(crate::max_out(&limits));
    let part = Part::Connector { connector: fixtures::OTHER };
    crate::parts::show(&mut root, &env, part.clone(), 1, PartScope::Project(fixtures::PROJECT), &mut out);
    assert_eq!(out.pop(), Some(Request::Fact { fact: Fact::UnknownConnector }));
    assert!(out.is_empty(), "an unknown connector's part asks for nothing more");
    assert!(!root.forge().contains(&part, 1), "the forge's side holds no other connector's part");
}
```

Proof: unit, `cargo nextest run -p temper-client-domain`, run
once: a part of the forge's number given at construction is held by the
forge's side, another connector's is reported `UnknownConnector`, and
kept requests come back parked per scope (the seven tests, under 1 s).
The joint layer needs 11.3's world.
Don't: link a wire, the link, smith or an engine crate; add a `Forge`
variant to `Owner`, `Ask`, `Answered`, `Action` or `Address` (15.1's);
match a child's vocabulary with `_`; keep a map from requests to
children (the table's owner does it, root.md, 10).
Size: at most 550 lines of production code.
Removes: nothing.

### 11.2 `temper-client-view`: ops's view root, the forge's words and fragments

Scope: the crate `crates/temper-client-view` (step code), copied from
ops's view root (`examples/ops/view/src/root.rs`, `limits.rs`), with
the forge's view in place of ops's two: the forge's phrasebook (client.md,
4.1, 4.2: the connector, a resource's shapes, the fifteen kinds and
"kind N", "Shown on the forge.", the unshowable words) and the parts'
fragments (5.2: one text node per connector, resource or kind; the
paragraph of 4.2 for any other part). `build` and `act` as ops's
`build` and `decode`,
jig's navigation slot spliced empty.
Needs: nothing beyond 11.1.
Files: `crates/temper-client-view/{Cargo.toml, src/{lib.rs, root.rs,
forge.rs, words.rs, limits.rs, tests.rs, tests/fixtures.rs}}`; the
workspace `Cargo.toml` (member; `temper-client-view`, `jig-view`);
`Cargo.lock` (the pins). Shared hotspots: the workspace `Cargo.toml`,
`Cargo.lock`.
Shape: as the interface sketch. `View::new(limits, domain, jig_words,
forge_words, forge)` asserts `forge_words.connector == forge` and a
finite worst case (ops `root.rs:29-63`). `build` (ops `root.rs:81-190`):
`render.stale(shown)` first; `jig.begin()`; `Role::Page`;
`jig_view::jig(builder, root.jig(), forge::nav(forge), words, jig)`,
which builds each page in 11.2 (no forge page before 15.1); slots in
two passes: `Owner::Jig` slots none (no forge page names a jig object
yet: unshowable); `Owner::Side { connector }` equal to the forge's
number: the navigation slot (`slot.reference == nav`) spliced with an
empty fragment, a part slot (`JigBindings::part`) from `forge::part`, an
owner part slot (`JigBindings::owner_part`) from `forge::owner_part`;
any other owner: `builder.unshowable(slot.id, Unshowable words)`. A
resource's words: `forge::resource(path)` reads the shape of client.md,
4.1's table (`forge` HOST OWNER NAME, then `branch` S1…Sn, `pull` N or
`issue` N; or two segments of 2 and 4 bytes, read big-endian; else
`Other`), and each shape's phrase copies the segments in ("OWNER/NAME"
one name hole of the two joined by `/`). `act` (ops's `decode`, `root.rs:203-217`):
`Owner::Jig` through `jig_view::decode`; `Owner::Side` of the forge
decodes to nothing in 11.2 (its fragments bind nothing); `Owner::Other`
nothing. `Words::english(connector, text)`: one language, holes up to 3
per phrase, phrase limits after ops's `words.rs:693`.
`Limits::of(domain, jig, forge)`: `jig_view::Limits::compose` with one
forge node per part slot (`domain.jig.parts`), text per node the larger
of jig's and `forge.capacity()`; `worst_case` sums jig-view's, jig's
bindings' and both phrasebooks' (ops `limits.rs:58-81`, one side).
Test shape: `src/tests.rs`; fixtures in `src/tests/fixtures.rs`:
`words()` (the forge's English at `text` 64), `jig_words()`,
`domain_limits()` (11.1's fixture values), `tree_limits()`,
`owner_part_text(&Words, &[&[u8]]) -> Box<[u8]>` (one fragment built
with `forge::owner_part` on a fresh `Builder`, finished, its text runs
joined), `part_text(&Words, Part, held: bool)`. Tests:
`each_resource_path_shape_says_its_words`,
`each_numbered_kind_says_its_words_and_others_kind_n` (rows 1 to 15, 0
and 16), `a_part_needing_a_document_says_shown_on_the_forge` (rows:
panel, result, effect, owner effect, outbox entry, hold, project,
deployment), `a_part_the_side_does_not_hold_says_it_cannot_be_shown`,
`the_navigation_slot_is_spliced_with_no_child`,
`a_slot_of_another_connector_says_it_cannot_be_shown`. Each under 20 ms.
Reference test, in full:

```rust
#[test]
fn each_resource_path_shape_says_its_words() {
    let words = fixtures::words();
    let rows: [(&[&[u8]], &[u8]); 6] = [
        (&[b"forge", b"forge.example", b"acme", b"shop"], b"acme/shop on forge.example"),
        (
            &[b"forge", b"forge.example", b"acme", b"shop", b"branch", b"temper", b"7"],
            b"branch temper/7 of acme/shop on forge.example",
        ),
        (&[b"forge", b"forge.example", b"acme", b"shop", b"pull", b"12"], b"pull request #12 of acme/shop on forge.example"),
        (&[b"forge", b"forge.example", b"acme", b"shop", b"issue", b"3"], b"issue #3 of acme/shop on forge.example"),
        (&[&[0, 1], &[0, 0, 0, 42]], b"repository 42 on forge 1"),
        (&[b"elsewhere", b"x"], b"elsewhere/x"),
    ];
    for (path, said) in rows {
        assert_eq!(&*fixtures::owner_part_text(&words, path), said, "path {path:?}");
    }
}
```

Proof: unit, `cargo nextest run -p temper-client-view`, run once:
each resource shape, each kind and each part needing a document is
said in client.md, 4.1 and 4.2's words, and a slot no owner fills says
it cannot be shown (the six tests, under 1 s). The joint layer needs
11.3.
Don't: bind a forge node (15.1's form and link are the first); draw a
style token of the forge's; translate a name (copy owners, names,
branches in as they are); add a page of the forge's.
Size: at most 500 lines of production code.
Removes: nothing.

### 11.3 `tests/client`: the joint world, and signing in to see the inbox

Scope: the world `tests/client` (package `temper-client-world`,
ordinary Rust), jig's client world kit (`jig-client-world`) for
temper's client: `Temper`, the `App` for temper's client root and view
(after `tests/ops/src/client/app.rs`), and `Joint`, the `Front` over
temper's engine root on the engine domain world's direct driver (after
`tests/ops/src/joint/front.rs`); `translate.rs`, the translation between
jig's contract (through the kit's `joint::{transport, packets, requests,
wire}`) and the root's party vocabulary, exhaustive both ways; the first
story: the deployment's owner signs in and sees the inbox (domain/testing.md,
6.3).
Needs: nothing beyond 11.2.
Files: `tests/client/{Cargo.toml, src/{lib.rs, app.rs, front.rs,
translate.rs, fixtures.rs}, tests/stories.rs}`; the workspace
`Cargo.toml` (member; `[workspace.dependencies]` `jig-client-world`,
`jig-fake-person`); `Cargo.lock` (the pins);
`crates/temper-engine-domain/src/{boundary.rs, route.rs}`
(`PartyInput::SignOut`, field for field with jig's
`people::Event::SignOut`, routed to the people child as the core takes
it, with one test of that routing; ruling 116). The crate's lints table is
`tests/engine/domain/Cargo.toml`'s; clippy reads `tests/clippy.toml`.
Shared hotspots: the workspace `Cargo.toml`, `Cargo.lock`. No existing
type changes.
Shape (ordinary Rust, not compiled):

```rust
// tests/client/src/app.rs, after tests/ops/src/client/app.rs
pub struct Temper;
/// The story limits: ops's jig numbers (app.rs:26-58), the forge's side's, the table's.
pub fn limits() -> temper_client_domain::Limits;
impl jig_client_world::harness::App for Temper {
    type Root = temper_client_domain::Domain;
    type View = temper_client_view::View;
    type Limits = temper_client_domain::Limits;
    type Event = temper_client_domain::Event;
    type Request = temper_client_domain::Request;
    type Record = temper_client_domain::store::Record;
    type Key = temper_client_domain::store::Key;
    type Address = temper_client_domain::Address;
    // new: `temper_client_domain::new(&limits, seed, fixtures::FORGE)`, then
    // `configure_navigation(&mut root, navigation(&limits))`; no provider.
    // act: `temper_client_view::act(view, node, act).map(|action| Event::Act { action })`.
    // duty, unsent, record_key, sent, received, seen, card: ops's, one side
    // fewer (app.rs:194-392), jig's parts through `observe::jig_*`.
}

// tests/client/src/front.rs, after tests/ops/src/joint/front.rs
pub struct Joint {
    /// temper's engine root on the engine domain world's driver and fake store.
    pub engine: temper_engine_domain_world::direct::Driver,
    /// Sign-ins the front made at the engine root.
    pub sign_ins: u32,
    transport: jig_client_world::joint::transport::Transport,
    limits: temper_client_domain::Limits,
    callbacks: Vec<(ClientId, temper_client_domain::Event)>,
    parked: BTreeMap<ClientId, Vec<temper_client_domain::Request>>,
    // offline_until, due, observed, lose_answers: ops's (front.rs:42-66).
}
impl Joint {
    /// The driver on `walking::config(seed)`, restarted; its forge connector is `fixtures::FORGE`.
    pub fn new(seed: u64, limits: temper_client_domain::Limits) -> Joint;
}
impl Front<Temper> for Joint {
    // bind: the transport binds the client to party 1 with the deployment
    //   owner's identity, then `translate::party(transport.start_sign_in(client))`.
    // wire: `translate::request` (ops front.rs:167-240's shape), parked while offline.
    // poll: the driver advanced and settled; each root output a client reads
    //   through `translate::incoming` (ops front.rs:502-640's shape).
    // fault: Restart (the driver rebuilt on its store, `Event::Restart`),
    //   Slow, NoAnswer, DropWatch as ops's front.rs:895-917; any other
    //   fault panics "not a temper joint story fault" until a story needs it.
    // reload, received, sign_in, sign_out: ops's.
}

// tests/client/src/translate.rs: the translation, written once.
/// The kit's core event, as the root's party input: SignIn -> SignedIn,
/// Ask -> Ask, Read -> Read, Watch -> Watch, WatchRun -> WatchRun,
/// Unwatch -> Unwatch, Delivered -> Delivered, StreamDelivered ->
/// StreamDelivered; each other `jig_domain_core::Event` variant named in
/// one arm that panics "the client kit does not make it" (no `_`).
pub fn party(event: jig_domain_core::Event) -> temper_engine_domain::Event;
/// One root output a client reads, as the kit's packet: PartyRequest's
/// SignInRefused, WatchRefused, RunText, and Deliver of WebReply, Read,
/// Document, WatchEnded; none for an output no client reads (Store,
/// Worker, Forge, Account, ToChild, Busy, Drained, Stop, Deliver of the
/// worker's and the store's deliveries), each named.
pub fn incoming(
    transport: &mut Transport,
    request: temper_engine_domain::Request,
    limits: &jig_domain_client::Limits,
) -> Option<jig_client_world::joint::packets::Incoming>;
```

Test shape: `tests/stories.rs`; fixtures in `src/fixtures.rs`:
`world(seed) -> World<Temper, Joint>` (ops `joint/fixture.rs:116-133`:
`Settings { seed, heap_bytes_per_client: 64 MiB, iterations: 4096, ..
}`, `randomness(2, 0, 0)`), `FORGE: u16 = 0` (`walking::config`'s forge
connector), `PARTY: u64 = 1`, `inbox()`, `WITHIN:
Duration = 1 s`, `SEED: u64 = 1101`. Test:
`signing_in_shows_the_inbox_from_the_engine_root`. Under 0.5 s.
Reference test, in full:

```rust
#[test]
fn signing_in_shows_the_inbox_from_the_engine_root() {
    let mut world = fixtures::world(fixtures::SEED);
    let person = world.client(fixtures::PARTY, fixtures::inbox());
    world.person(
        person,
        vec![
            Step::See { find: Find::named(Role::Status, b"Live"), within: fixtures::WITHIN },
            Step::See { find: Find::named(Role::Status, b"Signed in as Person"), within: fixtures::WITHIN },
            Step::See { find: Find::named(Role::Heading { level: 1 }, b"Inbox"), within: fixtures::WITHIN },
        ],
    );
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    assert_eq!(world.front().sign_ins, 1, "one sign-in reached the engine root");
}
```

Proof: joint, `cargo nextest run -p temper-client-world -E
'test(signing_in_shows_the_inbox_from_the_engine_root)'`, run once: the
person sees the link live, their name and jig's inbox, built by temper's
view root from temper's engine root's deliveries, with jig's referee
passing; under 0.5 s.
Don't: script an engine (domain/testing.md, 6.3); copy the kit's `joint`
modules (depend on them); route a client output around `translate.rs`;
start a thread.
Size: world code; production 0.
Removes: nothing.

### 11.4 The web stories on jig's pages; the web crates retired

Scope: the stories of `tests/web/domain` that still apply (findings/temper-misc.md,
2), written again in `tests/client` on jig's pages against temper's
engine root: a chat started and answered; a held chat's escalation
released; signed out mid-request, the request made once after signing in
again; a double press of "Start a conversation" made once; and one fuzzy
sweep of random people and faults. In the same increment, the four web
crates leave the workspace (client.md, 9).
Needs: nothing beyond 11.3.
Files: `tests/client/{src/front.rs, src/fixtures.rs, tests/stories.rs,
tests/fuzzy_client.rs}`; the workspace `Cargo.toml` (members at lines 6,
15, 17 and 27; `[workspace.dependencies]` at 37, 54 and 55); `Cargo.lock`
(cargo drops the four packages); deleted: `crates/temper-web-domain`,
`crates/temper-web-view`, `testing/temper-fake-person`,
`tests/web/domain` (8,425 lines). Nothing else names them outside
`docs/` (`git grep` at the base: only `Cargo.toml` and `Cargo.lock`).
Shared hotspots: the workspace `Cargo.toml`, `Cargo.lock`.
Shape: the front gains `FrontFault::SignedOut` (its next keyed request
answered `RequestEnvelope::SignedOut` and the client's sign-in dropped
from the transport, as a protocol layer whose bearer token ended; the
client signs in again by `Front::sign_in`) and `FrontFault::Busy`
(answered `Again` once); the chat's run is answered by the driver's
protocol stand (`Driver::answer`, `direct.rs:169-182`) once the
assignment is delivered; the held chat is made as
`tests/engine/domain/tests/root_routes.rs:3550`'s test makes one: its
run answered `tasks::End::Failed(tasks::Class::Run)` until it is held
past its tries. `src/fixtures.rs` gains `conversations()` (jig's
`Conversations { project: None }`, as jig's `stories.rs:286`) and
`chats(&Driver) -> u32` (the chat tasks in the driver's store rows).
Test shape: `tests/stories.rs` gains
`a_chat_started_on_jigs_page_is_answered_by_the_run`,
`a_held_chats_escalation_released_from_its_task_page`,
`signed_out_mid_request_is_made_once_after_signing_in`,
`a_double_press_of_start_makes_one_chat`; `tests/fuzzy_client.rs` (a
binary of its own): `random_people_and_faults_make_each_request_once`
(seeds 1 to 8; jig's random person, `randomness(12, 0, 0)` so the world
draws no fault the front does not play; `fixtures::faults(seed) ->
Vec<(Time, FrontFault)>` schedules at most 4 drawn from Restart,
NoAnswer, DropWatch, Busy and SignedOut with `World::fault_at`). The
steps use jig's words: "First words",
"Start a conversation" (`english.rs:714-715`), "Release", "Confirm",
"Released" (:1167, :1258, :848). Each focused story under 0.25 s, the
fuzzy test under 5 s. Reference story, in full:

```rust
#[test]
fn a_double_press_of_start_makes_one_chat() {
    let mut world = fixtures::world(fixtures::SEED);
    let person = world.client(fixtures::PARTY, fixtures::conversations());
    world.person(
        person,
        vec![
            Step::Type { find: Find::named(Role::TextArea, b"First words"), words: Box::from(*b"Hello") },
            Step::Press { find: Find::named(Role::Button, b"Start a conversation") },
            Step::Press { find: Find::named(Role::Button, b"Start a conversation") },
            Step::See { find: Find::text(b"Hello"), within: fixtures::WITHIN },
        ],
    );
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    assert_eq!(fixtures::chats(&world.front().engine), 1, "one chat task for the two presses");
}
```

Reference fuzzy test, in full (`tests/fuzzy_client.rs`):

```rust
#[test]
fn random_people_and_faults_make_each_request_once() {
    for seed in 1..=8 {
        let mut world = fixtures::world(seed);
        world.randomness(12, 0, 0);
        let person = world.client(fixtures::PARTY, fixtures::conversations());
        world.random_person(person);
        for (at, fault) in fixtures::faults(seed) {
            world.fault_at(at, fault);
        }
        let outcome = world.run(Until::Done);
        assert_eq!(outcome.verdict, Verdict::Passed, "seed {seed}");
        assert!(outcome.judged.0 > 0, "seed {seed}: the referee judged the run");
    }
}
```

Proof: joint, `cargo nextest run -p temper-client-world` and
`cargo nextest run -p temper-client-world --profile fuzzy`, each
run once: the four stories pass against temper's engine root, the sweep
makes each request once (under 1.0 s focused, 5 s fuzzy); and `git grep
-n -e temper-web- -e temper_web_ -e temper-fake-person -e
temper_fake_person -- ':!docs'` prints nothing.
Don't: edit `docs/` (the design's mentions stay; the web plan's files
are history); keep a story of the web world that is jig's to test
(drafts, Markdown, the referee's own tests, memory: findings/temper-misc.md,
2, last paragraph); merge two stories into one test.
Size: world code; production 0.
Removes: `crates/temper-web-domain`, `crates/temper-web-view`,
`testing/temper-fake-person`, `tests/web/domain` (8,425 lines).

## Interface sketches

`s11.rs` holds these, compiled under the workspace's lints. The client
root's vocabulary crosses into 11.2's view, 11.3's world, 12.1's native
client and 15.1's form; 15.1 adds the `Forge` variants named in the
comments. What ends what: a keyed request ends at its final answer or
its retention (the table); a part held by the forge's side ends when its
generation's parts leave; nothing else here has a lifetime.

```rust
//! Sketches of session 11: temper's client root (`temper-client-domain`,
//! 11.1) and view root (`temper-client-view`, 11.2), step code copied from
//! jig's `ops` reference roots (`examples/ops/domain-client`,
//! `examples/ops/view` at the pin), the forge's side in place of ops's two.
//! 15.1 adds the forge's form: the variants it adds are named in comments.
#![expect(unused_variables, reason = "a sketch's bodies are unreachable; its parameters are its signature")]

/// `crates/temper-client-domain`: the client root (client.md, section 3).
/// Owner: 11.1. Users: 11.2 (the view root), 11.3 (`tests/client`'s
/// `App`), 12.1 (the native client), 15.1 (the form), 16.x (the terminal).
pub mod temper_client_domain {
    /// `src/owner.rs`: the request table's owner (client.md, 3.1).
    pub mod owner {
        use skein_lib::Token;

        /// Who asked a keyed request: the table holds it, hands it back with
        /// the request's progress and final answer, and never reads it.
        /// 15.1 adds `Forge(Token)`, the forge's side's adoption.
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub enum Owner {
            /// jig's client domain asked it, under this token of its own.
            Jig(Token),
            /// Restored from the small store, its asker gone: its scope (the
            /// party it is for), parked until that party is confirmed.
            Kept { scope: u64 },
        }
    }

    /// `src/address.rs`: the root's address, the union of its children's.
    pub mod address {
        /// Where the client is: jig's pages only, until 15.1 adds
        /// `Forge(forge::Address)` (the adoption page, `Adopt { project }`).
        #[derive(Clone, PartialEq, Eq, Debug)]
        pub enum Address {
            /// One of jig's pages (the inbox, a task, a project, the deployment).
            Jig(jig_domain_client::address::Address),
        }

        /// A place the shell went to: an address, or one it could not read.
        #[derive(Clone, PartialEq, Eq, Debug)]
        pub enum Place {
            /// A known address; its owner opens the page, every other child
            /// is told it left its page.
            Address(Address),
            /// An address the shell could not decode: jig's page says so.
            Missing,
        }
    }

    /// `src/forge.rs`: the forge's client side (client.md, section 4), as
    /// 11.1 leaves it: it holds the parts shown in jig's slots for the
    /// page's generation, reads and watches nothing (4.2), and has no form.
    pub mod forge {
        use jig_domain_client::contract::Part;
        use skein_lib::{Env, Queue, Time};

        /// The side's bounds; 15.1 adds the form's (field bytes, checks).
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub struct Limits {
            /// Parts one page's generation may show in the forge's slots.
            pub parts: u32,
        }

        /// What the root hands the side.
        #[derive(Debug)]
        pub enum Event {
            /// jig's page shows `part`, one of the forge's, in a slot of
            /// generation `generation`: held until that generation's parts
            /// leave. From the root's routing of jig's `Request::ShowPart`.
            Show { part: Part, generation: u32 },
            /// The parts of `generation` and older left the page: forgotten.
            /// From jig's `Request::PartsLeft`.
            PartsLeft { generation: u32 },
        }

        /// What the side found wrong, as a fact.
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub enum Fact {
            /// A part past `Limits::parts`: not held, its slot says it cannot
            /// be shown. The root reports it as `Fact::PartsFull`.
            Full,
        }

        /// What the side asks of the root; 15.1 adds `Frame`, `Show`, `Send`.
        #[derive(Debug)]
        pub enum Request {
            /// A fact for the shell to observe; it ends nothing.
            Fact { fact: Fact },
        }

        /// The side's state: the parts held per generation and its shown
        /// count. Opaque: the view reads it through `contains`.
        #[derive(Debug)]
        pub struct Side {
            _held: (),
        }

        impl Side {
            #[must_use]
            pub fn new(limits: &Limits) -> Side {
                unreachable!("sketch")
            }
            /// Whether `part` is held for `generation`: the view fills its
            /// slot with the forge's words, or says it cannot be shown.
            #[must_use]
            pub fn contains(&self, part: &Part, generation: u32) -> bool {
                unreachable!("sketch")
            }
        }

        pub fn step(side: &mut Side, env: &Env<Limits>, event: Event, out: &mut Queue<Request>) {
            unreachable!("sketch")
        }
        /// The side keeps no deadline until 15.1's form.
        #[must_use]
        pub fn next_deadline(side: &Side) -> Option<Time> {
            unreachable!("sketch")
        }
        pub fn reclaim(side: &mut Side) {
            unreachable!("sketch")
        }
        /// Moves when what the side shows changed, so the view builds again.
        #[must_use]
        pub fn shown(side: &Side) -> u64 {
            unreachable!("sketch")
        }
        #[must_use]
        pub fn max_out(limits: &Limits) -> u32 {
            unreachable!("sketch")
        }
        #[must_use]
        pub fn worst_case(limits: &Limits) -> Option<u64> {
            unreachable!("sketch")
        }
    }

    /// `src/boundary.rs`: events in, requests out, one variant per child.
    pub mod boundary {
        use super::{address::Address, address::Place, store::Key, store::Record};
        use alloc::boxed::Box;
        use jig_domain_client::contract::{Ask as JigAsk, Read, Subject};
        use skein_lib::{List, RequestEnvelope, RequestKey, Token};

        /// One keyed request, tagged with the child whose meaning it is.
        /// 15.1 adds `Forge(forge::Ask)`, the adoption.
        #[derive(Clone, PartialEq, Eq, Debug)]
        pub enum Ask {
            /// jig's ask, as jig's client domain made it.
            Jig(Box<JigAsk>),
        }

        /// A final answer the protocol layer decoded, tagged with its owner.
        /// 15.1 adds `Forge(forge::Answer)`, the adoption's outcome.
        #[derive(Clone, PartialEq, Eq, Debug)]
        pub enum Answered {
            /// jig's done or refused, for one of jig's asks.
            Jig(jig_domain_client::contract::Answer),
        }

        /// A person's act, decoded by the view root, wrapped with its owner.
        /// 15.1 adds `Forge(forge::Act)`.
        #[derive(Clone, PartialEq, Eq, Debug)]
        pub enum Action {
            /// An act on one of jig's nodes.
            Jig(jig_domain_client::action::Act),
        }

        /// One event for the root, from the shell, the view or the
        /// protocol layer; each is routed to its child (root.md, section 5).
        #[derive(Debug)]
        pub enum Event {
            /// The client starts at `address` with the small store's records.
            Start { address: Address, kept: List<Record>, offset: jig_domain_client::boundary::Offset },
            /// The shell went to `place` (a link followed, an address typed).
            Went { place: Place },
            /// A person's act, from `temper_client_view::act`.
            Act { action: Action },
            /// The protocol layer's answer to the table's attempt: its
            /// envelope, read by the table, and the final answer, decoded.
            Answer { attempt: Token, envelope: RequestEnvelope, answer: Option<Answered> },
            /// One of jig's events (reads, watches, hello, kept requests).
            Jig(jig_domain_client::Event),
            /// The shell's sign-out ended, or the engine said the sign-in ended.
            SignedOut,
        }

        /// One of jig's unkeyed obligations, passed through the root.
        #[derive(Clone, PartialEq, Eq, Debug)]
        pub enum JigOut {
            /// Read `what` once; answered by `jig_domain_client::Event::Read`.
            Read { read: Token, what: Read },
            /// Open a watch; ended by `jig_domain_client::Event::Ended`.
            Open { watch: Token, subject: Subject },
            /// Widen or move an open watch's window.
            Continue { watch: Token, subject: Subject },
            /// Close a watch.
            Close { watch: Token },
            /// A shell-asked hand-off answered (a service's credential).
            Answered { asked: Token, answer: jig_domain_client::contract::Answer },
            /// A shell-asked hand-off whose answer is not known.
            Unknown { asked: Token },
        }

        /// What the routing or a child found, for the shell to observe.
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub enum Fact {
            /// jig's client domain's own fact.
            Jig(jig_domain_client::facts::Fact),
            /// The request table refused an ask past its bounds.
            TableFull,
            /// The forge's side held no more parts (`forge::Fact::Full`).
            PartsFull,
            /// A small-store record of another version or past a bound.
            InvalidRecord,
            /// A final answer of another owner's vocabulary, or none.
            InvalidAnswer,
            /// A part naming a connector that is not the forge's number.
            UnknownConnector,
            /// A kept request answered: its scope told it is gone.
            KeptAnswered,
        }

        /// One obligation for the shell; keyed sends come only from the table.
        #[derive(Clone, PartialEq, Eq, Debug)]
        pub enum Request {
            /// Write this record into the small store before any send of
            /// the same step.
            Save { record: Record },
            /// Erase this record from the small store.
            Erase { key: Key },
            /// Send the table's attempt with its key; ended by one
            /// `Event::Answer` for `attempt`.
            Send { attempt: Token, key: RequestKey, ask: Ask },
            /// jig's read, watch or hand-off answer, for the link.
            Jig(JigOut),
            /// Show `address` (the shell's place changes).
            Show { address: Address, replace: bool },
            /// Sign in through `provider`, then come back to `back`.
            SignIn { provider: u16, back: Address },
            /// Sign out: the shell's sign-out exchange; ends in `Event::SignedOut`.
            SignOut,
            /// A fact; it ends nothing.
            Fact { fact: Fact },
        }
    }

    /// `src/store.rs`: the small store's records, the children's wrapped.
    pub mod store {
        use skein_lib::{RequestKey, RequestRecord};

        /// The root's record version; another version is dropped at start.
        pub const VERSION: u16 = 1;

        /// A record of the small store: jig's own, or the table's. The
        /// forge's side keeps none (client.md, 4.5).
        #[derive(Clone, PartialEq, Eq, Debug)]
        pub enum Record {
            /// jig's record (a draft, where the client was).
            Jig(jig_domain_client::store::Record),
            /// A pending keyed request, at the root's `VERSION`.
            Table { version: u16, record: RequestRecord<super::boundary::Ask> },
        }

        /// A record's key, tagged with its owner.
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub enum Key {
            Jig(jig_domain_client::store::StoreKey),
            Table(RequestKey),
        }
    }

    /// `src/limits.rs`: the children's limits, the table's, and their sum.
    pub mod limits {
        use skein_lib::RequestLimits;

        /// The root's bounds: jig's, the forge side's, the table's.
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub struct Limits {
            pub jig: jig_domain_client::Limits,
            pub forge: super::forge::Limits,
            pub table: RequestLimits,
        }

        /// The most requests one step, `fire` or `take` may push.
        #[must_use]
        pub fn max_out(limits: &Limits) -> u32 {
            unreachable!("sketch")
        }

        /// The most heap the root holds; none when the limits contradict.
        #[must_use]
        pub fn worst_case(limits: &Limits) -> Option<u64> {
            unreachable!("sketch")
        }
    }

    /// `src/domain.rs`: the one shape (root.md, section 3).
    pub mod domain {
        use super::{boundary::Event, boundary::Request, limits::Limits};
        use jig_domain_client::pages::navigation::Navigation;
        use skein_lib::{Env, Queue, Time};

        /// The root: jig's client domain, the forge's side, the request
        /// table, and the forge's connector number given at construction.
        #[derive(Debug)]
        pub struct Domain {
            _children: (),
        }

        impl Domain {
            #[must_use]
            pub fn jig(&self) -> &jig_domain_client::Domain {
                unreachable!("sketch")
            }
            #[must_use]
            pub fn forge(&self) -> &super::forge::Side {
                unreachable!("sketch")
            }
            /// The forge's connector number the shell gave `new`.
            #[must_use]
            pub fn forge_number(&self) -> u16 {
                unreachable!("sketch")
            }
        }

        /// A root whose parts of connector `forge` go to the forge's side:
        /// the shell passes `temper_protocol_client_wire::FORGE` (client.md, 3.1).
        #[must_use]
        pub fn new(limits: &Limits, seed: u64, forge: u16) -> Domain {
            unreachable!("sketch")
        }
        pub fn step(root: &mut Domain, env: &Env<Limits>, event: Event, out: &mut Queue<Request>) {
            unreachable!("sketch")
        }
        pub fn fire(root: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
            unreachable!("sketch")
        }
        /// The table's records saved, then its sends, its progress routed;
        /// then every child and the table reclaimed.
        pub fn take(root: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
            unreachable!("sketch")
        }
        #[must_use]
        pub fn next_deadline(root: &Domain) -> Option<Time> {
            unreachable!("sketch")
        }
        /// jig's shown count plus the forge side's: the view builds when it moves.
        #[must_use]
        pub fn shown(root: &Domain) -> u64 {
            unreachable!("sketch")
        }
        /// Sets a sign-in provider's words before the start (none in standalone).
        pub fn provider(
            root: &mut Domain,
            provider: jig_domain_client::signin::Provider,
        ) -> Result<(), jig_domain_client::signin::Provider> {
            unreachable!("sketch")
        }
        /// Sets the navigation before the start; jig refuses one past its bound.
        pub fn configure_navigation(root: &mut Domain, navigation: Navigation) -> Result<(), Navigation> {
            unreachable!("sketch")
        }
        /// temper's navigation (client.md, 3.1): home the inbox; entries the
        /// inbox, the conversations across projects, the deployment.
        #[must_use]
        pub fn navigation(limits: &Limits) -> Navigation {
            unreachable!("sketch")
        }
    }

    pub use address::{Address, Place};
    pub use boundary::{Action, Answered, Ask, Event, Fact, JigOut, Request};
    pub use domain::{Domain, configure_navigation, fire, navigation, new, next_deadline, provider, shown, step, take};
    pub use limits::{Limits, max_out, worst_case};
}

/// `crates/temper-client-view`: the view root and the forge's view
/// (client.md, section 5). Owner: 11.2. Users: 11.3, 12.1, 15.1, 16.1.
pub mod temper_client_view {
    use super::temper_client_domain::{Action, Domain};
    use jig_view::{Act, NodeId, Patch, Tree};
    use skein_lib::Queue;

    /// `src/forge/words.rs`: the forge's phrasebook (client.md, 4.1, 4.2).
    pub mod words {
        use alloc::boxed::Box;
        use jig_view::{
            Builder,
            phrases::{Hole, Invalid, Phrasebook},
        };

        /// The forge's phrases (client.md, 4.1, 4.2), English as the design
        /// words them. 15.1 adds the form's, the outcome's and the
        /// navigation's.
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub enum Word {
            /// "forge".
            Connector,
            /// "OWNER/NAME on HOST": holes the full name, the host.
            Repository,
            /// "branch S1/…/Sn of OWNER/NAME on HOST": holes the branch, the full name, the host.
            Branch,
            /// "pull request #N of OWNER/NAME on HOST".
            Pull,
            /// "issue #N of OWNER/NAME on HOST".
            Issue,
            /// "repository ID on forge F": an adopted repository's two segments.
            Adopted,
            /// A path of no shape above: its segments joined by `/`.
            Path,
            /// Kind 1: "read".
            Read,
            /// Kind 2: "push".
            Push,
            /// Kind 3: "open a pull request".
            OpenPull,
            /// Kind 4: "merge a pull request".
            MergePull,
            /// Kind 5: "post a review".
            Review,
            /// Kind 6: "set a commit status".
            Status,
            /// Kind 7: "post a comment".
            Comment,
            /// Kind 8: "create an issue".
            CreateIssue,
            /// Kind 9: "create a branch".
            CreateBranch,
            /// Kind 10: "edit a pull request".
            EditPull,
            /// Kind 11: "update a pull request from its base".
            UpdatePull,
            /// Kind 12: "close or reopen a pull request".
            ClosePull,
            /// Kind 13: "edit, close or reopen an issue".
            EditIssue,
            /// Kind 14: "set reviewers".
            Reviewers,
            /// Kind 15: "delete a branch".
            DeleteBranch,
            /// "kind N", for a number the table does not word.
            OtherKind,
            /// "Shown on the forge." (client.md, 4.2), exactly.
            Shown,
            /// "Cannot show this part.": a slot no owner can fill.
            Unshowable,
        }

        /// A kind's word by the forge's numbering (`effect_shape`); none for
        /// a number the table does not word ("kind N").
        #[must_use]
        pub fn kind(kind: u16) -> Option<Word> {
            unreachable!("sketch")
        }

        /// The forge's phrasebook, loaded once at start, as data.
        #[derive(Clone, Debug)]
        pub struct Words {
            pub book: Phrasebook,
            pub language: u16,
            /// The forge's connector number the phrases are owned by.
            pub connector: u16,
        }

        impl Words {
            /// English, with names (owners, repositories, branches) at most
            /// `text` bytes each; the phrase checks are the phrasebook's.
            pub fn english(connector: u16, text: u32) -> Result<Words, Invalid> {
                unreachable!("sketch")
            }
            #[must_use]
            pub fn say(&self, word: Word, holes: &[Hole]) -> Box<[u8]> {
                unreachable!("sketch")
            }
            /// One plain text node of `word`.
            pub fn text(&self, builder: &mut Builder, word: Word, holes: &[Hole]) {
                unreachable!("sketch")
            }
            /// The longest saying of any phrase, for the view's limits.
            #[must_use]
            pub fn capacity(&self) -> Option<u32> {
                unreachable!("sketch")
            }
        }
    }

    /// `src/forge.rs`: the forge's view: its parts' fragments (5.2).
    pub mod forge {
        use super::words::Words;
        use alloc::boxed::Box;
        use jig_domain_client::contract::{OwnerPart, Part};
        use jig_view::{Builder, SlotRef};

        /// A resource's path as client.md, 4.1's table reads it; the
        /// segments are borrowed and copied into the words, never parsed
        /// but the adopted shape's two numbers.
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub enum Resource<'a> {
            /// `forge` HOST OWNER NAME.
            Repository { host: &'a [u8], owner: &'a [u8], name: &'a [u8] },
            /// … `branch` S1 … Sn: the branch's segments, at least one.
            Branch { host: &'a [u8], owner: &'a [u8], name: &'a [u8], branch: &'a [Box<[u8]>] },
            /// … `pull` N.
            Pull { host: &'a [u8], owner: &'a [u8], name: &'a [u8], number: &'a [u8] },
            /// … `issue` N.
            Issue { host: &'a [u8], owner: &'a [u8], name: &'a [u8], number: &'a [u8] },
            /// Two segments: a forge's number (2 bytes) and a repository's
            /// id (4 bytes), big-endian, as an adoption names it.
            Adopted { forge: u16, repository: u32 },
            /// Anything else: its segments joined by `/`, as they are.
            Other,
        }

        /// The slot reference of the frame's navigation slot, owned by the
        /// forge's side; spliced empty until 15.1 fills it with its link.
        #[must_use]
        pub fn nav(forge: u16) -> SlotRef {
            unreachable!("sketch")
        }

        /// Reads a resource's path into its shape (client.md, 4.1).
        #[must_use]
        pub fn resource(path: &[Box<[u8]>]) -> Resource<'_> {
            unreachable!("sketch")
        }

        /// A part's fragment in a slot: one text node of its words for a
        /// connector, a resource or a kind; the paragraph "Shown on the
        /// forge." for any other part; the unshowable status for a part
        /// the side does not hold for `generation`.
        pub fn part(
            builder: &mut Builder,
            side: &super::super::temper_client_domain::forge::Side,
            part: &Part,
            generation: u32,
            words: &Words,
        ) {
            unreachable!("sketch")
        }

        /// An owner part's fragment (a project page's resource, jig's
        /// `JigBindings::owner_part`): a resource path's words; "Shown on
        /// the forge." for every other kind.
        pub fn owner_part(builder: &mut Builder, part: &OwnerPart, words: &Words) {
            unreachable!("sketch")
        }
    }

    /// `src/limits.rs`: the view's bounds, jig-view's from the domain's
    /// plus the forge's fragments (client.md, 5.4).
    pub mod limits {
        use super::words::Words;

        /// The composed tree's bounds.
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub struct Limits {
            pub tree: jig_view::Limits,
        }

        impl Limits {
            /// jig-view's composition with one forge paragraph or text node
            /// per part slot; none past a bound.
            #[must_use]
            pub fn of(
                domain: &super::super::temper_client_domain::Limits,
                jig: &jig_view::words::Words,
                forge: &Words,
            ) -> Option<Limits> {
                unreachable!("sketch")
            }
        }

        /// The most heap the view holds: trees, bindings and phrasebooks.
        #[must_use]
        pub fn worst_case(
            limits: &Limits,
            domain: &super::super::temper_client_domain::Limits,
            jig: &jig_view::words::Words,
            forge: &Words,
        ) -> Option<u64> {
            unreachable!("sketch")
        }
    }

    /// `src/root.rs`: the view root's state (client.md, 5.1): jig-view's
    /// render state, jig's phrasebook and the forge's, jig's bindings for
    /// the last tree, the navigation slot's reference, the limits. 15.1
    /// adds the forge's bindings.
    #[derive(Debug)]
    pub struct View {
        _render: (),
    }

    impl View {
        /// Asserts that `forge_words` belong to connector `forge` and that
        /// the limits' worst case is finite.
        #[must_use]
        pub fn new(
            limits: &limits::Limits,
            domain: &super::temper_client_domain::Limits,
            jig_words: jig_view::words::Words,
            forge_words: words::Words,
            forge: u16,
        ) -> View {
            unreachable!("sketch")
        }
        /// The last tree built.
        #[must_use]
        pub fn tree(&self) -> &Tree {
            unreachable!("sketch")
        }
        #[must_use]
        pub fn limits(&self) -> &limits::Limits {
            unreachable!("sketch")
        }
    }

    /// When the root's shown count moved: the page node, jig's frame and
    /// page, jig's confirmation and notices; then the slots in two passes
    /// (the forge's parts, the navigation slot, unshowable otherwise);
    /// then jig-view's diff into `out`.
    pub fn build(view: &mut View, root: &Domain, out: &mut Queue<Patch>) {
        unreachable!("sketch")
    }

    /// A shell's act on `node` of the last tree (client.md, 5.1), resolved
    /// to its binding and handed to the owner's bindings' `decode` (jig's
    /// `jig_view::decode`) into the root's action; none for a node gone, a
    /// token of an older tree, or a node of the forge's (no binding in 11.2).
    #[must_use]
    pub fn act(view: &mut View, node: NodeId, act: Act) -> Option<Action> {
        unreachable!("sketch")
    }
}
```

## Questions to raise when reached

- **A joint story fault the engine root cannot play.** `FrontFault::NotReady`,
  `Versions` and `ObjectGone` have no engine-root counterpart in the
  direct driver. Default: they panic as ops's front does
  (`front.rs:910-916`) and no story draws them.
- **The root's `Delivery::Result`** (a requester's live notice of a
  task's end, `decision.rs:128-136`) has no counterpart in jig's client
  contract. Default: `translate::incoming` drops it, named in its arm;
  the task's document carries the result.
- **`Delivery::Application` before 15.2.** If 02.2 has landed, the root's
  delivery enum has the application's outcome, which no client of 11's
  asks for. Default: its arm in `translate::incoming` fails the story
  ("no client asked for an application") until 15.2 translates it.
- **A part slot of the forge's whose part jig never asked to show**
  (an owner part on a project page). Default: `forge::owner_part` says
  its words without the side's hold, since the owner part carries its
  path.

## Done when

- The crates and the world exist and pass: `cargo nextest run -p
  temper-client-domain -p temper-client-view -p temper-client-world`
  passes.
- The fuzzy sweep passes: `cargo nextest run -p temper-client-world
  --profile fuzzy` passes.
- The roots link no wire, link, smith or engine crate: `grep -hE
  '^(temper|jig|smith)-' crates/temper-client-domain/Cargo.toml
  crates/temper-client-view/Cargo.toml` prints only `jig-domain-client`,
  `jig-view` and `temper-client-domain` lines.
- The web crates are gone: `git grep -n -e temper-web- -e temper_web_ -e
  temper-fake-person -e temper_fake_person -- ':!docs'` prints nothing.
- The pins held: `grep -oE 'forgetic/(jig|smith|skein)\.git#[0-9a-f]+'
  Cargo.lock | sort -u` prints exactly the three pinned revisions.
- `tests/client` keeps its share: `cargo nextest run
  --workspace --profile measure -j 1 -E 'package(temper-client-world) &
  not binary(/^fuzzy_/)'`, then `awk -F'[][]' '/PASS \[/ {s += $2} END
  {printf "%.1f\n", s}' <its log>`, prints at most 1.0; the same with
  `--ignore-default-filter` added and `binary(/^fuzzy_/)` in place of
  `not binary(/^fuzzy_/)`, at most 5.0.
- The gate passed on this session's last landing: `git -C
  ~/src/rust/temper log -1 --grep='^Step: temper-11\.4$'
  --format='%(trailers:key=Gate-Result,valueonly)' origin/main` names
  `scripts/check.sh passed`.

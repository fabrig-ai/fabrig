+++
id = "public-web"
part = "temper-public"
kind = "feature"
needs = ["forge-public-wire", "public-wire", "public-http", "fetch-headers"]
files = ["temper/crates/temper-client-domain/src/address.rs", "temper/crates/temper-client-domain/src/public.rs", "temper/crates/temper-client-domain/src/public/**", "temper/crates/temper-client-domain/src/lib.rs", "temper/crates/temper-client-domain/src/route.rs", "temper/crates/temper-client-domain/src/authorize.rs", "temper/crates/temper-client-domain/src/tests.rs", "temper/crates/temper-client-domain/src/tests/address.rs", "temper/crates/temper-client-domain/src/tests/public.rs", "temper/crates/temper-client-domain/src/tests/public/**", "temper/crates/temper-client-view/src/public.rs", "temper/crates/temper-client-view/src/public/**", "temper/crates/temper-client-view/src/words.rs", "temper/crates/temper-client-view/src/sign_in.rs", "temper/crates/temper-client-view/src/lib.rs", "temper/crates/temper-client-view/src/look.rs", "temper/crates/temper-client-view/src/tests.rs", "temper/crates/temper-client-view/src/tests/public.rs", "temper/crates/temper-client-view/src/tests/public/**", "temper/crates/temper-client-shell-web/**"]
effort = "high"
minutes = 180
+++

# The web client's public mode: addresses, pages, and reading without signing in

## Goal

At an address under `/public`, the web client reads a public project without signing in:
temper's address codec knows the public pages by public identifier, a public mode reads each
page's documents by jig's public `GET` with no credential, no stream and no store, reads
again by each answer's freshness while the tab is visible, shows nothing past its freshness,
and the view draws the public records alone; the signed-out page links to `/public`
(`temper/docs/design/client/browser.md`, section 8; jig's `protocol/client.md`, 17.5).

Acceptance:

- **Addresses** (`temper-client-domain/src/address.rs`, browser.md 8.1): `Place` gains
  `Public(Public)`, appended last, with

  ```rust
  /// A public page, its identifiers public ones (jig's domain/people.md, 13.8).
  pub enum Public { Projects, Board { project: u64 }, Task { project: u64, task: u64 },
                    Landings { project: u64 }, Landing { project: u64, task: u64 } }
  pub fn print_public(page: &Public) -> Box<[u8]>
  ```
  `parse` gives `Place::Public` for `/public` and every path below it: the five paths of
  8.1, each identifier exactly 16 lowercase hexadecimal digits; any other path under
  `/public` (a number, upper case, short or long identifiers, an extra or empty segment, a
  trailing slash, a query, a percent-encoded byte) is `Public::Projects`, so the shell
  replaces it with `/public` as `opened` replaces any path that prints otherwise. `/publicx`
  and other paths are read as before. `Address` is unchanged: no signed-in page names a
  public one. The other `Place` matches (`route.rs`, `authorize.rs`, the shell's `host.rs`)
  gain the arm: a public place reaching the signed-in root is `Missing`'s.
- **The public mode** (`temper-client-domain`'s new module `public`, `src/public.rs` and
  `src/public/`), a pure state machine with no `jig-client`:

  ```rust
  pub struct Reader { .. }          // one page's documents and their timers
  pub enum Doc { JigProjects, JigBoard, JigCard, ForgeChange, ForgeLandings, ForgeLanding }
  pub enum Heard { Document { body: Box<[u8]>, fresh: Duration }, Unknown { fresh: Duration },
                   Busy { retry: Duration }, NotServed, Failed }
  pub enum PublicEvent { Open { page: Public }, Heard { doc: Doc, heard: Heard },
                         Visible { visible: bool }, Due }
  pub struct PublicRead { pub doc: Doc, pub owner: u16, pub version: u16, pub kind: u16,
                          pub ids: [Option<u64>; 2] }
  impl Reader { pub fn new(limits: &PublicLimits) -> Self;
                pub fn step(&mut self, now: Time, event: PublicEvent, out: &mut Queue<PublicRead>);
                pub fn deadline(&self) -> Option<Time>;
                pub fn page(&self) -> &Public; pub fn shown(&self) -> Shown<'_> }
  ```
  (`Shown` borrows each document's decoded record, or its state: reading, unavailable,
  unknown). A page reads its documents: `Projects` jig's *projects*; `Board` jig's *board*
  and the forge's *landings*; `Task` jig's *card* and the forge's *change*; `Landings` the
  forge's *landings*; `Landing` the forge's *landing* (jig's owner 0 and its
  `public_v1::PUBLIC_*` kinds and version; the forge's owner
  `temper_client_protocol_wire::number(Owner::Forge) + 1` and its `public_v1` kinds and
  version). Records decode with `jig_client_protocol_wire::public_v1` and
  `temper_client_protocol_wire::public_v1` at their limits; a record that does not decode is
  `Failed`'s.
  - **Freshness:** a document shown is fresh until `now + fresh`, `fresh` being the answer's
    `max-age` less its `Age` (public-http's client end gives it); while the tab is visible, it
    is read again then (a freshness of zero, one second later); a hidden tab reads nothing and,
    shown again, reads at once every document of its page whose freshness has ended.
  - **Nothing shown past its freshness:** a read answered `Busy`, `Failed` or `NotServed`
    keeps the document, marked as of when it was read, until its freshness ends, then drops it
    and the page says "Not available just now."; it reads again after `Retry-After`
    (`Busy`'s `retry`, one second for the others), doubling at each refusal in a row to a
    bound of 60 seconds, the doubling reset by a document; drops happen whether or not the
    tab is visible.
  - **Unknown:** the page's first document (jig's for `Projects`, `Board` and `Task`, the
    forge's for `Landings` and `Landing`) unknown drops everything the page showed and shows
    "Not public, or not here." with a link to `/public`; a second document unknown (a task
    that makes no change, a project with no landings) shows nothing of it.
- **The shell** (`temper-client-shell-web`): `lib.rs`'s `start` checks the path first: a
  `Place::Public` starts the public host (`src/public.rs`, `PublicHost`, with `new`,
  `start(path)`, `report(Report) -> Duties`, `deadline()`, as `Host` has) in place of
  `Host::start`: no `jig_client`, no `Transport`, no credential, no stream, no store read or
  written. It carries each `PublicRead` as an `Asked::Exchange` whose call is public-http's
  `GET` (no `Authorization`, no body), with `cache: Cache::NoStore` and
  `keep: jig_client_protocol_http::PUBLIC_KEEP` (`notes/contracts.md`, section 6), so the
  browser's own cache never answers it; it maps `Heard::Answered { status, headers, body }`
  through public-http's client end (the status and the freshness it reads from the kept
  `cache-control` and `age`, and `retry-after`) to `Heard`, and `Heard::Failed` to
  `Heard::Failed`; it sets the timer from `deadline()`, and reports the document's visibility
  (`visibilitychange`, `document.hidden`) as `PublicEvent::Visible`. A link between public
  pages navigates in place, as the signed-in pages' do.
- **The view** (`temper-client-view`'s new module `public`, browser.md 8.3):
  `pub fn build_public(view: &mut PublicView, reader: &Reader, wall: Wall, patches: &mut
  Queue<Patch>)` and its `act`/`href` as the signed-in view's; it reads the public records
  alone. A project "Project 3f2a9c01", a goal "Goal 3f2a9c01", a task "task 3f2a9c01" (the
  identifier's first eight digits); an executor its kind ("an agent", "a procedure", "a
  person", "a job"); the board's goals by line (phase in words, for how long, spend), their
  trees' lines, the folded count and the queues' lines from *landings*; a task's line,
  lineage as links, delegates and, with a change, its step in words, CI's state, its gates
  by number and kind with their verdicts, "its change on the forge" linking the pull's
  address and the head's short hash; the landings' turn, queues in order without places,
  recent landings with their landed commits linked. The banner offers "Sign in" (a link to
  `/`) when the page's `Ways` have a way, and nothing else of a person's. New words go in
  `words.rs` as its others are (each a `Word` with its own number).
- **The signed-out page** (`sign_in.rs`) gains a link "Read public projects" to `/public`
  (a full navigation, `external`, so the shell starts the public mode).
- **Step tests** (browser.md 8.4):
  - `temper-client-domain` `src/tests/address.rs`: `public_paths_print_and_parse_both_ways`
    (each page, identifiers with leading zeros); `a_malformed_public_path_reads_as_public`
    (a decimal number, upper case, 15 and 17 digits, an extra segment, a trailing slash, a
    query, `%61`); `other_paths_read_as_before`.
  - `temper-client-domain` `src/tests/public.rs`: `each_page_reads_its_documents` (owner,
    version, kind, ids); `it_reads_again_when_freshness_ends` (a freshness of 5 s after an
    `Age` of 3 s from a `max-age` of 8 reads at 5 s, not 8);
    `a_hidden_tab_reads_nothing_and_reads_at_once_when_shown`;
    `busy_keeps_the_page_until_its_freshness_ends_then_drops_it`;
    `refusals_back_off_from_retry_after_doubling_to_a_minute`;
    `unknown_drops_the_page_and_shows_its_main`; `a_second_documents_unknown_shows_nothing_of_it`;
    `drops_happen_while_hidden`.
  - `temper-client-view` `src/tests/public.rs`: `the_view_shows_only_what_the_records_carry`
    (the page's text, from records built in the test, is made of the view's words, the
    records' numbers and times and the identifiers' first eight digits alone; no other
    byte sequence of length 8 or more);
    `unknown_and_unavailable_say_their_words`; `the_signed_out_page_links_to_public`.
  - `temper-client-shell-web` `src/host_tests_public.rs`:
    `the_public_mode_sends_no_credential_and_opens_no_stream` (every `Asked` an `Exchange`
    with method `GET`, a path under `/jig/v1/public/`, no `Authorization`, no body,
    `Cache::NoStore` and `PUBLIC_KEEP`; no `OpenStream`, `Save` or `Erase`);
    `the_freshness_comes_from_the_kept_headers` (an answer with `cache-control:
    public, max-age=8, must-revalidate` and `age: 3` sets the reader's next read 5 s on);
    `the_timer_follows_the_reader`.

## Inputs

- `temper/docs/plan/public-reading-build/notes/common.md`; `notes/contracts.md`, sections 3
  and 5.
- `temper/docs/design/client/browser.md`, section 8 (8.1-8.4); 2.3 (the address codec);
  jig's `jig/docs/design/protocol/client.md`, 17.5 (the `GET`, its answers and headers, the
  client end); `jig/docs/design/client/domain/contract.md`, section 12 (jig's records);
  `temper/docs/design/client/forge.md`, 11.1 and 11.3 (the forge's records).
- `tasks/public-wire.md`, `tasks/forge-public-wire.md`, `tasks/fetch-headers.md`
  (`notes/contracts.md`, section 6; `skein/docs/design/ui-web.md`, section 10, "An exchange's
  cache and an answer's headers") and the merged `public-http` (its client end in
  `jig-client-protocol-http`: the public `GET` call, `PUBLIC_KEEP`, and the answer's reading
  with its freshness).
- As they stand: `temper-client-domain`'s `address.rs` (`Place`, `print`, `parse`, `segment`)
  and `src/tests/address.rs`; `temper-client-view`'s `sign_in.rs` (`front`), `words.rs`,
  `pages/change.rs` (how a page draws a change's step, CI and gates), `root.rs` (`build`,
  `act`, `href`); `temper-client-shell-web`'s `lib.rs` (`start`, `Shell::turn`, the timer),
  `host.rs` (`Host::start`, `opened`, `carry`, `deadline`), `host_tests.rs`.

## Owns and may change

- Owns the files in the header; in `lib.rs`, `tests.rs`, `route.rs` and `authorize.rs` only a
  module line or the `Place::Public` arm. `public-fuzzy-temper` adds fuzzy tests elsewhere
  and reads `address::parse`, `print_public` and the records' decoders: keep their names.
- **Reader files,** each edited only for `Place::Public`'s arm or a `Word`'s, as the compiler
  names them: `temper/tools/gallery/src/**`, `temper/tests/client/src/**`,
  `temper/tests/one-process/src/**` (a forced arm only), `temper-client-shell-cli`.
- `fetch-headers` gave `Asked::Exchange` its `cache` and `keep` and `Heard::Answered` its
  `headers`, every existing exchange passing `Cache::Browser` and `&[]`: keep those as they
  are; only the public host's exchanges send `Cache::NoStore` and `PUBLIC_KEEP`.
- Pre-approved: if public-http's client end names its call or answer otherwise than the
  shapes above, map them to `Heard` in the shell and keep `Reader`'s shapes; if a view's
  phase, time or duration words exist for the signed-in pages, reuse them; if the shell
  cannot run `PublicHost` beside `Host` in `Shell`, hold an enum of the two and say so.

## Steps

1. The codec's public paths and their tests
   (`heavy cargo nextest run -p temper-client-domain --lib -E 'test(/address/)'`).
2. The reader and its tests (`-E 'test(/public/)'`).
3. The view's public pages, the words, the signed-out link; their tests
   (`heavy cargo nextest run -p temper-client-view --lib -E 'test(/public|sign_in/)'`).
4. The shell's public host and its tests
   (`heavy cargo nextest run -p temper-client-shell-web --lib -E 'test(/public/)'`), and the
   web build (`heavy cargo check --target wasm32-unknown-unknown -p temper-client-shell-web`,
   the target `temper/scripts/bundle.sh` builds).
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-client-domain -p temper-client-view -p temper-client-shell-web --all-targets -- -D
   warnings`; the focused tests of those packages and of `temper-client-world`,
   `temper-client-gallery` and
   `temper-client-shell-cli`; the citations check.
6. Commit (why: a visitor reads a public project in the browser without signing in, and sees
   nothing past its freshness); push your branch.

## Report

As the implementer's text says, and also: where hidden-tab handling and Retry-After live,
the words added with their numbers, and how the shell reports visibility.

## Answers

2026-11-03. **The user's decision on public titles.** User, 2026-11-03 15:40Z: public reading shows titles. A goal's and a task's title (the first line of its spec, as jig's client contract names a task's title) crosses into its public line, and a public project's name crosses into its public record. There is no author flag and no human publication capability for these. Everything else in people.md 13.3's "No text at all" list still never crosses: spec beyond its first line, instructions, inputs, results, words, questions and answers, transcripts, notes, reasons, labels' meanings, proposals, escalations, people's and parties' names, charters', procedures' and roles' names. Review finding 1 is settled by the user's decision.

For this task: Show each goal's and task's title on the board and the goal tree, and the project's name as the board's heading.

2026-11-03. **A public project's name in the wire** (answered for `public-wire`): jig's
*projects* answer is a list of `PublicProject { project, name }`, and `PublicBoard` carries
`name` after `project`, each the project's name within the client wire's project-name bound
(64 bytes).

+++
id = "kept-metas"
part = "forge-sign-in"
kind = "feature"
needs = []
files = ["temper/crates/temper-service/src/clients/**", "temper/crates/temper-service/src/clients.rs", "temper/crates/temper-service/src/forges.rs", "temper/crates/temper-service/src/forge/gone.rs", "temper/crates/temper-service/src/forge/records.rs", "temper/crates/temper-service/src/tests/forge_sign_in.rs", "temper/crates/temper-service/src/tests/sign_in_configuration.rs", "temper/docs/design/protocol/sign-in.status.md"]
effort = "high"
minutes = 170
+++

# temper serve takes `application.sign_in.forges`, and the page's metas come from the kept forges

## Goal

A document with `application.sign_in.forges` and a bundle refuses its start today, and the refusal blames `</head>` and `clients.bundle_bytes`, neither of them the cause. The cause, to confirm first: `crates/temper/src/lib.rs` calls `fill_forge_sign_in(named.sign_in.as_ref(), &[], ...)`, so `forge_metas` in `temper-service/src/clients/bundle.rs` finds no forge and returns `None`.

Build what the design says: an entry refuses no start, its serving is decided once the forge's records load and when a forge is made or removed, and the page's metas are made from it.

Acceptance:

- With a bundle, Google's sign-in, a provider and `"forges": [{"forge": 1, "client": "forgejo"}]`, and no forge kept, `temper::prepare_in` starts. This test comes first and is seen red with today's message.
- A page with no `</head>` refuses the start with exactly `application.clients.bundle: NAME: the page has no </head> for its sign-in metadata`, `NAME` as today's message has it. Nothing says `clients.bundle_bytes` but what is past it.
- Each case of the design's 9.7 and 9.8, in the tests below, holds.
- `temper-service`'s tests pass; `heavy cargo nextest run -p temper --test suite -E 'test(~bundle)'` and `-p temper --lib -E 'test(~note)'` pass (a change of what the page holds runs its readers' tests).

## Inputs

- `temper/docs/design/protocol/sign-in.md`, 7, 9.1, 9.3, 9.7 and 9.8, the sections this cut wrote or changed.
- `temper/docs/design/protocol/forge.md`, 2.1 (`web`).
- `temper/docs/plan/help-server/inputs/request.md`, finding 1.
- `temper/docs/plan/help-server/notes/common.md`.

## Owns and may change

Owns: `temper-service/src/clients.rs`, `clients/bundle.rs`, a new `clients/forge_sign_in.rs`, and a new `tests/forge_sign_in.rs`.

May also change, and how:

- `temper/crates/temper/src/lib.rs`: the fill in `prepare_with_limits` alone.
- `temper/crates/temper/src/host.rs`: `note_line`'s new arm, and its test.
- `temper/crates/temper/tests/suite/startup.rs`: the two start tests below.
- `temper-service/src/lib.rs`: the `Note` variant. `temper-service/src/system.rs`: one call beside `clients.configure_google`. `temper-service/src/tests.rs`: the `mod` line.
- `temper-service/src/forges.rs`, `forge/gone.rs`, `forge/records.rs`: one call each, to refresh.
- `temper-service/src/startup/application.rs`: `forge_origin` takes `web`.
- `temper-service/src/tests/sign_in_configuration.rs`: `forge_metas_escape_attributes_publish_modes_and_change_the_etag`, rewritten onto the new functions, its bytes kept.
- `temper/docs/design/protocol/sign-in.status.md` (a departure from your text, which leaves `docs/design/` alone): remove the **Not yet** entry for 7, 9.1, 9.2 and 9.7, keeping 2.4's and 2.5's.

Options, each pre-approved:

- A page reply on its way must end with the page it began. Default: hold the new page until no page reply runs, new requests answered from the old one meanwhile. If giving each page reply its own copy is simpler in the code you find, do that and say so.
- If `temper-service/src/family.rs`, `making`, drops a Forgejo's `web`, keep it there and say so; from reading, it carries the whole place.
- A key for the start test: `support::forge_sign_in::command(directory)` makes one with `openssl`, as `suite/forge.rs` does. If a kept fixture key is cheaper and the start takes it, use that.

## Steps

1. The failing test, in `temper/crates/temper/tests/suite/startup.rs`, beside `a_directory_that_is_no_bundle_refuses_the_start`: `bundle_configuration`, a bundle whose page has `</head>` (the file's `PAGE` has none: write `index.html` over it), `support::forge_sign_in::configure`, `support::forge_sign_in::command`, then `temper::prepare_in(...)` is `Ok`. See it refused, with the words above. Then its neighbour, `fixture_bundle` as it is, `expect_err` with the new words.

2. In `lib.rs`, the start calls `fill_sign_in` alone. Its refusal is the new words.

3. The decision, pure, in `clients/forge_sign_in.rs` (protocol/sign-in.md, 9.3 and 9.7):

   ```rust
   /// The entries of `sign_in.forges`, joined to their clients' `redirect_uri`, and what each was last named for.
   pub(crate) struct Entries { /* rows, the owned part's public, the bundle's limit, named: Map<u16, Unserved> */ }

   /// Why an entry serves no meta.
   #[derive(Clone, Debug, PartialEq, Eq)]
   pub enum Unserved { NotKept, NotForgejo, Origin { origin: Box<[u8]> }, NoRoom }

   /// Each entry's verdict and the metas' bytes, entries in the document's order.
   fn decide(entries: &Entries, forges: &crate::forges::Forges, page: usize, files: usize) -> (Box<[u8]>, List<(u16, Box<[u8]>, Unserved)>);

   /// Decide again, set the page's metas, and name each entry newly not serving (`Note::ForgeSignIn`).
   pub(crate) fn refresh(engine: &mut crate::engine::Engine);
   ```

   `refresh` returns at once until `engine.forge_records.done()`, so no forge is called absent before the records load. A forge's origin is `forge_origin(&forge.git, forge.web.as_deref(), public)`, `public` the machine's `owned.public` when `forge.mode` is `Mode::Owned` and none otherwise; the meta's mode word is `owned` for `Mode::Owned`, `other` otherwise. `checked.setup` is always empty at start, so `Bundle` no longer reads a `Setup`; the meta's bytes and escapes stay as the rewritten test pins them.

4. `Bundle` gains the page's change:

   ```rust
   impl Bundle {
       /// Set the page's forge metas, in place of any before them, and its entity tag (protocol/sign-in.md, 9.7).
       pub(crate) fn set_forge_metas(&mut self, metas: &[u8]) -> Option<()>;
   }
   ```

   `fill_forge_sign_in` and `forge_metas` go. Room is `decide`'s: an entry whose meta would take the page and files past `clients.bundle_bytes` is `NoRoom`, and later entries that fit still serve.

5. `Clients` holds `Entries`, given at start beside `configure_google` (`system.rs`): `sign_in.forges`, `settings.jig.clients.provider`'s clients, `settings.machine.owned`'s `public`, `settings.limits.clients.bundle_bytes`. `refresh` runs where the records' phase becomes done (`forge/records.rs`), at the end of a kept forge's handling in `forges.rs`, where it did not return early, and after `forge/gone.rs` removes one.

6. The note:

   ```rust
   ForgeSignIn { number: u16, client: Box<[u8]>, why: clients::forge_sign_in::Unserved },
   ```

   `note_line` says `temper serve: forge N: no sign-in through temper: WHY`, `WHY` one of: `no such forge is kept`; `it is not a Forgejo`; `its browser origin ORIGIN is not client CLIENT's redirect_uri's`; `no room for its meta within clients.bundle_bytes`. Named once for a reason; a forge that begins to serve and later stops is named again.

7. Tests, in `temper-service/src/tests/forge_sign_in.rs`, on `sign_in::world_settings` and `fixtures::keep_forge`, a bundle adopted, each answered by `GET /`:

   - `a_kept_forgejo_at_its_redirect_gets_its_meta_and_the_page_a_new_etag`: a managed Forgejo, `git` `http://127.0.0.1:3000`, `web` `https://git.example.com`, the client's `redirect_uri` `https://git.example.com/user/oauth2/temper/callback`. No meta before; after, `content="1 forgejo https://git.example.com other"`, another `ETag`, and no note beyond the one before it was kept. Revalidating with the old tag answers 200.
   - `an_owned_forge_takes_the_machines_public_and_says_owned`.
   - `an_entry_that_does_not_serve_is_named_once_and_the_others_serve`: a table of no forge kept, a GitHub forge, another origin (`git` alone, `web` none), past the room, each beside a second entry that serves; two refreshes, one note.
   - `a_removed_forge_loses_its_meta`.
   - `a_restart_makes_the_meta_from_the_restored_record`.
   - `a_page_reply_on_its_way_ends_with_the_page_it_began`: a page large enough to take two chunks, a forge kept between them, the body whole against its `Content-Length`.

8. `cargo fmt`, clippy, the acceptance's tests, push.

## Report

As the implementer's text says, and the rule each new test was broken by. Your report says whether a process-tier story's stderr comparison met the new note: that story is `walk-story`'s, and goes under `## Left`.

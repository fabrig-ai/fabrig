+++
id = "web-tokens"
part = "web"
kind = "feature"
needs = ["pasted-list", "wire-connected"]
files = [
  "temper/crates/temper-client-domain/src/forge.rs",
  "temper/crates/temper-client-domain/src/forge/**",
  "temper/crates/temper-client-domain/src/tests/**",
  "temper/crates/temper-client-view/src/forge/**",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/connect_forge.rs",
  "temper/crates/temper-client-view/src/tests/forges.rs",
  "temper/crates/temper-client-view/src/tests/accounts.rs",
  "temper/tests/client/src/front.rs",
  "temper/tests/client/tests/suite/forges_edges.rs",
  "temper/tests/client/tests/suite/forgejo_tokens.rs",
  "temper/tests/client/tests/suite/main.rs",
  "temper/docs/design/client/forge.status.md",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 150
+++

# The web page connects a Forgejo with its bot's tokens

## Goal

`ConnectForge`'s Your Forgejo draws its addresses and the admin's form alone, and a kept
connected Forgejo's Details draw an owned forge's set up. Bring them to
`temper/docs/design/client/forge.md`, 5.4: the tokens form beside the admin's for a forge to
make, the tokens form alone for a kept connected Forgejo not connected or stopped, its
Connecting, its stops, and the Forges row's "Connect ›" to it; the client world's joined story
runs it.

Acceptance:

- **The client domain** (`temper-client-domain`): a tokens draft (a module of its own beside
  `forge/admin.rs`, on its model): three private secrets, API token, git token and webhook
  secret, each with its write count, `checked` and `too_long`; their `forge::Event`s (as
  `AdminToken`'s), an act that sends it for a kept forge and one for a new forge from the place
  fields Your Forgejo shares with the admin's form. It sends only for a Forgejo of the connected
  mode not connected or stopped, or a new one: `w::Request::Connect { forge, how:
  How::ForgejoTokens }`, `forge` `Making::Kept(n)` or `Making::New(ForgeAsk::Connected(..))`
  (`forges.rs`' `making`, `(Forgejo, Connected)`), the value `forges::pasted_tokens(api, git,
  secret)`, a request name of its own, `Event::Carry`; the secrets cleared as it leaves, and
  again at a stop. Nothing is sent with a token empty (`checked`) or the bundle past its bound
  (`too_long`). The flow's step (`forge/connect.rs`): such a kept forge not connected or stopped
  is Details; connecting, Connecting; connected, Ready.
- **The view** (`temper-client-view`), in client/forge.md's words (5.4, *Your Forgejo*, *A
  connected Forgejo*, *Connecting, with its bot's tokens*, *Ready*, the actions' table's Connect
  row and the paragraph after it):
  - `managed` (`forge/connect.rs:321`) draws, after the admin's form, the tokens form: its
    heading "Connect with its bot’s tokens", its paragraph, the fields "API token", "Git token"
    and "Webhook secret" (`EntryKind::Secret`, empty values, their write counts; the last with its
    hint), and "Connect with its tokens"; its alerts "Required." and "The pasted values are past 4
    KiB, the most a request carries."; a refusal under the button in `ForgeRefusal`'s words
    (`connected`, "Forge N is connected already: replace its tokens on Forges.").
  - `body` (`:143-197`) sends a kept Forgejo of the connected mode to the tokens form alone (today
    it reaches `here`, an owned forge's set up), and a connecting one to Connecting's words:
    "Connecting forge F with its bot’s tokens." and "Checking that both answer as one user, not a
    site admin."; a stop shows the row's last connect's line under the button.
  - `forges.rs`' row (`personal`, `:91-192`): a Forgejo of the connected mode, not connected or
    stopped, offers "Connect ›", a link to `ConnectForge` at its number.
  - `bindings.rs` (`:157-173`): the three fields' meanings move their input straight into a
    `Secret` and the domain's events, never an action, as `AdminToken`'s does.
  - `words.rs`: each new phrase a `Word`, in `ALL`, numbered from the next free frame number
    (2155 at the survey; take the next free one), its English, and `PHRASES` and `SEGMENTS` raised
    by what is added (`english_phrase_declarations_match_the_vocabulary` and `SKEIN_CODEGEN_WRITE=1`
    rewrite `ALL`).
- **Tests:** the view's (`src/tests/connect_forge.rs`, beside `admin_details_*` at `:1968-2065`):
  Your Forgejo's Details offer both forms; the three fields are secret entries, and no token
  reaches the tree, an action or a rendered page; the local refusals mark their field; a kept
  connected Forgejo stopped shows the tokens form alone; Connecting's words; a stop's line under
  the button with the fields empty. `src/tests/forges.rs`: the row's "Connect ›". The domain's
  (`src/tests/`): the bundle sent decodes (`decode_pasted`) to `ApiToken`, `GitToken` and, when
  given, `WebhookSecret`; nothing is sent with a token empty; the drafts are emptied at send and
  stop. The client world (`temper/tests/client`): `front.rs`' `Joint::immediate` bridges
  `How::ForgejoTokens` as it bridges `ForgejoAdmin` (`:1465-1480`): the carried value decoded
  with `decode_pasted` to its items, the connect recorded, `ConnectAnswered::Connecting`; a new
  suite file, `forgejo_tokens.rs`, on a world made as `forges_edges.rs`' `admin_world` is but of
  the connected mode: Choose, Your Forgejo, its addresses and both tokens, "Connect with its
  tokens", Connecting; the world then ends the run connected and tells `Connected { id, login
  }`: Ready, "Connected as LOGIN."; and the tokens absent from the tree and the engine's logs, as
  `admin_private` checks.
- Status: in `temper/docs/design/client/forge.status.md`, the sentence that begins `5.4: a
  connected Forgejo's Connect with its tokens:` goes; in `client/README.status.md`, the entry
  `wire-connected` left, `3.5, 4.6, 8: a connected Forgejo's \`Connect\` with its tokens and
  Replace, on the Forges page (\`temper-client-domain\` sends neither); and Forget, ...`, reads
  `3.5, 4.6, 8: Replace, on the Forges page (\`temper-client-domain\` sends none); and Forget, ...`,
  its tail word for word.

## Inputs

- `temper/docs/plan/forgejo-connect/notes/common.md`; `notes/contracts.md`, C1 and C2.
- `temper/docs/design/client/forge.md`, 5.4 (as listed above, and *A pasted value is a field of
  its own*).
- `temper/docs/design/client/README.md`, 4.6 (`Making`, `Connect`, `ForgeRefusal`).
- The admin's form as built: `temper-client-domain/src/forge/admin.rs`,
  `temper-client-view/src/forge/connect.rs` `admin_form` (`:1484-1553`).

## Owns and may change

- `temper-client-domain`: `src/forge.rs` (events, the side's draft), `src/forge/**` (the tokens
  draft's module, `forges.rs`' `making` and acts, `form.rs`' acts, `connect.rs`' step), and
  `src/tests/**` for the new tests.
- `temper-client-view`: `src/forge/**`, `src/words.rs`, the three test files listed.
- `temper/tests/client`: `src/front.rs` (the bridge alone), `tests/suite/forgejo_tokens.rs` (new)
  and its line in `tests/suite/main.rs`; `forges_edges.rs` only to share `admin_world`'s
  making, by a parameter of mode.
- The two status files, as the acceptance says.

Pre-approved: the Replace on the row is `web-replace`'s, not this task's: draw no "Replace". If
`PHRASES`'s line meets another branch's edit at merge, it is regenerated, never merged by hand.
The mockups (`temper/docs/design/client/mockups/`) are not this task's.

## Steps

1. The domain's draft, its events and acts, `making`; its tests: `heavy cargo nextest run -p
   temper-client-domain --lib`.
2. The view's form, `body`, the row's link, the bindings and the words; their tests: `heavy cargo
   nextest run -p temper-client-view --lib`.
3. The world's bridge and story: `heavy cargo nextest run -p temper-client-world --test suite -E
   'test(forgejo_tokens) | test(forges_edges)'`.
4. The status edits; the citations check.
5. `cargo fmt`; old forms for temper; `heavy cargo clippy -p temper-client-domain -p
   temper-client-view -p temper-client-world --all-targets -- -D warnings`; their focused tests,
   and `heavy cargo check --tests` for `temper-client-tools`, `temper-client-shell-web` and
   `temper-client-shell-terminal`. Commit the code and tests, then the status text; push.

## Report

As the implementer's text says, with the words' numbers taken and the new `PHRASES` and
`SEGMENTS`.

+++
id = "web-replace"
part = "web"
kind = "feature"
needs = ["web-tokens", "ctl-connected"]
files = [
  "temper/crates/temper-client-domain/src/forge.rs",
  "temper/crates/temper-client-domain/src/forge/**",
  "temper/crates/temper-client-domain/src/tests/**",
  "temper/crates/temper-client-view/src/forge/**",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/forges.rs",
  "temper/crates/temper-client-view/src/tests/accounts.rs",
  "temper/tests/client/src/front.rs",
  "temper/tests/client/tests/suite/forgejo_tokens.rs",
  "temper/tests/client/tests/suite/forge_replace.rs",
  "temper/tests/client/tests/suite/main.rs",
  "temper/docs/design/client/forge.status.md",
  "temper/docs/design/client/README.status.md",
  "temper/docs/design/client/tools.status.md",
]
effort = "high"
minutes = 150
+++

# The Forges row replaces a GitHub App's key or a connected Forgejo's tokens; the joined stories

## Goal

Offer Replace on the Forges page as `temper/docs/design/client/forge.md`, 5.4's actions' table
writes it: in the row's opened part, a GitHub forge's new key or a connected Forgejo's new
tokens, sent as `Replace { forge }` with its value carried. Then the client world's joined stories
of both ways, the page's and ctl's (`ctl-connected`), which `client/tools.md`, 11.2 lists.

Acceptance:

- **The client domain:** a Replace draft per row being replaced (a module of its own): on GitHub
  one secret, the key, restored as Connect's is (`forges::restored_app_key`) and sent as one
  `Pasted` item; on a connected Forgejo three, API token, git token and webhook secret, sent as
  `forges::pasted_tokens`; their events, as the tokens draft's; acts to open and close a row's
  Replace and to send it. It sends only for a GitHub forge, or a Forgejo of the connected mode,
  connected or stopped: `w::Request::Replace(Replace { forge })`, a request name of its own,
  `Event::Carry`; its answers decoded as a connect's (`connecting`, `connect_refused`,
  `unknown`). Nothing is sent with the key empty, with both tokens empty, or past the bundle's
  bound; the secrets are cleared as it leaves and at a stop.
- **The view:** the row's opened part (`forge/forges.rs`, `personal`) offers "Replace" on such a
  forge; opened, its fields in the row: on GitHub "Private key" and "Replace the key", on a
  Forgejo "API token", "Git token", "Webhook secret" and "Replace the tokens", each field a secret
  entry as the tokens form's are; the page's own refusals ("Required." for the key; "Paste a new
  API token, git token or both." with both tokens empty; "The pasted values are past 4 KiB, the
  most a request carries."); a refusal under the button in `ForgeRefusal`'s words (`not
  connected`, "Connect it first."); answered connecting, the row says Connecting; a stop leaves
  the row as it was with its last connect's line. The terminal shell's GitHub row says, in the
  key's field's place, "Paste the key on this page in a browser, or run temper ctl forge replace
  N FILE, FILE the key's file, readable by you alone." The words as `web-tokens` adds them.
- **Tests:** the view's (`src/tests/forges.rs`): Replace offered on GitHub connected and stopped
  and on a connected Forgejo connected and stopped, not on a managed, owned or not connected
  forge; the fields secret and absent from the tree; the local refusals; the terminal's words.
  The domain's: each bundle decoded with `decode_pasted` to its items; nothing sent in the
  refused cases. The client world: `front.rs` bridges `Request::Replace` (the carried value
  decoded, the replace recorded, `ConnectAnswered::Connecting`); a new suite file,
  `forge_replace.rs`: the page's Replace of a connected Forgejo's git token alone and of a GitHub
  App's key, each Connecting, then connected again as the world tells it, no secret in the tree
  or the logs; and ctl's, on the same world: `forge connect forgejo FILE --api HOST:PORT --git URL
  --key KEY` with a tokens file, `connecting` and its connect recorded; `forge replace FORGE FILE
  --key KEY` with a tokens file of the git token alone, `connecting`; a replace on a forge not
  connected, exit 2, `not connected`; a tokens file with a line broken, exit 1, its line named,
  nothing sent. `forgejo_tokens.rs` may share the world's making with it.
- Status: in `client/forge.status.md`, "Replace and Forget, with no request on the wire" reads
  "Forget, with no request on the wire"; in `client/README.status.md`, the entry `web-tokens`
  left reads `3.5, 4.6, 8: Forget, on the Forges page and as the FORGE request \`Forget\`: no
  \`forget\` in \`temper-client-protocol-wire\`'s schema or in \`temper-service\`'s routing.`; in
  `client/tools.status.md`, 11.2's "; a connected Forgejo's connect by its tokens file and its
  replace" goes (11.3's real loop stays).

## Inputs

- `temper/docs/plan/forgejo-connect/notes/common.md`; `notes/contracts.md`, C1 and C2.
- `temper/docs/design/client/forge.md`, 5.4 (the actions' table's Replace row, the paragraph on
  a pasted value), 10 (*Mending a stopped managed or owned forge*).
- `temper/docs/design/client/tools.md`, 5.5 (`forge connect ... forgejo FILE`, `forge replace`)
  and 11.2 (area 6's stories).
- `temper/docs/design/client/README.md`, 4.6 (`Replace`, `ForgeRefusal`).

## Owns and may change

- `temper-client-domain`: `src/forge.rs`, `src/forge/**`, `src/tests/**` for the Replace.
- `temper-client-view`: `src/forge/**`, `src/words.rs`, the two test files listed.
- `temper/tests/client`: `src/front.rs` (the `Request::Replace` bridge alone),
  `tests/suite/forge_replace.rs` (new) and its line in `main.rs`; `forgejo_tokens.rs` only to
  share its world.
- The three status files, as the acceptance says.

Pre-approved: if the world's ctl calls cannot write a private tokens file as ctl reads it, make it
as `forges.rs`' or `first_repository.rs`' calls make their key files, and say how. No change to
`temper-client-tools` (`ctl-connected`'s) beyond a fixture the story needs, listed in the report.

## Steps

1. The domain's Replace draft and its tests: `heavy cargo nextest run -p temper-client-domain
   --lib`.
2. The view's Replace and its tests: `heavy cargo nextest run -p temper-client-view --lib`.
3. The world's bridge and stories: `heavy cargo nextest run -p temper-client-world --test suite -E
   'test(forge_replace) | test(forgejo_tokens)'`.
4. The status edits; the citations check.
5. `cargo fmt`; old forms for temper; `heavy cargo clippy -p temper-client-domain -p
   temper-client-view -p temper-client-world --all-targets -- -D warnings`; their focused tests,
   and `heavy cargo check --tests` for `temper-client-tools`, `temper-client-shell-web` and
   `temper-client-shell-terminal`. Commit the code and tests, then the status text; push.

## Report

As the implementer's text says, with the words' numbers and counts.

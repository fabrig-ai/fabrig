+++
id = "pg-words"
part = "playground"
kind = "interface"
needs = []
files = ["temper/crates/temper-client-view/src/words.rs"]
effort = "medium"
minutes = 30
+++

## Goal

Add to temper's client phrasebook every new English word the run's
client pages draw (the playground's banner and reset, the welcome's
unfinished forge, a forge's removal, a project's settings and its
deletion: `temper/docs/design/draft/playground-reset.md`, 4, 5, 6.1,
6.2, 6.4), so that the later client tasks, which run at once, use them
and none of them edits `words.rs`. Words only: nothing draws them yet,
nothing is removed or reworded.

Acceptance:

- the 38 `Word` variants below exist in
  `temper/crates/temper-client-view/src/words.rs`, each with its id and
  its English text exactly as given (bytes: `’` is `\xe2\x80\x99`, `·`
  is `\xc2\xb7`, `…` is `\xe2\x80\xa6`);
- `ALL`, `PHRASES` and `SEGMENTS` match the vocabulary (the
  declarations test passes without its write mode);
- every existing test of `temper-client-view` passes unchanged.

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md` and
  `temper/docs/plan/playground-reset-r4/notes/contracts.md`.
- `temper/docs/design/draft/playground-reset.md`, sections 4, 5, 6.1,
  6.2 and 6.4 (the words quoted there are the draft's own).
- `temper/docs/design/client/README.md`, 5.6 (the first run's words),
  for the phrasebook's tone; nothing there changes.

## Owns and may change

Owned: `temper/crates/temper-client-view/src/words.rs`.

No other file. Pre-approved:

- if a phrase's hole needs a bound other than `hole_bytes`' default
  (`joined` for a `Name`), give it the existing bound its kin has
  (`REFUSAL_BYTES` for the two `…Refused` words' reason); default:
  none needed;
- if an id in 2200..=2237 is taken (it is not at the base), take the
  next free block of 38 and say so in the report;
- if a text here breaks a phrasebook check (a segment or capacity
  bound), keep the text and raise the bound the check names, saying so.

## Steps

1. In `pub enum Word`, after its last variant, a group headed by the
   comment `// The playground reset's words (draft playground-reset.md,
   4, 5, 6.1, 6.2, 6.4).`, these variants in this order, with these ids
   in `Word::id` and these texts in `english` (holes in order;
   `Name` is `HoleKind::Name`):

   | Variant | Id | Segments | Holes |
   |---|---|---|---|
   | `PlaygroundBanner` | 2200 | "This is the playground: a scripted model and in-memory forge. Use the wizard to " | |
   | `PlaygroundSetUp` | 2201 | "set up temper" | |
   | `PlaygroundLeaving` | 2202 | "Leaving the playground…" | |
   | `PlaygroundNotLeft` | 2203 | "The playground was not left: ", "." | Name |
   | `PlaygroundTryAgain` | 2204 | "Try again" | |
   | `PlaygroundForgeLine` | 2205 | "Playground forge · in memory · ", "/", "" | Name, Name |
   | `PlaygroundNothingToAdd` | 2206 | "Nothing to add while the playground runs: set up temper to bring your own." | |
   | `ForgeNotFinished` | 2207 | "", " · not finished" | Name |
   | `ForgeNotFinishedDescription` | 2208 | "Its set up was cut short. Start over removes it, and you choose again." | |
   | `ForgeGitHubNotFinished` | 2209 | "temper’s App is made on GitHub. Go on to install it, or start over." | |
   | `ForgeGitHubStartOver` | 2210 | "Starting over removes this forge from temper. Its App stays on GitHub, yours to delete from its settings." | |
   | `ForgeStartOver` | 2211 | "Start over" | |
   | `ForgeGoOn` | 2212 | "Go on" | |
   | `ForgeStartingOver` | 2213 | "Starting over…" | |
   | `ForgeAppSettings` | 2214 | "The App’s settings on GitHub" | |
   | `ForgeRemove` | 2215 | "Remove" | |
   | `ForgeRemoveTitle` | 2216 | "Remove ", "?" | Name |
   | `ForgeRemoveGoes` | 2217 | "temper stops using it at once: its live changes are cancelled, its repositories leave their projects, and its tokens and secrets are erased. Chats that worked on it stay." | |
   | `ForgeRemoveOwned` | 2218 | "Forgejo on this machine stops, and its directory is removed with its repositories. Its downloaded binaries stay in the cache." | |
   | `ForgeRemoveManaged` | 2219 | "On the forge, temper’s bot and webhooks stay: remove them there if you wish." | |
   | `ForgeRemoveConnected` | 2220 | "On the forge, nothing changes: revoke the bot’s tokens there if you wish." | |
   | `ForgeRemoveGitHub` | 2221 | "On GitHub, temper’s App stays yours: delete it from its settings." | |
   | `ForgeRemoveFake` | 2222 | "The in-memory forge stops, and what it held is gone." | |
   | `ForgeRemoveConfirm` | 2223 | "Remove forge" | |
   | `ForgeRemoving` | 2224 | "Removing ", "…" | Name |
   | `ForgeRemoveRefused` | 2225 | "Not removed: ", "." | Name |
   | `ProjectSettings` | 2226 | "Settings" | |
   | `ProjectSettingsTitle` | 2227 | "", " · settings" | Name |
   | `ProjectSettingsNumber` | 2228 | "Project ", "" | Name |
   | `ProjectDelete` | 2229 | "Delete project" | |
   | `ProjectDeleteDescription` | 2230 | "Its chats and tasks are cancelled, then erased with their transcripts, and its repositories leave it. Its spend stays in the deployment’s totals. This cannot be undone." | |
   | `ProjectDeleteName` | 2231 | "Type ", " to confirm" | Name |
   | `ProjectDeleteConfirm` | 2232 | "Delete ", "" | Name |
   | `ProjectDeleting` | 2233 | "Deleting ", "…" | Name |
   | `ProjectDeleteRefused` | 2234 | "Not deleted: ", "." | Name |
   | `ProjectDeleteOwners` | 2235 | "Only the project’s owners can delete it." | |
   | `ProjectDeleted` | 2236 | "", " is deleted." | Name |
   | `ProjectGone` | 2237 | "This project is gone." | |

   In Rust each segment is a byte string, as the neighbours write them
   (e.g. `Word::ForgeRemoveTitle => (&[b"Remove ", b"?"], &[Name])`).
2. Regenerate `ALL` and the `PHRASES`/`SEGMENTS` literals with the
   declarations test's write mode, then run it without it:
   `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper-client-view
   --lib -E 'test(=tests::accounts::english_phrase_declarations_match_the_vocabulary)'`
   (the test's path as nextest lists it; check the count is 1), then the
   same without `SKEIN_CODEGEN_WRITE`. Add one line to the comment block
   above `PHRASES`: `// The playground reset adds thirty-eight words.`
3. `cargo fmt`; `heavy cargo clippy -p temper-client-view --all-targets
   -- -D warnings`; `heavy cargo nextest run -p temper-client-view`;
   `heavy cargo check -p temper-client-shell-web --tests`, `-p
   temper-client-shell-terminal --tests`, `-p temper-client-world
   --tests`, `-p temper-client-gallery --tests` (the view's direct
   dependents), one at a time. Commit,
   push.

## Report

fab's sections; `## Interfaces`: the variants and ids added (or the
block taken instead); any bound raised and why.

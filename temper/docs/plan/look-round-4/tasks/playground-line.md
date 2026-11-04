+++
id = "playground-line"
part = "playground-line"
kind = "feature"
needs = ["account-configure"]
files = [
  "temper/crates/temper-client-view/src/root.rs",
  "temper/crates/temper-client-view/src/tests/connect_forge.rs",
  "temper/crates/temper-client-view/src/tests/connect_model.rs",
  "temper/crates/temper-client-view/src/tests/frame.rs",
  "temper/crates/temper-client-view/src/tests/playground.rs",
  "temper/tests/client/tests/suite/playground.rs",
  "temper/crates/temper-client-shell-terminal/src/tests/golden/chat_delegation_landed.txt",
  "temper/crates/temper-client-shell-web/temper.css",
]
effort = "medium"
minutes = 60
+++

# What holds of `ready/look-fixes/playground-pane`

## Goal

Land what still holds of two commits that never merged. The queue refused the branch at 00:55Z on
`temper-client-shell-terminal`'s golden pages. These are an earlier round's fixes, no request's: this
task's commits carry no `Look-request` trailer. The cut read both against the frame now on main:

- **`a3616ef9e9` holds, whole:** the playground's line in the chats pane alone, on every page only
  while it is leaving. The rail's "Set up temper" makes its reason stronger.
- **`4bf27f716c` partly holds:** a chat in the side pane as a compact line. The rail doesn't replace
  the pane, but `temper.css`'s `#app:has(> .temper-rail) .temper-sidebar` rules outrank its selectors,
  so the commit as it stands does nothing but mix. Land its intent, not its hunk as it is.

Acceptance:

- **`a3616ef9e9`'s view and tests** are on your branch; `git cherry-pick -x` is allowed. Its design
  hunks are on the trunk already, so resolve `README.md` by keeping the trunk's. The other conflict,
  `tests/connect_forge.rs`, `the_forge_pages_have_navigation_and_the_playground_banner`: keep the
  trunk's `b"Shop"`, with the commit's `is_none()`.
- **The golden page** `chat_delegation_landed.txt` loses its playground lines, the refusal's cause:
  regenerate with `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper-client-shell-terminal -E
  'test(each_fixture_page_prints_as_its_golden_file)'`, and read the diff: only those lines move.
- **On `Welcome`** while the playground runs no line shows, as `the_banner_is_one_line_with_its_set_up_link`
  pins: the welcome's own "Set up temper" goes on to the Forge step.
- **`4bf27f716c`'s rules** go in, with `#app:has(> .temper-rail) .temper-sidebar .temper-row` and its
  three neighbours removed (keep the `.temper-rows` rule above them), so that the compact line
  applies under the rail: title and when, then status · TN, the whole line its link, no spend. Its
  hover takes `var(--gf-hover)`, not the old palette.
- Rendered at 1440, 1000 and 420 in the gallery's chats states: nothing overflows the pane. On a phone
  the pane's rows stay readable.
- `README.status.md`'s *3.5, 4.8, 5.1, 5.6* entry goes; `browser.status.md`'s **Not yet** goes.

## Inputs

- `temper/docs/plan/look-round-4/notes/common.md`, but for its Commits.
- `temper/docs/design/client/README.md`, 3.5 (*Its frame*), 4.8, 5.1, 5.6; `browser.md`, 2.5.
- `git show a3616ef9e9`, `git show 4bf27f716c`.

## Owns and may change

Owns, as `files` lists them. May also change `temper/docs/design/client/README.status.md`,
`browser.status.md`, as above. Nothing else of `docs/design/`.

Options, each pre-approved:

- If the trunk's other tests pin the line on a page the commit removes it from, follow the commit.
- `temper/docs/design/client/journeys/*`: reprint if a page moves; `welcome-playground/02.html` is the
  chats page, where the line stays.
- If `record-landing` lands first and a journey's page moves, follow it.

## Steps

1. The cherry-pick, the golden.
2. The rules, the render.
3. **Check,** `notes/common.md`: `heavy cargo nextest run -p temper-client-view -p
   temper-client-shell-terminal -p temper-client-world -p temper-client-journeys -p
   temper-client-gallery`, clippy, push.

## Report

Which of the two commits landed and how, and what the frame superseded, in words for the user: fab
carries it to them. The render's result.

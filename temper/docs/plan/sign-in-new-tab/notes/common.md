# sign-in-new-tab: what every task holds to

- **No test reaches a real host.** A provider in a test is skein's fake issuer (`skein_fake_peers::oauth::Peer`) or temper's `--fake-backends`.
- **Failing tests first,** in the lowest tier that shows each. A task's new test fails before its change and is seen to fail for the rule's reason.
- **Words are the design's,** `temper/docs/design/client/README.md`, 3.5, with temper's curly apostrophe and its ellipsis.
- **Not yours:** `temper/docs/design/client/journeys/**` (the look's owner, fabrig/ui, holds the words and pages there), and the `look-requests` branch.
- **Keep hunks small in `temper-client-view` and `temper-client-domain/src/pages.rs`:** other drafts are changing them, and fabrig/ui has priority. Move nothing that a task doesn't need moved.
- **The browser tier runs Chromium alone.** What a task reads of Firefox or Safari, it says is read and not run.

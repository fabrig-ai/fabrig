# journeys: status

2026-11-08. Where temper's code lags or departs from [journeys.md](journeys.md).

Section 3.6 has no known gap in the TUI's three kinds of original,
`Kind::Approval`, `Kind::Credential` and `Kind::Bearer`, or the shared
`Originals::new`, `redact_field` and separate original and shape scans.
The TUI uses these with its own records; the browser's redaction and
scan retain their originals and refusals.

Section 9 has no known implementation gap in the terminal's drawing
route: the request pages remain hand-made references, nine gallery
states draw the shared view, and `chat-draft` and `welcome-playground`
prints include the terminal button. `chat-draft`'s opening paragraph
renders Markdown emphasis in the shared document pane. Render and
real-key comparisons remain section 10's after-landing work.

Section 9's `chat-title` has no known implementation gap. Its hand-made
heading page is a request reference; no recorded journey's heading
changes, so no recording or print needs regeneration. The account is
commit `4283afdd91`; temper/ui's rendered comparison remains section
10's after-landing work.

Section 9 has the Models gallery states and the picker link to that
project's Models tab. The page 07 account is commit `415e659c54`; its
render comparison remains temper/ui's later work (section 10).

Unlisted sections have no known gap. Section 9's `connect-flows` pages
remain hand-made request pages, with the shipped states in the gallery;
no request CSS or script is part of a journey. The `project-landing`
recording is refreshed at `37e4a1aa55`. Render comparison remains
temper/ui's after-landing work (section 10).

Section 9 has no known gap in the flow's drawing route. The gallery
measures no box and prints the fallback; existing recordings contain no
flow. A rendered flow in a chat and its document remains section 10's
after-landing comparison.

Sections 8.2 and 9 have no known gap in `chat-draft`'s drafts-on start:
its setup leaves the fresh project's value alone, its recording has no
drafts toggle, and its account names `again chat-draft` and twenty prints.
That fake-model walk reaches the goal and its Draft link back to the chat;
a real-box recording remains the person's later work (section 10).

- **Not yet:**
  - 9: the recordings of `welcome-set-up`, `welcome-playground`, `project-landing` and `chat-draft` were made on fake boxes whose stock held template 3 and charter 3, the configuring chat's, and their `me` and settings records still list them; each is recorded again (8.1) on a box whose stock no longer does.
  - 3.5, 9: no recording from a real box has been through `add`, and no journey is `real`: the `real-journeys` request of 2026-11-05 leaves that to the user's own recording, so the guard has met fake secrets alone, through the shipped host natively.
  - 3.6: `welcome-set-up` was added before the redaction and keeps its fake box's link secret (3.5), until it is next added, because that request left the journey's files to its owner.
  - 8: nothing refuses `again` on a real journey. No recording says it is real, so `again NAME` would walk it on a fake box and, where its acts held, add that walk over it. Seen by reading.
  - 8.1: a typed secret's act holds its `x`s once added, so `again` types those. No committed journey types a secret, so no run has shown it.
- **Departs:**
  - 3.6: the API-key field carries a pasted trailing newline rather than trimming it. Redaction preserves that newline and replaces both the whole-value and trimmed-value digests, matching what the host carries so the replay holds.
  - 5.1: a `--redact` TEXT that is empty or starts `--` is the usage line, exit 2, so such a text can't be named, because a TEXT left out must not take the next flag for one.
  - 6: the refusal's process tests (`temper/tools/journeys/tests/suite/refusals.rs`) compare the listings of the journeys' root and of `tools/journeys`, a level each, and the print's test compares a whole root, where the section has nothing written on any path. No test traces a process's every write, so that `add`'s writes all stand after its last scan is seen by reading `add`.

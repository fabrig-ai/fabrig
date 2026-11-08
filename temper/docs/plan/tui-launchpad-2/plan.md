# tui-launchpad-2: the plan

The user's four requests of 2026-11-08 from their walk on a real box, and
the look request `tui-chat-title-below`
(`inputs/request.md`). Design written by this cut:
`temper/docs/design/client/README.md`, 6.12 to 6.15 (and section 11's
after-landing check); `temper/docs/design/client/tui/journeys.md`, 3.2,
8.2 and 11; `skein/docs/design/ui-screen.md`, 5.2; their status files;
`temper/docs/design/draft/tui.status.md` (Design departs, the user's
requests of 2026-11-08); `launchpad-chat/README.md`'s last frame.

## Parts and tasks

| Part | Task | Crates | Needs | Minutes |
|---|---|---|---|---|
| `rule-title` | `rule-title` | skein-ui-screen | | 40 |
| `look` | `look` | temper-client-tui (+ call sites) | | 80 |
| `seam` | `seam` | temper-client-shell-tui, temper's `tui.rs` | | 70 |

Later batches add `left-off` (the launchpad's arrows, view and host),
`chat-rule` (the chat's title in the rule above the box) and `journeys`
(recording `launchpad-chat` again, the look request's account, the
status entries closed).

## Decisions

- **Item 1, the left-off chats:** "Where you left off" shows up to three
  live chats, newest first (it showed one), so that the arrows have
  chats to move through. The selection is the TUI's own state, moved by
  the view's `LeftOff` event: skein's options menus always hold an
  active item and must follow their field in one form, which would mark
  a chat before any arrow and put the list after the box. With none
  selected, either arrow selects the newest; Enter on the empty box opens
  the selected chat (`Act::Go`); Esc, typing and leaving clear it.
- **Item 2, `/effort`:** a one-line change in the TUI's own `offered`;
  `temper-client-view`'s `shared` module is unchanged (`shared::offered`
  already offers `/effort` wherever the picker is), so the browser's
  terminal is untouched. Home's choice already reaches `Act::StartChat`.
- **Item 3, the band:** each of the background's channels moved by 20,
  saturating, toward 255 on a dark ground and 0 on a light one (by the
  ground's own luma, whatever `/theme` says): an equal step in
  lightness. The background is now asked at every start, also where
  `COLORFGBG` decides the scheme, so a terminal that sets `COLORFGBG`
  but doesn't answer OSC 11 waits up to `limits::BACKGROUND` (200 ms)
  at start. The palette drops `sel`, which nothing paints any more.
- **Item 4, bullets:** temper's own fix, a `tui-bullets` group around
  the agent's Markdown whose mark `•` wins for every unnumbered item
  under it (skein's nearest-token rule); skein's mark rules stay as they
  are.
- **Item 5, `tui-chat-title-below`:** skein's renderer gains two things
  (an end-only title lays no start spaces; a rule's end part keeps two
  glyphs across), since the TUI's box rule is skein's border. The place
  is `temper_client_tui::place(address, boxed)`, for `tui-box-front` to
  call too. "On a box" is the engine's link file read at start naming
  the TUI's address; `temper serve` alone writes the same file and reads
  as a box too. The journey `again` runs the TUI apart from the box's
  link file (to walk the approval), so `launchpad-chat` shows
  `127.0.0.1:PORT` where the request's frame drew `box:40691`: an
  explained difference in the account. The title is cut to 20 Unicode
  scalar values at a word.
- **Formats changed before v0.1.0, no migration:** the seam's `Start`
  gains `background` and `boxed`; the TUI's recording gains the start
  facts `background` and `box` (version stays 2; absent reads as none);
  the TUI's look names gain four tokens and lose `tui-star` and
  `tui-chat-top`; the palette loses `sel`.

## Overlaps

- `tui-native-secret` (integration): its `world-file` branch changed
  `temper/tests/client/src/tui.rs`; task `seam` adds the two `Start`
  fields' literals there (compiler-forced). Its `serve-file` branch wrote
  the link file this plan reads for "on a box".
- `post-landing-probes`' finding-465 repair (a fast chat losing its
  opening question) is in the TUI's chat: expect a catch-up merge in
  `temper-client-shell-tui/src/host.rs` or the domain.
- `tui-box-front` follows `tui-native-secret` and calls `place`.
- `temper-client-view`'s `shared` module: no change.
- Inside the plan, `look` and `seam` both edit
  `temper-client-shell-tui/src/host.rs` in the first wave, in separate
  hunks (`look`: the two `tui::theme` calls and one `Host::event` arm;
  `seam`: `Start` and the start handling).

## Critical path

`look` (80) → `chat-rule` → `journeys`, with `seam` (70) → `arrows`
beside it; about four hours of implementer time end to end as it stands.

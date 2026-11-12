# view: status

2026-11-11. Where the tree's code lags or departs from [view.md](view.md).


- **Not yet:** 6: the *dated* form, `TimeForm::Dated` [in flight: change 784].
- **Departs:** 1: at 760 px and under, `temper.css` hides the accounts' and the
  playground's banners on every chat's page and `NewChat`
  (`#app:has(> .temper-session) > .temper-accounts`), where the section draws
  them there, because the request `ui-phone` of 2026-11-05 has those pages whole
  at `100dvh`, which a banner above them would cut. With no usable account "No
  model yet · Connect a model" still stands above the composer; no line says there
  that the playground is running or leaving. Seen by reading; no test holds it.
- **Not yet:** 4: no manual proof on record shows long messages reading
  whole and the over-bound composer in a browser and a real terminal. The request
  `long-words` leaves that proof after landing and adds no browser story because
  the tier is over budget. The joined client's world now reads 66 messages of
  538 and 1,149 bytes through the real engine and link on a chat and an agent's
  page (`sixty_six_long_messages_read_to_their_start`,
  `an_agents_page_reads_long_lines_through_older`); a fresh walk of the reported
  browser history remains unrecorded. The decoder's 512-byte field bound and a
  malformed answer treated as unreachable account for that chain, so its cause
  is no longer unknown.
- **Not yet:** 3: the rename's form asks the focus, and so does an open reply:
  `temper-client-view`'s `pages/ask.rs`, `reply_form`, calls `b.focus(0)` with no
  guard, where `pages/chat.rs`, `document.rs`, `settings.rs`, `new_chat.rs` and
  `ask_page.rs` ask only while no rename is open. So an owner who presses "Rename
  PROJECT" with a reply open on Home or Needs you meets skein-ui's
  `Builder::focus`, at most one per tree, which asserts. The `ui-rename-focus` task
  `rename-row` has opening the form on such a page not panic; no reason is recorded
  for this site. Seen by reading; no test holds it.
- **Design departs:** `pages/README.md`, *Away*, `pages/goal.md`, 2, and `view.md`, 6, and `pages/settings.md`, 2, from the request `ui-away`
  of 2026-11-05, each decided by its plan's cut, for the user to see:
  - 6: every clock in the browser reads in the person's zone, where the request
    asks it of Away's 08:00, because `TimeForm::Clock` is one form and read
    universal it would say "Away until 06:00". The offset is read once, at start,
    so a change of daylight saving shows at the next load, and
    `forge::away::next_eight` takes that one offset, so the night the clocks change
    its 08:00 can be an hour off. The terminal stays universal (`README.md`, section 5).
  -  the chip is a second button in the rail, the phone's top bar, where the
    request has it in the banner: a project's page has no banner (3). The
    terminal's is the rail's button, where the request has its `navigation`, which
    a project's page does not draw.
  -  "Limits" focuses the section's heading, by a count in `Pages`, and a
    browser brings it into view. `#away` is no anchor: `Address` holds none, so no
    address reaches the section.
  -  the toast names the project, where the request's words have "fabrig’s".
    "Away: agents decide within PROJECT’s limits.", "Away didn’t change. Try
    again." and "That was overturned already." are words the request does not name.
  -  a row whose task has ended says "landed…" or "ended…", overturned or not,
    its took still struck, where the request's list has "Overturned: it asks you
    again" first: a refused message leaves the row marked, and its task won't ask.
  -  Home's away form is not drawn while To begin shows, where the request has
    it with any row showing.

- **Design departs:** 1, from the request `ui-small-1` of 2026-11-06,
  item 1's unconditional used-link explanation: the engine remembers the
  last 8 used or expired links only until restart. A forgotten used link
  reads "This link isn't from this temper. Use the one its terminal
  printed." The plan chose bounded memory rather than stored history
  (jig's `protocol/client/sign-in.md`).

- **Design departs:** 3, from the greenfield target, by the request `ui-repo-link` of 2026-11-06: the mock's
  switcher row is one button holding the name and the small words; here the small words are a sibling of the
  button, since a link can't sit inside one, and they are a link with its "↗" where the repository has an address.

- **Design departs:** 3, from the greenfield target, by the request
  `ui-rename-focus` of 2026-11-06: closing the rename's form leaves the project
  menu open with the focus on its summary, where the mock draws the menu closed,
  since a disclosure's fold is skein's page part's (`skein/docs/design/ui/web/README.md`, 5). The field does
  not select its name on opening, and no toast says "Renamed": the request names
  neither, and `README.md`, section 5 holds them. On a new chat or Ask temper's page the closed form
  leaves the focus on the page's own field, where the request has the opener or the
  summary, because that plan's cut keeps a page's own request. The field's hidden
  label says "Rename", where the mock's says "Project name": no one sees it, and
  that plan's cut took no new word for it.

- **Design departs:** 4, from the request `long-words` of 2026-11-06, by its
  plan's cut: an edit past `long_text`, 131,072 bytes, keeps only its UTF-8 prefix
  and the count of characters beyond it in the domain (`pages/composer.rs`,
  `held`). A mounted browser field keeps the full edit until its value count
  changes; a new drawing and the terminal have only the prefix, where the request
  has nothing typed dropped or kept stale. The cut bounds the client's kept edit;
  `README.md`, section 5 keeps retaining the whole beyond that bound open.

- **Not yet:** 3, *The rail's parts*: no read gives the rail a standing
  role (`../draft/standing-roles.md`).

- **Design departs:** 3, from the greenfield target, by the request
  `ui-owner-menu` of 2026-11-06, at the user's word: the rail's foot row is the
  button of a menu with "Accounts" and "Sign out", where the mock draws the
  person's name as plain text, and a browser over 760 px signs out there rather
  than on the `Accounts` tab. `web/greenfield/README.md` notes it.

- **Design departs:** 1, from the look request `outside-pages` of 2026-11-08,
  decided by its plan's cut: the top bar's link region holds the link's state in
  every state, "Signed out", "Connecting" and "Live" among them, where the request
  draws none of these, and the browser shows it only while it says "Offline", by a
  rule the request's overlay lacks. The terminal's first line prints the region's
  word (`line.md`, 4), and the story worlds read "Live" there.

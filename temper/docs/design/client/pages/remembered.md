# What temper remembers about you

The person's own page of what Ask temper keeps about them
(`../../draft/ask-temper.md`, section 7): the lines of their party scope (jig's
`domain/engine/briefs.md`, 2.1), which only they read, each changed or deleted
there. It draws what the engine holds, with no placeholder.

- **Its address,** `Remembered`, at `/remembered`, of no project: the
  person's menu (`../view.md`, 3) and, on a phone, the `Accounts` tab's last row (`settings.md`, 1)
  open it. It takes the rail's frame and the
  column of Deployment's tabs, 720 wide, with no kicker and no tabs, under its
  own level-1 heading, "What temper remembers about you", and its lede, "What
  Ask temper keeps about you across your chats, when you ask it to. Only you
  see these lines." No entry of the rail is current on it.
- **Its read,** on entering and after each act's answer: jig's `Read::Notes
  { scope: Party { party }, description: None, after: None }`, `party` the
  person's own number from `me`, then each next page until none is left; a
  party scope holds at most `notes.party_entries` lines (jig's
  `domain/engine/briefs.md`, 2.1), so one page answers it whole. No watch: only
  the person and their own Ask temper chats write it, and a chat's kept line
  shows when the page is opened again. Until it answers, the page keeps what
  it last knew (`README.md`).
- **A line a row,** in the order of the scope's names, oldest first, as Ask
  temper numbers them: the line, in the body's type; under it, in small type,
  who kept it and when, "Ask temper chat 14 · Today · Thursday 22 October"
  for a line a chat kept, its first reference the chat's number, or "Changed
  by you · Yesterday · Wednesday 21 October" for one the person wrote, its
  writer the party, the day by `../view.md`, 6's day's rule; and at its right
  "Change" and "Delete", quiet buttons, as Deployment's rows' acts are.
- **Change** opens the row's field in place, its line in it, with "Save" and
  "Cancel"; "Save" sends jig's `EditNote { project: 0, note, scope: Party {
  party }, revision, description, body, references }`, the field's words
  trimmed as the description, the body empty and the references as read, at
  the revision read; "Cancel" and Escape close it unchanged, the focus back
  on "Change". At most one field is open; an empty field sends nothing.
- **Delete** sends jig's `DeleteNote { project: 0, note, scope: Party {
  party }, revision }` at once: a line is short, and Ask temper keeps another
  only when asked.
- **A refusal** says why under its row, in a line, and keeps the row as read:
  moved, "This line changed meanwhile; it shows as it is now.", the page
  reading again; past the line's bound, "A line is at most 120 bytes.", the
  bound jig's refusal names; any other, jig's words for it (`../view.md`, 1).
- **With none kept,** a quiet band in the column, as Repositories' with none
  (`README.md`): the tag "Nothing kept yet", and "Ask temper keeps a line here when
  you ask it to remember something, such as how you like its answers."
- **Full,** at `notes.party_entries` lines, a quiet line under the last: "12
  lines, the most Ask temper keeps: it asks you which to change before it
  keeps another."
- **Its look** is Deployment's rows' (`settings.md`, 1, *The rows*): each row a hairline
  under it, 14 px 4 px, no card and no shadow; its style under the `settings`
  marker of `temper.css`. At 760 px and under, a row's acts go under its
  line, each at least 44 pixels tall to a press (`../web/look.md`, *What a
  phone presses*).
- **The terminal** has no such page: the TUI's pages are `../tui/pages/README.md`'s, and
  `Remembered` falls back to its launchpad, as Deployment's tabs do.
- **Its tests:** the domain's read and acts, each request as sent and each
  answer's page, as rows of `temper-client-domain`'s page tests; the view's
  rows, field and refusals as `temper-client-view`'s; the person's menu with
  its three items; and the gallery's states (`../web/gallery.md`), none
  kept, three lines, twelve, a line being changed and a refusal, each by day
  and by night, its prints read at a laptop's width and a phone's.

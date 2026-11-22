# The terminal inbox

This document holds the terminal inbox, split from [`needs-you.md`](needs-you.md).

## 1. The terminal inbox

- **The inbox in the terminal** (`inbox.html`; the browser's is *Needs
  you,* 1) shows what needs the person, then the
  updates, each entry its card (`../README.md`, section 1), under two headings, each
  with how many entries it lists beside it, the updates' as new ones
  since what is read leaves them (below): "Needs you 2", "Updates 2
  new", or "0". The needs' count is every open need, each row of every
  page the needs' read holds (`needs-you.md`, 1, **What the client keeps**, *The needs'
  rows*), "2+" where that read is cut; the updates' "2+ new" where the
  inbox's document says its list was cut (jig's `more`, an owner inbox's
  next page, a filtered page's `cut`), so that a count never says less
  than there is; with
  none, once that section's read has answered, "Nothing needs you." or "No
  updates.", and no count before it. Its first heading's
  row holds "+ New chat", a link to `NewChat` (`/chats/new`, its text area
  focused: no act of its own), then
  its filters as folded menus, in order: projects (only with two or more),
  goals, then Kind. Each menu names its choice: "All projects" or the
  project's name; "All goals" or "Goal · TN", or "Milestone · Store shape"
  where the filter's task is a milestone (`../../draft/milestones.md`, 5.3),
  its document read once by number and `milestones::is` of its chip
  (`milestone.md`, 1); "Kind" or "Kind · Question". The goal filter takes a
  root, so a milestone's holds every need under its goals.
  The project menu holds All projects and each project; the goal menu holds
  All goals and the Goal field; Kind holds All kinds and each kind. A choice
  moves to the filtered inbox's address and folds its menu (`../web/README.md`, 2.3).
- **A card that needs the person** begins with its kind's tag, then
  "PROJECT › TN TITLE", for its own task alone, and its wait at the right
  (`../view.md`, 6). A proposal holds no proposing task's title, so its line stops at
  TN; questions, held tasks and person tasks use their documents' titles.
  Every TITLE a card draws, on this page, on Home and in the inbox, its
  milestone's and goal's in the from line among them, is cut past 80
  characters (`shared-behaviour.md`, 1, *A task's title on a line*).
  A batch proposal keeps the mockup's Why, Goal, Landing and Budget rows:
  its reason quoted, its executor and specification, its landing resources
  and grants with priority, and its funding once. Template and charter
  names are not supplied, so their existing numbered words remain. Its
  acts are "Accept", "Reject with reason" and "Open TN ↗"; a chat's card
  keeps "Open in inbox ↗". Under its words, its reason line, with Stop
  asking and its offers numbered as any control (`needs-you.md`, 1, *Why it
  reached you*); its project's `decided` document is one *Decided
  for you* watches, the unfiltered inbox watching `me`'s first projects.
- **A finding that is the person's** (`../../domain/areas.md`, section 5)
  is a card that needs them, from the forge's `findings` subject with
  `mine` and `new`, watched for each of `me`'s projects as `decided` is
  (`../forge/records.md`): its tag "Finding", then "PROJECT › P/N TITLE",
  and its wait since it was filed; its kind, severity and area, "no area"
  where it has none; the last hand-off's why, quoted, with who handed
  it; and its acts, "Hand on", an area's menu and a line of why, "Route
  to a goal", the project's live goals' menu, "Backlog" and "Drop with
  reason", each the forge's request, a refusal said in its words. Kind
  holds "Finding" among its choices. The card leaves once the finding
  leaves `new` or stops being theirs.
- **A group of findings** (`../../draft/home-for-a-person.md`, 5;
  `../../domain/areas.md`, 6, *Its groups*): a question, by `message` or
  a person task in words, asked under a goal, a chat or a standing role
  alike (the triager's runs ask under theirs), whose whole words name findings in their line
  form (`../../domain/agent/agents.md`, 2, a `Findings:` line) is a
  group's card, on this page and on Home (`home.md`, 1, **Findings,
  drawn**, *A group's card*), where one of them at least is read. It is
  the question's card still: its kind, its answers, its why line, Later,
  At and the held send are a question's, and it is the inbox's card, so
  Kind and the filters place it as a question.
  - *Its findings' read:* the findings store watches, for each project
    whose person's findings it watches (`needs-you.md`, 1, *A finding that is the
    person's*: as `decided` is), the project's open findings, the subject `findings` with `state` `new`
    and `mine` false (`../forge/records.md`), its own handle beside the
    person's, while one of that project's open cards names findings, and
    unwatches it once none does or the project leaves the store. Those
    findings are the triager's, which the person's watch, `mine`, never
    holds.
  - *Its rows:* the findings its line names that are among its
    project's open findings' rows, matched by project and number, in the
    line's order; a finding named of another project, past the
    document's rows (`cut`), or no longer `new` is no row. A card whose
    findings' document is read and that has no row is a plain question.
  - *Its card waits,* as a question's waits for `decided` (*Why it
    reached you*, *Its card waits*), until its project's open findings are
    delivered, or their watch is refused or ended, so that its rows never
    move its choices under a finger. A card whose project the store
    doesn't watch, or with no project, waits for nothing and is a plain
    question.
  - *The domain's group,* in `temper-client-domain-pages`'s
    `pages::findings`: `Group<'a> { project: u32, noun: Box<[u8]>, phrase:
    Box<[u8]>, rows: Box<[&'a w::FindingRow]> }`, the noun and phrase its
    `Group:` line's, empty with none, and `Group::severity(&self) ->
    w::Severity`, its rows' highest, and `Group::oldest(&self) -> Wall`,
    its rows' earliest `at`. `Store::group<'a>(&'a self, need: Need<'_>)
    -> Option<Group<'a>>` gives a card's, none for any card that is no
    group's. `Need::severity(self, store: &Store) -> Option<w::Severity>`
    is a finding's, a group's highest, none for any other card; the
    cards' order (`needs-you.md`, 1, **What the client keeps**) and `counted` read it.
    `Need::ready` holds a group's card while it waits.
  - *On this page,* the open card is a question's (`needs-you.md`, 1, **Needs you**,
    *The card*), but its from line: on the unfiltered address the
    project's chip; then the group's chip of findings, "3 bugs" on its
    highest severity's ground (`home.md`, 1, **Findings, drawn**, *A chip
    of findings*), its count the rows' in digits and its noun, or "3
    findings", "1 finding" with none; then the phrase, " · " and "the
    oldest 1 d", its oldest row's span, or the one row's span alone, "1
    d"; then " · " and the asked task's link, its TITLE as the question's
    from line has it; " · later" with it. Under its question, its rows, a
    row a finding (`home.md`, 1, **Findings, drawn**, *A finding's row*),
    each kind said only where the rows hold more than one; then its
    context, "What the agent saw", its reason line, its choices, the
    recommended one first, and *Tell it something else*, as a question's.
    A group's row's title is text, no link: no page shows a finding the
    person doesn't hold (`../../draft/home-for-a-person.status.md`). Its
    *SHORT* is its question's line. In the terminal's inbox the card is a
    question's, its words as written.
- **An update's card** has its heading as the link that opens it and marks
  a result seen. It repeats no project line or task-number link. The heading
  says "TN TITLE replied", "TN TITLE is done", or "TN TITLE ended";
  while its document is unread, the existing "Task N" words are the heading's
  link. Then the result's first line and the unread count and age (`../view.md`, 6).
  A reply has no preview: its document carries no reply's first line.
- **A question** in the inbox is a delegate's, asked of a task the person
  requested, their chat among them. It has no heading of its own: its line,
  then its words as written. "Answer in words" is folded until opened,
  holding "Answer words" and "Answer"; "Open TN ↗" opens the asked task.
  The answer goes to that task, which relays it to its delegate. "Pending"
  and "Answered" keep today's request outcomes, and a refusal stays under
  its card. This is the terminal's card; the browser's reads the
  question's words as *Needs you* (`needs-you.md`, 1) says.
- **Seen** is how far the person has read: jig's read positions (jig's
  `client/domain/pages.md`, section 3), one over their results and one
  over each chat's turns. An update is seen when it is opened, and no
  page offers "Mark seen":
  - *a chat's turns,* once its page shows them: as its window loads,
    and as turns arrive while it is open, through the newest its window
    holds;
  - *a result,* when the person follows its entry's link from the
    inbox: through that result, and so through every result before it,
    since jig keeps one position over them. A result opened from
    another page, or its entry left unopened, stays unread.

  A position only moves forward, with one `Seen` in flight for each;
  one refused or failed is sent again at the next move. What is read
  leaves the inbox's updates, and a task forgotten once read is erased
  then (jig's `domain/people/README.md`, section 5). The pages draw no mark of
  their own for what is unread, beyond the inbox's counts of it
  (above).

## 2. Open questions

None.

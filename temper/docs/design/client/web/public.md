# Reading without signing in

The web client's anonymous mode: a visitor reads a public project
(jig's `domain/people/public-reading.md`) through its public documents,
jig's and the forge's (`../forge/README.md`, section 9), never signing in and
never streaming (`../../draft/first-release.md`, 5.3).

## 1. Its pages and addresses

| Page | Path | Reads |
|---|---|---|
| `PublicProjects` | `/public` | jig's *projects* |
| `PublicBoard { project }` | `/public/projects/<project>` | jig's *board*, and the forge's *landings* for its queues' lines |
| `PublicTask { project, task }` | `/public/projects/<project>/tasks/<task>` | jig's *card*, and the forge's *change* where the task makes one |
| `PublicLandings { project }` | `/public/projects/<project>/landings` | the forge's *landings* |
| `PublicLanding { project, task }` | `/public/projects/<project>/landings/<task>` | the forge's *landing*, through a member's change's task |

- **By temper's address codec,** as every page's (`README.md`, 2.3); `<project>`
  and `<task>` are public identifiers, 16 lowercase hexadecimal digits
  (jig's `domain/people/public-reading.md`, 8), never numbers. A path under
  `/public` that does not read is `/public`.
- **Served as every page is:** the server answers each with the page
  and the bundle (`README.md`, 2.2); nothing on the server is new but the public
  kinds the forge declares to jig (`../forge/README.md`, 9.1).

## 2. How it reads

- **No sign-in, no stream, no small store.** At an address under
  `/public`, the shell starts the public mode in place of `jig-client`'s
  sign-in: no `Transport`, no credential, no watch. It reads each
  document its page shows by jig's public `GET` through jig's HTTP
  client end (jig's `protocol/client/default-http.md`, 5), and decodes it with
  jig's and the forge's public records alone.
- **It reads again by itself,** when a document's freshness ends (the
  `max-age` its answer gave, less the `Age` a cache added), while its
  tab is visible; a hidden tab reads nothing, and reads at once when
  shown. The proxy's cache answers most of these reads; the engine sees
  one per document per `max-age` (`../../protocol/sign-in.md`, 2.4).
- **Nothing shown past its freshness.** A document is shown only until
  its freshness ends: a read again that is answered busy or fails keeps
  it until then, marked as of when it was read, and after it the page
  drops it and says "Not available just now.", reading again after
  `Retry-After`, doubling to a bound of a minute. So a project made
  private leaves an open page within the `max-age`, as it leaves the
  proxy; a copy already shown or saved is not withdrawn.
- **Unknown** shows "Not public, or not here." with a link to
  `/public`, and drops what the page showed.
- **Anyone, signed in or not,** sees the same pages at these addresses:
  the answers are the same for every visitor. A signed-in person's
  cookie goes with the `GET` and is never read. The top bar (`look.md`,
  *Outside a project*) offers "Sign in" at its right, with or without
  Google's way, and nothing else of a person's: a link to the page's
  origin, `/`, whose page signed out is the sign-in page where Google's
  way is given, and otherwise the column "Signed out" with the printed
  link's words (`../view.md`, 1, *The pages not about the work*). It
  needs the page's origin alone, as the view's `Ways` give it.

## 3. What it shows

- **Titles and the project's name, no other text:** the records carry
  no text but these (jig's `domain/people/public-reading.md`, 3), so a project is
  its name, and a goal is "Goal" and a task "task", each with the first
  eight digits of its public identifier, then its title, with "…" where
  cut; an executor is its kind, "an agent", "a procedure", "a person".
  A title is the first line of a spec an agent wrote from what a person
  said, so a chat's words can show here; ctl's publication help and completed
  answer say so (`../ctl/tools.md`, 3). A preceding acknowledgement is Later
  (`../ctl/README.md`, section 11), and no page makes a project public.
- **The look:** one plain column, the same at every width from 320 to
  1440 px, in the client's type and colours (`temper.css`'s tokens), its
  rows modelled on the Chats page's: no table, no monospace, and nothing
  overlapping at any width. A long title wraps inside its row.
- **A row:** a goal's is its identifier and title, then, muted, on one
  line that wraps cleanly, its phase in words, for how long, and its
  spend: "running · for 14 minutes · 126 spent". A task's is the same
  form without spend, indented below its goal: "task 3f2a9c01 Write the
  parser", then "running · for 14 minutes". No row shows a budget or a
  priority; a goal's priority orders the board.
- **An ended row** says how it ended and how long ago: "landed · 2 hours
  ago · 126 spent". A goal reads "landed", "failed", or "ended" for one
  closed with neither, by the same endings as the Goals page; a task
  reads "done", "failed" or "ended".
- **The board:** the project's name as its title and, under it, "What
  temper is doing on this project, as it happens. Read-only." and, in the
  same muted form, "Work that ended before this project went public, or
  before temper last restarted, isn't shown."; each goal's row, with its
  tree's rows below it and the count of lines folded; "No goals to show
  yet." when it has none. Below the goals, only when the forge's landings
  hold a queue or a landing, "Landings", a link to them, with the queues'
  lines, and "No landings to show yet." until one has. The link looks like
  one whatever stands under it, which may be nothing: the mock's `.link`,
  in the ink at a row's size and underlined, not a section title's small
  muted capitals.
- **A task:** its row, its executor first among the muted words ("an
  agent · running · for 14 minutes"), its lineage up to its goal as
  links, its delegates' rows, and where it makes a change, the change's
  step in words, CI's state, its gates' verdicts by number and kind, and
  "Its change: 789ada1", the merge's short commit once it has landed and
  the head's until then.
- **Public lines wrap at desktop and phone widths:** the identity link and
  title form one group; the row's executor, phase, duration and spend form
  a second group with gaps between facts. Titles wrap without truncation
  beyond the public record's cut. Public content is a block reading column
  with at least 16 CSS pixels of horizontal gutter, including navigation.
- **The landings:** the turn, each queue in order, without places, and
  the recent landings, each landed commit a link to its public landing;
  "No landings to show yet." when there are none. A landing's member
  reads "Its change:" and its landed commit, or its candidate until
  then. What a stock queue merged is a recent landing in this same form
  (`../forge/README.md`, 9.2): its line says landed, its change and its commit,
  and its public landing holds its change, with no gates. A landing's
  stale proofs show only when it has any: a zero draws nothing, on a queue
  merge's landing as on a landing by turn.
- **The projects:** each public project's name, a link to its board; "No
  public projects yet." when there are none.
- **A short commit** is the first seven digits, plain text on a change
  or a landing's member. In the recent landings list it links to the
  public landing, as above. A commit is held in 32 bytes, a 40-digit one
  zero-padded, so a page never prints one whole.
- **No link to the forge:** a commit, a pull request and a CI run are
  never links to the forge, for the reason in `../forge/README.md`, 9.3.
- **Words never touch:** where a line holds several, a visible separator
  or space stands between them in the page's text, not only in its
  layout.
- **Public data comes only from the public records:** their text is titles
  and the project's name, never a person's name. The view adds the page's
  own words specified above.

## 4. Its tests

- **Step tests:** the address codec's public paths, both ways, and a
  malformed one (a number, upper-case, short or long identifiers) read
  as `/public`; the public mode's reads, its timer from the answer's
  freshness and `Age`, a hidden tab reading nothing, busy keeping the
  page until its freshness ends and then dropping it, backing off,
  unknown's main; a goal's row and a task's, an ended row's words, each
  page's empty words, the board's line of what it doesn't hold, Landings
  on the board only with a queue or a landing, and as a link, whose look
  the sheet's rule holds, a queue merge's line and its landing, a landing's
  zero stale proofs drawing nothing, a 40-digit commit's seven digits as
  plain text, and no link to the forge on any page.
- **Fuzzy tests:** the address codec's parser over seeded paths under
  `/public`, never panicking and printing back what it read.
- **The browser tier** (`testing.md`): a public project on fakes read at
  `/public`, its board, a task and its landings, by a page with no
  sign-in and no stream, the view's text checked against the public
  records' fields alone. Its board and task lines, goals and delegates,
  are also read at 1440 px and 420 px with a long title and populated facts:
  one geometry query reads all text matches at each page and width, with the
  count of any matches beyond the browser's limit checked. Text boxes stay
  separate, titles remain readable, navigation stays inset and content stays
  within the viewport. A missing text box reports its text and measurement stage. A project made private has its pages
  unknown at the next read. The goal it reads is under way: a landed goal's pages
  hold nothing a browser alone shows (`testing.md`, 6). Every page includes the "temper" and
  "Sign in" top bar (2), including unknown and empty pages. Its landings page
  then reads "Landings", optionally "Queue" and its one member, then "No landings to show yet."
  or the stock queue's merge. The member reads "task " and eight lowercase hexadecimal
  digits, " · Goal " and eight such digits, then " · Ready " and "just now",
  "a minute ago" or decimal digits and " minutes ago". The merge reads
  "landed · task ", eight lowercase hexadecimal digits, " · " and seven such
  commit digits. These are the whole text joined without separators; nothing may
  precede or follow it. One run reads only one form, so the tier's library holds
  the queued and unqueued forms as a table in the focused suite, and refuses
  any other line. The story's other page and geometry checks stay the same.

# The view's navigation

The view's navigation is split from [the view](view.md).

## 1. Navigation

**The rail** (`../draft/surfaces.md`, section 2; the whole story's mockups,
`web/whole-story/`, each page's `nav.rail`), on a project page (1): a
navigation named by the open project, in this order:

- temper's name, named "temper home", a link in the browser to where
  the root goes, `rail::root_home` (`web/addresses.md`, 1; draft-2's
  answer of 2026-11-18: the mark goes where the root goes, inside and
  outside a project), and in the terminal to where "Home" goes;
- the switcher (*Projects in the rail,* below);
- "Home", to the open project's `Home`, and to `AllHome` on a page across
  projects; "Needs you", to *Needs you* filtered by the open project, and
  on a page across projects to *Needs you* across them, `Inbox` with no
  project, with the count of what needs the person there
  (`pages/needs-you.md`, 1, *What the client keeps*), omitted when zero
  (*All projects,* below); "Work", to
  `Work { project, filter: All }` (`pages/work.md`, 1, *Work*); "Team", while the rail's
  parts show it (*The rail's parts,* below), to Team's page
  (`../draft/surfaces.md`, section 6); "Ask temper", to the person's newest
  Ask temper chat open in the project or else `Ask { project }` (`pages/ask-temper.md`, 1, *The
  first chat*), only while `me` lists template 5 among the project's
  templates (`../draft/ask-temper.md`, section 9; the look request
  `ask-temper-place`); a hairline, `div.temper-rail-sep`, named nothing;
  and "How work runs", to `Settings { project }` (`pages/settings.md`, 1).
  Kind 7's switches hide no entry of the rail: Goals' and
  Changes' hide Work's filters (`pages/work.md`, 1, *Work*). "Work" is current on `Work`, `Goals`, `Changes`, `Chats`,
  `NewChat`, a `Chat` but an Ask temper chat's, and a `Task`; "Ask temper"
  on `Ask` and an Ask temper chat; "How work runs" on `Settings`,
  `Repositories` and `Adopt`;
- the new box, `div.temper-rail-new`, one dashed box. While the project
  offers drafts (`drafts_offered`, `pages/chat/document.md`, 1), it is "+ New draft or change", the
  button of skein-ui's menu (`Role::Menu`; skein's `ui/web/elements.md`),
  named "New draft or change", opening a panel under the box 288 pixels
  wide, past the rail's edge and over the page. Its items, a name and a line
  each: "Draft", "A design session that ends in a goal, best had in the
  terminal.", to `NewChat { draft: true }`; "Change", "Ask for one change.",
  to `NewChat { draft: false }`; and, while the rail shows "Ask temper",
  "Ask temper", "Ask about this project, or change how its work runs.",
  where the rail's "Ask temper" goes. With no drafts offered, as at Pair
  (`../draft/surfaces.md`, section 12), the box is "+ New change", a link
  named "New change", to `NewChat { draft: false }`, with no menu. The box
  takes the hover's grey; open, the button keeps it. Skein's keys, arrows
  and Escape; a press on an item or outside closes it; a focused item
  takes the hover's grey. A phone doesn't show the box. Which templates it
  offers is fixed until `../draft/template-places.md` is taken up;
- at its foot: the Away switch, while `pages/shared-behaviour.md`, 1's *Away* shows it; while the
  playground runs and is not leaving (`forge/family.md`, 7),
  "Set up temper", the playground's way out (`pages/welcome.md`, 2); the link's state, as
  1 has it; "Behind" while a page's watch reopens, beside the link's
  region in the browser, and in the region in the terminal, whose first
  line reads it (`line.md`, 4). Over 760 pixels the browser draws the two
  out of the foot's flow, as one word with its dot at the right of the
  rail's first row, temper's name's, so that the foot keeps its height and
  place in every state: "Live" while the link is live and no page's watch
  reopens; "Behind", its dot amber, in place of "Live" while the link is
  live and a page's watch reopens; otherwise the link's own word,
  "Connecting" or "Offline", alone, with no "Behind" beside it (the look
  request `rail-word`, the user's word of 2026-11-15); "Deployment", to `Accounts`,
  Deployment's first tab (`pages/settings.md`, 1), current on
  `Accounts`, `Forges`, `Models`, `Builders`, `Members`, `Limits`,
  `ConnectForge` and `ConnectModel`, with the dots "Forges" and "Accounts"
  carry below, each its state in words; the person's name, a line in the
  terminal and the person's menu in a browser; and "Sign out did not reach
  the engine." while `me` or the page cannot say otherwise (`line.md`, 6), the
  terminal's "Sign out" being the `Accounts` tab's (`pages/README.md`);
- the person's menu, a browser's: the person's name is the button of
  skein-ui's menu (`Role::Menu`; skein's `ui/web/README.md`, 5), in a
  `div.temper-rail-person`, drawn as the foot's other rows. It opens a panel
  above the row, as wide as it, in the look of the switcher's menu. Its items:
  "Accounts", to `Accounts`; "What temper remembers about you", to
  `Remembered`, Ask temper's memory of the person (`pages/remembered.md`); "Telling you", to
  `Told`, where pushes go (`pages/told.md`); and "Sign out", the
  `Accounts` tab's act. A press
  on an item, Escape and a press outside close it, the focus back on the row
  unless that press took it elsewhere. Open, the row keeps the hover's grey. A
  phone has no foot, and signs out on the `Accounts` tab (`pages/README.md`);
- the night switch, a browser's, the rail's last child, after its foot and drawn
  at the foot's right: a button named "Night mode", pressed while the night look
  is on, a moon by day and a sun by night, drawn by style; on a phone, at the top
  bar's right. Where the top bar draws in the rail's place (1), the same button
  is the top bar's last child, at its right (the look request
  `night-switch-top`). It shows on every page that draws the rail or the top
  bar, at every width,
  wherever the rail's other parts go: in the rail folded beside a chat's Split,
  at the folded rail's foot; on a phone's chat layout (`pages/chat/README.md`, 1, *One layout for
  every chat*), where the rail and its top bar go, at the right of the chat's
  own bar, which keeps room for it (the look request `night-mode`, which
  places it at the rail's foot or a phone's top bar). Pressing it turns the look, on every page of that browser, and
  keeps it (`pages/chat/terminal.md`, 1, *Its look, kept per browser*; `web/look.md`, *Night*). The
  terminal has none.

**While To begin shows** (`pages/home.md`, 1, *To begin*), on `Home` and on `Ask { project }`
alone: the rail's navigation holds "Home", "Ask temper", while `me` offers
template 5, the hairline and "How work runs", and no other entry
(`../draft/surfaces.md`, section 2; `web/whole-story/pair/first-run.html`);
`div.temper-rail-new` is not drawn; the foot is as above, "Deployment" among
it, so that today's Settings is "How work runs" and "Deployment" there too.
The tab bar holds "Home", "Ask" and "Deployment", three equal tabs, How work
runs reached by the switcher's menu as on any phone page (*On a phone,*
below). Both shells draw the
rail so, the terminal reading its entries as the browser does. The view asks
`temper_client_domain::pages::home::beginning(root, project, wall)`, true on
those two addresses while `facts(..).to_begin()` holds; every other page draws
every row (the look request `to-begin-two-steps`, the user's word of
2026-11-07).

**The rail's parts** (`../draft/surfaces.md`, section 12): what the rail and
the tab bar show is one pure function of the domain's,
`temper_client_domain::pages::rail::parts(&Facts) -> Parts`, which both shells
draw from and neither decides again. `rail::facts(root, project, wall)`
gathers its facts, plain values, once a build; no fact is a page's address,
and what is current is the view's. Its rules:

| Part | Shows when | Its fact |
|---|---|---|
| "All projects", the switcher's first row | the person is in two projects or more | `me`'s projects |
| "Needs you"'s count | it is more than zero | `pages().needs().count(..)`, as above; on a page across projects, the count across them (`pages/needs-you.md`, 1, *What the client keeps*) |
| "Team" | the project has a second member or a standing role | the project's people, from its document, and its live standing roles, from the roles' watch (`pages/team.md`, 1, *Its reads*) |
| "Ask temper", in the rail, the tab bar and the new box's menu | `me` lists template 5 for the project | `me`'s templates |
| the Away switch and chip | `pages/shared-behaviour.md`, 1's *Away* shows it; on All projects, it shows for any of the person's projects (`pages/shared-behaviour.md`, 1, *Away*, *On All projects*) | `Domain::away_shown`; on All projects, `pages::away_all::across` |
| the new box's menu, "+ New draft or change" | the project offers drafts | `drafts_offered`; else "+ New change" alone |
| To begin's cut, above | `home::beginning` holds | `home::beginning(root, project, wall)` |
| "Home", the hairline, "How work runs", "Deployment" | always | |
| "Needs you", "Work" | always, To begin's cut aside | |

"Forges", "Accounts" and "Adopt a repository" are no entries of the
rail's navigation: they are Deployment's tabs and the Repositories tab's (`pages/README.md`).

**On a phone** (`../draft/surfaces.md`, section 10; the whole story's
`phone-home.html`, `phone-need.html` and `phone-tell.html`), at 760 px and
under, the rail is a top bar showing the switcher alone, and the tab bar closes
the page: in the browser, a navigation named "Sections" after the main, drawn
at phone width alone (`web/look.md`). Its tabs are one pure function of the
domain's, `temper_client_domain::pages::rail::tabs(&Parts) -> Tabs`, over the
rail's parts, which the browser draws from and decides nothing again:

| Tab | Shows when | Current on |
|---|---|---|
| "Home" | always | as the rail's "Home" |
| "Needs", with its count after it in the signal colour, "N+" where cut, named by its words, "Needs 2" | the rail's "Needs you" shows: its count while more than zero | as the rail's "Needs you" |
| "Work" | the rail's "Work" shows | as the rail's "Work" |
| "Ask", named "Ask temper" | the rail's "Ask temper" shows | as the rail's "Ask temper": Ask temper's page, an Ask temper chat and `Tell` |
| "Deployment" | To begin's cut holds, the rail's "Work" not shown | as the rail's "Deployment" |

Each tab goes where its rail entry goes, so on a page across projects "Home"
opens `AllHome` and "Needs" *Needs you* across them, with the rail's count
(*All projects,* below).
So four tabs in four columns as wide as their words, or To begin's three,
"Home", "Ask" and "Deployment", equal, and one fewer while `me` offers no Ask
temper. Team, How work runs and Deployment are reached by the switcher's "Team
for PROJECT", "How work runs for PROJECT" and "Deployment" (below), the phone
drawing no rail's foot and no person's menu. A chat's layout draws no tab bar
on a phone, but Ask temper's page and an Ask temper chat, the Ask tab's, whose
tabs close it under the composer (`pages/README.md`). The Away switch is a chip in the top
bar, on the rail's terms. Every target the top bar and the tab bar draw, the
switcher, the Away chip, the night switch and each tab, is at least 44 pixels
tall to a press (`web/look.md`, *What a phone presses*). The terminal has
no tab bar.

**No navigation outside a project** (1; the look request
`outside-pages`): a page that is no project's is drawn under the top
bar, which offers none; with no project yet, the column's acts are the
way on (`pages/welcome.md`, 1, *With no project yet*).

**No switcher outside a project** (`../draft/first-run-projects.md`,
7; the look request `outside-pages`): the top bar names no project;
`me` read with no project, the column's "New project" makes the first
(`pages/welcome.md`, 1, *With no project yet*), and once one is open the rail's switcher
lists them.

**Projects in the rail:** the same switcher, a bordered box with its
chevron, its menu a panel under it: "All projects", while the rail's parts
show it (*All projects,* below); each project, the open one first
and pressed, then, after a hairline, "Rename PROJECT" where the
rename is offered, the form "New project", and, on a phone alone (760
pixels and under), "Team for PROJECT" where Team shows (`pages/team.md`, 1),
"How work runs for PROJECT", to the open project's `Settings`, and
"Deployment", to `Accounts`. On a wider page the menu ends at "New
project", since the rail draws all three ("Team" and "How work runs" in
its navigation, "Deployment" at its foot; the look request
`switcher-links`, 2026-11-12): the three stay in the tree, which the
terminal prints, and the browser hides them by a look token of the
view's on each, from 761 pixels. Each project's row says, in small
type at its right, what it holds, from the `forges` watch's rows
(`forge/records.md`), the playground's project from the playground's read (`forge/family.md`, 7): its
one repository, "work/hello"; with several, its home then how many
more, "infra/deploy +2"; with none, "no repository"; and nothing
while the watch is refused, ended or not delivered. A long name is cut
with an ellipsis before its count, never the count. `me` does not say
which repository is a project's home (`forge/family.md`, 4), so its first adopted, the
first its rows list on the forge numbered lowest, stands in for it.
The repositories in a row follow the forge's repository ids ascending,
not the time they were adopted.
In the browser the row is two siblings, since a link can't sit inside a
button: the project's name, the button as above, and the small words. A
repository with an address (`forge/deployment.md`, *A repository's link*) is
the link "work/hello ↗" there, in the same place, size and colour,
underlined on hover, and opens the repository's page in a new tab; the
project's name alone switches project. "+2" stays outside the link, which
goes to the repository named. "no repository", the playground's and one
with no address stay plain words.
With no home known, "3 repositories" is the row's words; the stand-in
leaves that state unreached until the client is given the home. In the
terminal a row reads "PROJECT · work/hello", one run of words. Choosing
a project keeps the page where it can (`pages/README.md`): on `Home`, `Work`, keeping its
filter, `Goals`, its filter All, `Changes`, `Settings`, `Models`, `Builders`,
`Repositories` and `Ask`, that project's page; on `AllHome`, that project's
`Home`; on `Inbox`, across projects or on one, that project's `Inbox`, its
filter's kind kept and its goal dropped, since a goal is one project's, but
the filter whole where it already names the project chosen; on `Forges`, `Accounts`,
`Members` and `Limits`, the same page, the words naming the project changed.
The All projects row is the way back across projects, below.

**All projects** (`../draft/surfaces.md`, section 2): the switcher's first
row while the person is in two projects or more, a button named "All
projects", in small type at its right how many, "4 projects". Pressed, it
goes to a page across projects: on `Inbox`, its filter with the project
taken out, `/needs` having none; anywhere else
`AllHome` (`pages/home.md`, 1, *Home on All projects*). While the page is one across
projects, `AllHome` or `Inbox` with no project, and the person is in two
projects or more, the switcher's summary names "All projects" and its row
is the one pressed. In one project, the switcher has no All projects row
and never names it: its summary names that project and its row is the
pressed one on every page, those across projects included, in the rail
and a phone's top bar alike (the look request `one-project`, the user's
word of 2026-11-15). "Home" and "Needs you"
stay across projects there (`../draft/surfaces.md`, sections 1 and 2): "Home"
and the phone's "Home" go to `AllHome`, current on it, and temper's name in
the terminal; "Needs
you" and the phone's "Needs" go to `Inbox` with no project, current on it,
their count across the person's projects (`pages/needs-you.md`, 1, *What the
client keeps*). The rail's other entries keep the open project, the one last
open, so that "Work", "Team" and "How work runs" open on it, and the switcher
names it again there; "Team" shows by that project's parts. Whether the page
is across projects is one function,
`temper_client_domain::pages::rail::across(root)`, which the rail's facts and
both shells' entries read.

The acts of both, each one act and one request (`root.md`, 1.6):

- **Rename,** after the rows in the rail, offered while `me`'s role on
  that project is owner, the role jig gives, which the context line
  reads too (`pages/welcome.md`, 2); jig judges the request (jig's `domain/people/acts.md`,
  1). Pressed, the open project's row is the form, "Rename PROJECT",
  and the rail's "Rename PROJECT"
  leaves its acts while the form is open. The form holds the name's
  field, filling the row, its value the project's name and the focus on
  it, then "Save", primary and small. The field's Enter or "Save" sends
  jig's `Rename { project, name }`, its name without the spaces at its
  ends, the form's acts disabled while it is out; Escape closes the form
  and sends nothing: the form's "Cancel" names it (skein's `ui/README.md`,
  6.1), which the terminal shows and the browser's sheet hides. An empty
  name, or the name unchanged, sends nothing and closes the form,
  keeping the old name. Answered renamed, the form closes, and `me`
  brings the new name wherever the project's name shows; refused (the
  name taken, its shape, the role), the form stays and says why under
  its field. A closed form leaves the focus on the switcher's summary
  (skein's `ui/web/README.md`, 5), or on a page's own field where the page asks
  it (a new chat, Ask temper's page, a composer after a send, `pages/README.md`).
- **A new project:** the list ends with a form, "New project", holding
  a name's field, "+ New project" its placeholder, and "Create".
  "Create", or the field's Enter, sends jig's `MakeProject { name,
  policy: 1 }` in the deployment's scope, on the stock policy
  (`../protocol/configuration/stock.md`); an empty name sends nothing.
  Answered made, the field empties and the client opens the project
  once `me` lists it, on its `Home`, with no repository (`pages/README.md`),
  while the person is still where they asked it
  (`../draft/project-open-superseded.md`, 2). A later move supersedes
  that opening: once "Create" has sent, a navigation to an address other
  than the one shown, asked (the pages' `Act::Go`) or applied (a `Went`,
  or any return), or a project chosen (the pages' `Act::Project` and
  `Act::OpenProject`), cancels it, whether the answer has come or not and
  whether `me` lists the project yet or not. The creation still
  finishes: the field empties on the answer, `me` brings the project into
  the switcher's rows, and the page the person went to stays. Staying
  put, a `Went` to the address already shown among it, keeps the opening,
  in either order of the answer and `me`'s listing.
  Offered whatever the person's role, as above: jig judges it (an owner
  of the deployment), and a refusal says why under the field.
- **Its state:** the rename's form, at most one, with its project and
  words; the new project's words; each within jig's name bound; at most
  one request out of either, with its key; the new project's opening,
  armed when its request goes out, cancelled by a later move, and spent
  when it opens `Home`; and the refusal, with the form it answers.
  Choosing another project, or a page of no project, closes the rename's
  form.

## 2. Open questions

- **A project's home in the switcher** (`view-navigation.md`, 1, `forge/family.md`, 4): `me` does not say
  which repository is a project's home, so the first listed stands in
  for it, and a row with several repositories never reads "3
  repositories", the state `project-home`'s hand-made page draws. The
  project's own document does say its home, per project; reading it for
  each row is the candidate, or a fact of `me`'s, which is a protocol
  change and the user's. Default: the stand-in.
- **The rail's "N+"** (`pages/needs-you.md`, 1, *What the client keeps*): the standing document's
  `next` says more follows of needs or of updates alike, since a page holds
  every need and then the updates (`../../help/inbox.md`), so that a long list
  of updates, which no page of the browser's reads, says "N+" of a whole list of
  needs. It never says less than there is. Default: "N+", as that request has it.
- **The rename's opening and closing, as the mock draws them** (1; the request
  `ui-rename-focus` of 2026-11-06): the mock selects the name as the form opens,
  folds the menu as it closes and says "Renamed" in a toast. A view can neither
  select a field's words nor fold a disclosure (skein's `ui/web/README.md`, 12), and that
  request names none of the three. Default: the name unselected, the menu open with
  the focus on its summary, and no toast.
- **The switcher's name** (`view.md`, 1): the look requests name it "Project:
  NAME" apart from its summary's words, NAME; skein's disclosure names
  its summary by its words alone (skein's `ui/web/elements.md`).
  Default: named NAME, until skein's disclosure carries a label of its
  own.

# Talking with a role

## 1. The page

- **`Talk { project, task }`,** at `/projects/<project>/talk/<task>`: a
  person's talk with one standing role of the project
  (`../../draft/standing-roles.md`, section 6;
  `../../domain/agent/standing.md`, section 7), jig's talk (jig's
  `domain/tasks/standing.md`, 5.6). The terminal draws it
  (`../tui/pages/talk.md`), opened by `/talk`; Team's "Talk to it" is the
  web's later part (`team.md`, 2, *Team's later parts*), and nothing on the
  web links to it yet. A page of the project's frame: `project` is its
  project, for the rail's reads and Team's (`team.md`, 1, *Its reads*), as
  `Team`'s is.
- **The role's chat page, in its talk:** on `Talk`, `Pages::chat()` is the
  `chat::Chat` of the role's task, reading what `Chat { task }` reads
  (`README.md`, the table's `Chat { task }`): its task, its thread,
  `activity` a page at a time, `run` while one is live, and the proposals
  in its subtree, its cards. So a shell draws its conversation, its
  running line and its cards as a chat's. `Chat::talk(&self) ->
  Option<&chat::Talking>` is `Some` on a `Talk` page alone, `None` on every
  chat; it says what differs (section 2).
- **Its words are talk:** the page's send makes `Ask::Talk { project,
  task, words }` (jig's `client/domain/contract/requests.md`, 2), where a
  chat's makes `Ask::Message`, while the task's document offers the
  person `ActKind::Talk`; its words are held, pending, answered and
  refused as a message's are (the thread's `sending` and `answered`), and
  a refusal, such as another person's talk open, is drawn where a
  message's is, in the view's words (jig's `domain/tasks/standing.md`, 5.6,
  *Refused*). With `Talk` not offered nothing is sent, as a message not
  offered.
- **Leaving the page leaves the talk:** when the root leaves a `Talk`
  page, by any `Went` elsewhere, while the person's talk is open on the
  task's document (its `talk`'s `party` is the person's) and the document
  offers `ActKind::Leave`, the domain sends `Ask::Leave { project, task }`,
  one request under a key of `Pages`; its answer draws nothing, and a
  refusal or a lost answer is dropped: the talk then lapses (jig's
  `domain/tasks/standing.md`, 5.6, *It lapses*). Closing the shell sends
  nothing, and the talk lapses so too.
- **Not a chat's** (`../../draft/standing-roles.md`, section 6, *Not a
  chat*): the task's spec words, the role's own, are not drawn as the
  conversation's first words, as a chat's request is; and the chat's acts
  are not offered on a talk: pause and resume, close, start over, the
  picker's model and effort, a document's modes and a goal from it
  (`shared::offered` offers none of them there).
- **A proposal accepted in place:** a card of the talk (`Chat::cards`)
  that is a proposal of settings, accepted with `Act::Decide { act:
  Accept }` on a `Talk` page, is decided with running work moved,
  `contract::Running::Moves`, so that the role takes its new rules at its
  next run, which starts its talk's next session fresh from its Now
  (`../../domain/agent/standing.md`, 7, *A rule changes its charter*);
  elsewhere a card's Accept keeps `Running::Stays`, as today. Its
  confirmation and refusal are a card's, as anywhere (`../README.md`,
  3.6).
- **Its reads beside the chat's:** the role's Now, jig's `Read::Notes {
  scope: NoteScope::Task { project, task }, description: Some("now"),
  after: None }`, the lowest-numbered note so described, read as Team
  reads a role's (`team.md`, 1, *Each listed role's Now*): on opening, and
  again whenever the task's `last_activity` moves, one read at a time;
  none yet is a normal state. The project's findings document
  (`../forge/records.md`, the subject `findings`), for the role's areas
  (section 3): the page's project is among those whose findings Needs you
  watches (`needs-you.md`, 1), as Home's project is. Leaving the page
  leaves them.

## 2. What a talk holds

`chat::Talking`, the domain's public surface for a shell, each fact
`None` or empty until its read answers:

- `project: u32` and `task: u64`, the address's;
- `name(&self) -> Option<&[u8]>`: the role's name, its task's title, the
  chip's `title`, as Team's row names it (`team.md`, 1, *Standing roles*);
- `open(&self) -> Option<&contract::Talk>`: the task document's `talk`,
  while its `party` is the person reading: their talk is open;
- `from(&self) -> Option<u64>`: while their talk is open, the position of
  its opening, the newest `TalkOpened { party }` line of theirs among the
  lines read; `None` while it isn't among them;
- `cost(&self) -> Option<u64>`: the talk's cost so far, the sum of the
  `spent` of each `RunEnded` line after `from`, in the unit Team's
  sessions sum (`team.md`, 1, *Its sessions*); `None` while `from` is;
- `spend(&self) -> Option<(u64, u64)>`: what the role has spent of its
  period's budget and that budget, the task document's `budget`, as
  Team's row reads a role's (`team.md`, 1, *Standing roles*);
- `now(&self) -> Option<&team::Now>`: its Now, its body, when it was
  written and whether it was cut, Team's `Now`.

`pages::team::weekly(root: &Domain, project: u32) -> bool`: true while the
project's document, as Team reads it, holds an open period whose `next`
is seven days after its `began`, as How work runs says "a week"
(`settings.md`, 2, *Budgets*); false while unread or otherwise.

## 3. A project's areas

`pages::team::areas(root: &Domain, project: u32) -> Option<team::Areas>`:
the project's areas and its triager, as the findings document Needs you
watches for it gives them (`../forge/records.md`, the subject `findings`;
`../../domain/areas.md`), `None` while no document of the project's is
read; read for a `Talk` page's project (section 1) and the launchpad's
(`../tui/pages/launchpad.md`, 2). The talk's head and the launchpad's team
lines read it; Team's areas, its later part, will.

```rust
pub struct Areas {
    pub rows: Box<[Area]>,          // the document's `areas`, in its order
    pub triager: Option<Holder>,    // the triager's holder, none where it doesn't stand
}
pub struct Area {
    pub name: Box<[u8]>,
    pub holder: Option<Holder>,     // none where its owner doesn't stand (../../domain/areas.md, 3)
}
pub enum Holder { Task(u64), Party(u64), Role(u32) }
```

A role holds an area whose `holder` is `Holder::Task` of its task, and is
the project's triager while `triager` is; a person holds one whose
`holder` is `Holder::Party` of their party.

## 4. Open questions

- **The talk on the web** (`team.md`, 2, *Team's later parts*): Team's
  "Talk to it" opening the role's conversation, its look and where it
  stands, are P4c's (`../../draft/whole-story.execution.md`). Default:
  the browser draws `Talk` with a chat's layout from the same page,
  nothing linking to it, until P4c draws it.
- **Leaving when the shell closes** (section 1): a closed terminal sends
  no `Leave`, so the role's session stays warm until the talk lapses, 30
  minutes on. Default: so; a shell's closing that waits for its last
  requests could leave first.
- **A person's open talks, listed:** no read lists the talks a person has
  open across roles (jig's `TaskLine` carries no `talk`), so Where you left
  off lists none (`../tui/pages/launchpad.md`, 4). Default: none listed;
  `/talk NAME` opens the role, and its open talk with it.

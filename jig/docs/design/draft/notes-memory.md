# Notes as memory

Draft, 2026-11-04, from the user thinking aloud that day: notes used
by agents as memory of several kinds, "internal memories e.g. for not
losing track of things after compaction, small user-related
preferences that are remembered behind the hood and influence LLM
choices and judgement, things that the UI needs to show", "pretty much
like Claude memories but maybe with some slight organization". The
user adopted a part of section 5 on 2026-11-05 (its status says what), and
section 3, narrowed, for temper's Ask temper on 2026-11-07
(`temper/docs/design/draft/ask-temper.md`, section 7), and section 4 for
temper's standing roles on 2026-11-08, a standing role's Now
(`temper/docs/design/draft/standing-roles.md`, section 2); the rest is not
adopted, and no plan takes it up until the user does. It
starts from jig's notes as they stand (`../domain/engine.md`, section
10; `jig-domain-notes`), and its first consumer is temper's chats, whose
document is drawn beside their conversation (the look request
`configure-chat-draft`, `temper/docs/design/client/journeys.md`, 7).

## 1. In one page

- **One mechanism, four uses.** A note keeps what it keeps today: a
  description, a body, a scope, the tasks it names and who wrote it,
  written by a tool, read in two steps, corrected and deleted by
  parties (`../domain/engine.md`, section 10). A kind on each says
  what it is for (section 2).
- **Lessons,** today's notes: what agents learn and pass on, in the
  deployment, a project, a goal or a resource pattern.
- **Preferences:** a person's small preferences, kept for them and
  read by the runs that work for them, in a new scope, a party
  (section 3).
- **Working notes:** a task's own memory, what it decided and what is
  left, kept on purpose rather than left to a compaction's summary, in
  a new scope, a task (section 4).
- **Documents:** what a task shows its person beside the conversation,
  Markdown the client draws, in the task's scope (section 5).
- **Hints, never authority:** no note widens what a task may do; where
  a note and the facts disagree, the facts are right, as today.

## 2. The kinds

| Kind | Scope | Written by | Read by | For example |
|---|---|---|---|---|
| lesson | the deployment, a project, a goal, a resource pattern | a task its authority lets write there; a party | runs whose project reaches it, through their brief's index | "the staging database takes 90 s to warm up" |
| preference | a party | a run working for that party; the party | the runs working for that party | "Pat wants short review comments, in English" |
| working | a task | that task's runs | that task's runs, its whole index in each brief | "Decided: drafts before goals. Left: approval before landing." |
| document | a task | that task's runs | the client, beside the task's conversation; that task's runs | "How Hello works" |

- **The index groups by kind,** each line its description, as today;
  `recall` and a party's search take a kind as well as a name or words.
- **A run works for a party** when the party requested its task, or a
  task above it (`../domain/tasks.md`); a goal's coding agents work for
  the goal's requester.

## 3. A person's preferences

- **Told or noticed:** "remember that I…", or a correction the agent
  takes as lasting; the run writes a preference in its party's scope,
  and says so in its reply, as a coding agent says it saved a memory.
- **Seen and kept by the person:** a page of their own, "What temper
  remembers about you" in temper's words, lists them, each corrected or
  deleted there (`../client/domain/pages.md`, 6.4's notes, for one's
  own scope).
- **Read only for them:** a preference is in the brief's index of a
  run working for its party, and in no other's; no other party reads
  it, an owner included.
- **Small:** a party's scope holds few entries, each short; past the
  bound the run is told to correct one rather than add.

## 4. A task's working notes

- **The task's own memory:** a run keeps there what it must not lose,
  decisions, open questions, the plan's thread, written as it goes.
- **Whole in every brief:** a task's working notes are the first index
  its brief carries (`../domain/engine.md`, section 9), never evicted
  while the task lives, so that a fresh run (`../domain/engine.md`,
  7.4) and a compacted window
  (`smith/docs/design/domain/session.md`, section 8) find them.
- **Gone with the task,** by its retention (`../domain/tasks.md`,
  section 3); a working note worth keeping is copied by the run into a
  lesson.

## 5. Documents a task shows

- **A chat's document:** one note in the chat's own scope, the lowest
  numbered whose description does not begin `work:`, with no kind. A `work:`
  note is never drawn, and a chat with only working notes has no document;
  its description its title, its body Markdown, read by
  skein-ui's reader (`skein/docs/design/ui.md`, section 10), flows included
  once skein's diagrams draft is built. Every note's body has the room of a
  document: no bound of its own.
- **Read live with no new watch:** the chat's page already watches the task
  and its run, and only the task's own run writes the note, so it reads the
  scope again when a `note` call shows, and draws the newest beside the
  conversation (temper's Chat, Split and Draft); a revision says "edited
  just now", and the section changed last comes from the answer to that
  call, its first changed line.
- **Written by the run's note tool,** a first version whole, then by line
  ranges (`../domain/engine.md`, section 10), against the revision it read,
  refused as moved otherwise, and stored whole, so that a reader sees no parts.
- **Later, into a repository:** a document that matures may be proposed
  as an effect, a file in a repository, as a lesson may become a guide
  (`../domain/engine.md`, section 10).

## 6. Bounds and the store

- **Per scope, as today:** entries and bytes per scope; a party's and a
  task's bounds of their own.
- **Erased with their scope:** a task's notes with the task, a party's
  with the party (`../domain/people.md`), as a project's go with it
  today.
- **Indexes kept per scope in use,** evicted least recently used, but
  a live task's working index, which its runs need whole.

## 7. Its order, when it is taken up

1. **jig:** the kind on an entry; the party and task scopes, in the
   note tool's checks (`../domain/authority.md`, section 3) and the
   brief's index.
2. **smith:** nothing: a compaction's summary keeps the conversation,
   and a working note what the run chose to keep.
3. **temper:** the person's page of preferences.

## 8. Open questions

- **When:** the rest of this draft is the user's; it keeps the idea, and no
  plan takes it up until then (2026-11-04).
- **Preferences:** a person's across the deployment, or per project;
  whether the person confirms each before it is kept, or sees it kept
  and corrects it. For temper's Ask temper, settled: across the
  deployment, kept when the person asks, and seen and corrected on their
  page (`temper/docs/design/draft/ask-temper.md`, section 7).
- **Working notes and compaction:** whether a working note replaces
  part of the summary smith writes, or only sits beside it.
- **Documents:** whether a task that isn't a chat shows one. A chat's
  stays with its chat, closed or started over, as a task's history does.

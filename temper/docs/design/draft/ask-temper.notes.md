# ask-temper: the reasons behind it

This holds the reasons for `ask-temper.md`, moved from [`notes.md`](notes.md), section 10.

The reasons behind `ask-temper.md`, from the user, 2026-11-07.

- **One chat, grown from configuring.** The user asked for "an overall
  'embedded' AI in temper": configuring a project's charters and rules to
  their workflow, "where are we at?", and questions about a goal's run, "the
  user can start chats/session with this prompt/model and ask what they
  need", with a single prompt "unless it turns out bloated". The three share
  temper's model and the voice, and differ by a few paragraphs.
- **What they care about most:** "that the model understands temper/jig well
  and can reason accordingly". So the instructions carry temper's model with
  its reasons, at the height fabrig.ai explains it to people, rather than rules
  learnt from findings.
- **Small.** The configuring chat's first request was 52 KB on ChatGPT and 62
  KB through Claude Code, 43 KB of it tools, most of them ones it may never
  call: `delegate`, `amend` and `change` alone were 14 KB. Claude Code's own
  prompt is written for a coding terminal and tells the run of tools it lacks.
  The user asked for no workspace, no file or shell tools and no Claude Code
  prompt.
- **Tools on demand, settled by pruning.** The user asked whether tools could
  be described when asked for. With four or five tools there is little left to defer,
  and a provider's own deferral would be two mechanisms; one reading tool by
  address keeps its declaration a line and its addresses on a page.
- **Not ctl's text.** The user was wary of `temper ctl`'s output for this
  reader. On a playground box it printed the raw authority line in `me`,
  money in two units, charters by number alone, each call's JSON and long
  cursors, and commands to copy: fit for an agent driving temper, not for one
  explaining it. The queries are right, so `see` reads them and answers in its
  own way.
- **Code for mechanisms.** Half of `configuring.txt` taught a model what code
  can do: a note's line arithmetic, copying a repository's host and port, who
  may approve, a large proposal through a note. The tools do it instead.
- **Green field, beside.** "I'd rather start greenfield, we can have some switch
  that keeps what's in place, but let me play with the greenfield version
  myself since it's something that requires human judgement." So no sweep of
  the old instructions, the configuring chat kept, and no golden judging it.
- **It changes nothing but settings,** and most probably replaces the
  configuring chat (the user's answers to the two questions).
- **In the rail, the old chat dropped.** Having read the draft, the user:
  'the "configuring agent" is now the built in temper AI', and "we should find
  another place for it, not Settings". Offered both side by side to compare:
  "just drop the old one". It keeps the name Ask temper with no persona, since
  its instructions have it speak as temper and the landing page already says
  "temper proposes". The rail over a box on Home, which would invite requests
  for work it cannot do.
- **Memory, guarded.** "Adding notes to the tool… would be the model's memory,
  so it can adjust a bit to the person preferences and remember past choices",
  with care: "it tends to create bloat and unexpected behaviors. The best would
  be a notes system with guardrails". jig's `draft/notes-memory.md`, section 3,
  already held it; the user adopted it for this chat, narrowed to its runs and
  shown to the person alone, since a chat's transcript is read by the
  project's roles. Past choices are records already, in the settings' history
  and proposals with their reasons.
- **Pictures like fabrig.ai's.** The user asked that it "communicate very well
  graphically", like the home page's "Four examples, not modes" rail, which
  "depict well what is code, what is LLM, what is human", with "zooming the
  system at various level of detail, for example the landing queue (how it
  works, what knobs are there), the goal execution (their roles, their
  prompt)". Views drawn by code can't be wrong and cost the model a line;
  sketches cover what doesn't exist yet. The look is temper/ui's.

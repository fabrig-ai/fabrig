# reference-m4: surveys as sub-agents on Claude Code

Carry out milestone **M4** of `temper/docs/design/draft/reference-case.execution.md`. Its entry in section 3 is the scope. Its section 2 says how a milestone runs: its design and its code go through this plan as one pipeline, and its designers write the design into the trunk with the specs that cite it (the repository's `DESIGN.md`).

## The scope (M4's entry)

- **Executes:** `temper/docs/design/draft/reference-code-production.md`, section 5, step 2: surveys as inspect-only sub-agents, side by side, each answering with its report. The reasons are in `temper/docs/design/draft/notes.md`, N10.
- **Design to write:** smith's `protocol/programs.md`, section 2, settling its section 9's open question, together with smith's `domain/run.md`, 5.3. The shape: on a program backend (Claude Code), smith serves its `sub_agent` through the program's MCP server, each child a program session of its own at the model and effort its LLM choice names. Inspect-only children run side by side.
- **Builds:** smith alone; jig and temper need no change.
- **Configures:** nothing in this plan. The coordinator's and the designer's charters name sub-agents later (M3).

## Old code, for reference only

No old branch touches program-backend sub-agents (hel8's `~/src/fabrig/coord/old-repos-value/`).

## Walk

fabrig/qa walks M4 as a rung once it lands: a run on Claude Code asking inspect-only sub-agents side by side, each answering with its report. Mark the slice that makes this story run `walk = true`.

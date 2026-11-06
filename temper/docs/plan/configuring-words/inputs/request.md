# configuring-words: the configuring chat speaks the person's terms, and never sends them to a terminal

The user's report, 2026-11-06, from a chat recorded on a box from main at `359cb69106`, gpt-6.1-sol, signed in to ChatGPT. The design is `temper/docs/design/draft/configuring.md`.

## What happened

- "what is the current setting" got an answer in the engine's vocabulary: "The current settings are revision 1, all defaults, with no pending proposals", then a table of "Charter 1 ...", "Template 2; tracked at priority 1", "Charter 4; read-only", and "Project-specific Ways and Questions reads returned 'unknown,' so I cannot confirm those settings." The user found it cryptic. Two of its five `settings` calls printed `unknown / end`: by the answer, project ways (kind 7) and project questions (kind 6).
- "can we change it to require a blocking human review instead?" got "Which forge account should provide the human approval?". The user answered "free", their own login on the box's forge. The chat called `read_permission` (printed "Write"), then read configure, project, landing-rule and party, and replied: "The forge reports collaborator ID 3 with Write access, but these tools don't confirm that it is free or establish its project role. Could you share the output of temper ctl DIR project show 1 (using your deployment directory for DIR)?" The user: "I shouldn't be pointed to ctl". The person is in the browser.
- Every reply ended with "Nothing has changed" or similar.

## Where it comes from (confirm it)

`temper/crates/temper-service/src/stock/configuring.txt` and the chat's tools. Its last paragraph has it explain with `settings` and `read_docs` in their concepts (settings, revision, proposal, charter, template, policy, landing-rule), and nothing asks for the person's words. Its approvals paragraph has it establish eligible people by forge user id and role first. Its tools can't read a project's members, their roles or their forge identities (what `project show` prints), and `read_permission` gives a user id with no login. So the agent can't match "free", and falls back on help pages, which speak `temper ctl`, and relays them.

## What the user wants

1. **Never a terminal.** The configuring agent never sends the person to a terminal or `temper ctl`. What it can check, it checks with its tools. What it can't, the owner's acceptance checks: acceptance already refuses an unsatisfiable rule and says why. So it proposes the rule naming the approver, and says what acceptance will check.
2. **The person's terms.** It answers in terms of what happens to their work, who does it, what must pass before a change lands, who approves, and which model runs. It names a charter, template, revision or kind only in a proposal or when asked. "What is the current setting" is best answered from, or as, the "How PROJECT works" document the same instructions keep.
3. **Approvals start from the person.** The natural first answer is the person: "you (free on the forge), or someone else?". That needs the person's forge identity in the chat's brief or from a tool, and a read of the project's members and roles. `read_permission` names the login with the id.
4. **Ways and questions read as settings.** Project ways and project questions read as settings, not `unknown`. Find why they answer `unknown`: a real read that fails for this agent's scope is a bug; if the agent asked with the wrong key, the tool's words should say how. In any case, the agent doesn't report a read it couldn't make as a fact about the settings.
5. **"Nothing has changed" once,** where a person could think something did, such as a proposal waiting for an owner. Not in every reply.

## What the plan must hold

- **Which of these go beyond the stock instructions and tools?** Say plainly in `plan.md`: words are one thing; a new read, a wider tool result, or the brief's new section is another, and 3 and perhaps 4 are the second. The user wants those named. Prefer the fewest new tools: a wider answer from one the agent already has, where that's right. Make them and name them; a protocol change you can't avoid is a `Question:`, with your recommendation.
- **A person's identity is what this agent's authority may read.** A member list is only for an agent that may configure this project. Say what's newly readable, and by whom.
- **Words are tested where they can be:** stock words by the tests that hold them (`temper-service/src/tests/stock*`), help pages by theirs; how a model takes them isn't provable below a walk, so test what's mechanical: the tool's answer, the brief's section, the refusal naming nothing of `ctl`. The stock's byte figures move: say by how much.
- **It must not break a walk's brief or a rung's clause** that reads the old words: `temper/docs/design/draft/configuring.execution.md` and its stories. Say which you moved.

## Size

Two to four tasks in temper; jig only if its surface must grow, which is a `Question:` first. fabrig/ui has priority: its files stay its own; `ui-*` plans may be on `temper-client-*`. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1). Formats change in place before v0.1.0.

## Walk

`walk = false`: a box walk of the configuring chat by fabrig/qa follows the landing, on the happy path.

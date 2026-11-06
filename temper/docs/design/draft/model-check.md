# Trying a model on the welcome

Draft, 2026-11-06, from a session with the user that day. On a box
whose only account was a Claude subscription, "I've signed in" passed and
every chat then hung, because it asked only `claude auth status`. The
user: "in the welcome journey it would be nice to validate that claude is
actually working via smith, after clicking 'Signed up' ... It could get a
nice UX like we do for Forgejo with validation steps being shown live."
The look request `model-check` draws the page
(`../client/journeys/model-check/`). This draft carries what the look
cannot.

## 1. In one page

- **One real turn:** once a model's account is connected, the welcome
  runs one short task through the path a chat takes, the worker, smith and
  the model (section 2).
- **Its steps, live,** as Forgejo's setting up shows them (section 3).
- **A failure says where and why,** and what to do. It keeps no one out
  (section 4).

## 2. The trial

- **When:** on the welcome's Model step and on `ConnectModel`, when an
  account becomes usable: after "I've signed in", a sign-in's end or a key
  added. "Try again" runs another. A trial's end is not stored as a fact
  about the account.
- **What runs:** one task of a stock charter of its own (no tools, one
  turn, a small output), asked for one word. It names no `llm`, so that
  the engine selects as it does for a new chat: a trial that named its
  model could pass while chats still hung.
- **Who pays:** the person, as for a chat. On a plan that is its included
  usage. On a key, one short turn at the key's price; the line "one short
  message" says so before it runs.
- **It is not a chat:** it is not in Chats, Needs you or Home's counts,
  and it writes nothing to a project's document.
- **Its end:** the first committed text ends it as answered. A run that
  ends otherwise, or a wait of more than a minute, ends it as failed.
- **Leaving** the page, or "‹ Back", cancels it.

## 3. The steps

Each step reads what the page already watches, in the terminal as lines:

| Step, for Claude | Done when |
|---|---|
| Signed in to your subscription | the deployment's document shows the source usable on a host |
| Claude Code starts on this machine | the trial's run has started, past "Starting" |
| Claude answers | the trial's first committed text |

- **For a sign-in or a key,** two steps: "Signed in to your ChatGPT plan"
  or "Key added", then "ChatGPT answers". The first ticks at once. A
  direct account has no program to start.
- **While it runs:** a spinner, "Trying your Claude subscription", and the
  list, its step under way marked.
- **Once answered:** the line "✓ Claude subscription · answered in 6 s",
  the steps folded under "Each step", and "Continue". The welcome does not
  move on by itself, so the person sees it worked.

## 4. When it fails

- **What it says:** "Claude didn't answer", the step that stopped, and why,
  in the person's terms: the run's ending, or why it waits
  (`RunWait` in jig's `domain/engine.md`, 7.1). Never a command for
  temper's own tools.
- **What it offers:** "Try again"; the act that answers the reason, such
  as "Sign in again"; and "Continue anyway", quiet. A person who knows
  better goes on, and Ready's Model line then says the last try failed.
- **The failure stays on the page.** Nothing is taken from the account: a
  source's own state is the host's check's (jig's `domain/accounts.md`,
  3.4).

## 5. Open questions

- **Settings:** whether the accounts page offers "Try it" on an account.
- **The playground** has no trial: its model is scripted.
- **Fake backends:** the scripted model answers, so the journeys walk the
  steps; the wait of a minute needs a world of its own.

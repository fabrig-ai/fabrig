# Trying a model on the welcome

Draft, 2026-11-06, from a session with the user that day. On a box
whose only account was a Claude subscription, "I've signed in" passed and
every chat then hung, because it asked only `claude auth status`. The
user: "in the welcome journey it would be nice to validate that claude is
actually working via smith, after clicking 'Signed up' ... It could get a
nice UX like we do for Forgejo with validation steps being shown live."
And of where it shows, once the first version
of this draft had put it in the page: "what I expect is no "inline" check
happens, it's a background check that shows up in the side pane as forgejo".
The look request `model-check-pane` draws the page
(`../client/journeys/model-check-pane/`). This draft carries what the look
cannot.

## 1. In one page

- **One real turn:** once a model's account is connected, one short task runs
  through the path a chat takes, the worker, smith and the model (section 2).
- **In the background:** the welcome moves on at once, and the check shows
  in the side pane, beside the forge's line and drawn as it is (section 3).
- **A failure says where and why,** and what to do, in the pane. It keeps no
  one out unless the account itself isn't usable (section 4).

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
  usage. On a key, one short turn at the key's price; the pane's line says
  "one short message" once it goes.
- **It is not a chat:** it is not in Chats, Needs you or Home's counts,
  and it writes nothing to a project's document.
- **Its end:** the first committed text ends it as answered. A run that
  ends otherwise, or a wait of more than a minute, ends it as failed.
- **Leaving** a page doesn't stop it: it belongs to the account, not to the
  page that started it. Removing the account cancels it.

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
- **Moving on:** "I've signed in", a sign-in's end or a key added goes on to
  Ready at once, as "Run on this machine" goes on to Model. Nothing on the way's
  page waits or changes first, so nothing flashes. Its host's check, today the
  way's page's, goes on behind it.
- **The pane:** the region named on the Model step, from the Model step on, with
  two lines, the forge's then the model's, each with the same parts. While it
  runs: the ring, ACCOUNT strong, "Trying: STEP" muted, the moving bar and "Each
  step" folded. Once answered: ticked, ACCOUNT, who it is, "answered in 6 s", and
  no "Each step". Below 1280 px it stays after the page's acts, with a hairline
  above it.
- **Ready's checklist goes,** since it would say the same two lines beside the
  pane. A line with nothing yet, "Forge none yet · Connect a forge" or
  Model's, moves to the pane as it is.
- **Ready's heading:** an account just declared counts as usable, for the
  heading and for Create, until its host's check says otherwise. So the page
  doesn't go from "Almost ready" to "Ready" 300 ms later.
- **Having left the welcome:** a try that answers says nothing more. On
  `ConnectModel` the account's row on the accounts page holds the same lines.

## 4. When it fails

- **What it says:** "✕", ACCOUNT, "Claude didn't answer", the step that stopped
  under "Each step", and why, in the person's terms: the run's ending, or why it
  waits (`RunWait` in jig's `domain/engine.md`, 7.1). Never a command for
  temper's own tools.
- **What it offers:** "Try again", and the act that answers the reason, quiet,
  such as "Sign in again", to the way's page.
- **What it stops:**
  - *The account isn't usable,* its host's check saying not signed in: "Almost
    ready", Create disabled, as for any model not usable.
  - *The account is usable and only the turn failed:* the page stays "Ready"
    and Create works. A person who knows better goes on.
- **Having left the welcome:** Home's line about the accounts says "Claude didn't
  answer when tried", with "Try again".
- **Nothing is taken from the account:** a source's own state is the host's
  check's (jig's `domain/accounts.md`, 3.4).

## 5. Open questions

- **Settings:** whether the accounts page offers "Try it" on an account.
- **The playground** has no trial: its model is scripted.
- **Fake backends:** the scripted model answers, so the journeys walk the
  steps; the wait of a minute needs a world of its own.

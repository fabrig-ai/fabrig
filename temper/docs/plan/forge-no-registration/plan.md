# forge-no-registration: an owned Forgejo that registers no one through temper

The request (`inputs/request.md`) asks for one task, in temper. `TEMPER_FORGE_NO_REGISTRATION` closes account-making on a forge `application.sign_in.forges` names, unset meaning today's bytes, and a real Forgejo 17.0.1 shows what the settings do.

## Parts and tasks

| Part | Task | Needs | Effort | Min | What it does |
|---|---|---|---|---:|---|
| `no-registration` | `no-registration` | | high | 90 | the settings, the variable, the goldens, a focused wiring test, and the process story's new leg |

The critical path is that task.

## What was decided

- **The entry is written here, not by the task:** `temper/docs/design/draft/dogfood-hacks.md`, 2.8 and its status line, since an implementer writes no draft, and the request and `temper/AGENTS.md` put the entry in this landing. `setup.md` and `sign-in.md` stay as they are; the status names the gap, and `domain/testing.md`, 2.6 with it.
- **`SETTINGS` follows the variable:** `temper forge sign-in` is the same binary, and the read sits in `forge_sign_in.rs` for both commands. The request's "otherwise say so" doesn't arise.
- **The step crate reads no environment:** `temper-forge-setup` takes `Option<FirstSignIn>`, which replaces `ini::text`'s `bool` and `prepare.rs`'s private `Registration`, and `crates/temper` reads the variable.
- **Empty counts as unset,** as `directories` treats its own variables; any other value is set, whose wrong guess (`=0`) fails closed.
- **A forge not named by `sign_in.forges`** gets today's text whatever the variable says: it has no sign-in to close.
- **The story grows, with no new test:** `domain/testing.md`, 2.6 has a variant grow an existing story. The leg restarts Forgejo after the first sign-in, so it also shows an account made by auto-registration still signing in after the switch, which is the deployment's own case. Deleting `sam` gives a member with no account, since the printed link signs in only that one member.
- **"A later prepare" is shown in the focused suite** on scripted programs, byte for byte, not on the real Forgejo: `Running::start` appends its log section to `app.ini`, and nothing about it needs a real Forgejo.

## What the run may stop on

- **What Forgejo does at 17.0.1 is unknown:** the request's own `Question:` if (1) or (3) doesn't hold; the task says what to name.
- **The binaries:** preparation holds Forgejo and its runner to the pin. In the login shell of the machine this plan was cut on, `TEMPER_FORGEJO` and `TEMPER_FORGEJO_RUNNER` name 17.0.0 and 13.2.0, against the pin's 17.0.1 and 13.2.1; builders weren't checked. A slot without the pinned ones ends in a `Question:`, since the task fetches nothing.
- **The tier's CPU:** the story's `cpu_test_s` is 60 s and its figure wasn't measured here; the task measures it before and after.
- **The queue's end checks don't run the process tier,** so the implementer's run is the only proof of (1), (2) and (3); the spec makes it acceptance.

## Overlaps

- `ui-goal-page/plan-filter-out` lists `temper/crates/**` and `temper/tests/**`, unmerged, which covers everything here by glob. No listed file of any other draft beside this one is in this task's `files`.
- `temper/docs/design/draft/dogfood-hacks.md`: a plan adding its own entry at the same time would meet this one at 2.8.
- fabrig/ui's files are untouched, as the request asks.

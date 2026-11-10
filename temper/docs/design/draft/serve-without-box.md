# The welcome is the box's

Draft, 2026-11-04, from a session with the user that day, whose
decisions it records. Which deployments show the welcome and its
playground, what `temper-box init` writes for each, and what a person
finds after signing in to one with no welcome. It keeps
`first-run-projects.md` as it stands: no project at the first start,
"Demo" for the playground and gone with its reset, "Default" made at
Ready.

## 1. In one page

- **`temper-box init` writes a bare deployment:** no welcome, no
  playground and no Forgejo on this machine; at its first start no
  forge, no model account and no project (`first-run-projects.md`,
  2). It ends saying to start it with `temper-box serve` (section 2).
- **`temper box` drives init:** its first run passes `--box`, with
  which init also writes the welcome's member, the owned Forgejo and
  the playground, as init writes the last two today (section 2).
- **`temper-box serve` reads the document, whoever wrote it:** the welcome
  shows where the document says so. A person may write that by hand or
  run a box's document from a systemd unit; outside `temper box` it is
  never the default (section 2).
- **With no welcome, signing in opens the inbox,** whatever is set up;
  setting up is the person's own (section 3).
- **Both clients learn it the same way:** from the engine, on the wire,
  the client's domain deciding from it, in the browser as in the
  terminal (section 4).
- **Nothing changes under the box:** the welcome, the playground with
  its "Demo" and its reset, and Ready's "Default" stay as they are.

## 2. The document says

- **The welcome's member:** `application.welcome`, `true` to show the
  welcome; absent, none (`../protocol/configuration/engine.md`).
- **The box's parts, one flag:** `temper-box init --box` writes them, and
  `temper box`'s first run passes it; a person may pass it too. They
  are the welcome's member; the owned Forgejo, its person and its
  port; and the playground, `application.demo` with its `demo/` files
  and its two ports (`../protocol/setup/box.md`, 1 and `../protocol/setup/init.md`, 2 and `../protocol/setup/playground.md`). The
  document keeps them as three members, so that splitting the flag
  later changes no format.
- **`temper-box init` alone** writes none of them, and its last line says
  to start the deployment with `temper-box serve`, with the flags init was
  given.
- **`temper-box serve` reads the document:** the welcome follows its
  member, whoever wrote it. It grows no flag, and the welcome does not
  follow `--box`, which only the box gives `temper internal serve`, for
  the pipe its owned Forgejo runs by (`../protocol/setup/box.md`, 1). A systemd unit
  running `temper box` shows the welcome as any box does; one running
  `temper-box serve` on a hand-written document shows it only if that
  document asks (`../protocol/setup/box.md`, 4).
- **The playground is the welcome's:** offered only where the document
  shows the welcome and has `application.demo`
  (`../protocol/setup/playground.md`). A server written from
  `docs/examples/owned.json` (`../../help/server.md`) shows neither.
- **No existing deployment is carried over:** there are none (the
  user, 2026-11-04).

## 3. With no welcome

- **Signing in opens the inbox,** the printed link's sign-in and every
  later one alike, with nothing of the welcome drawn first.
- **`/welcome` opens the inbox.**
- **With no project yet,** the chats page says so and offers neither
  "Set up temper" nor "Try the playground"; "+ New project" in the
  switcher makes one (`first-run-projects.md`, 7).
- **The Forge choices** offer GitHub and a Forgejo elsewhere, and
  Forgejo on this machine only where the document has one.
- **Setting up is the person's:** the Forges and Accounts pages,
  `temper ctl` or the document, as they are.

## 4. How the clients know

- **On the wire, read by the domain:** a signed-in client learns
  whether the document shows the welcome from a record both shells read
  before the welcome decides; beside the playground's answer, which
  the welcome already waits for (`../protocol/setup/playground.md`), is one
  place for it. The browser's page carries nothing new.
- **The browser's sign-in still lands on `/welcome`:** the welcome's
  decision sends the person of a deployment with no welcome on to the
  inbox, as it sends a whole first run's on to the chats page
  (`../client/README.md`, 3.4 and `../client/pages/README.md`).
- **The terminal,** which opens at the inbox, takes the same fact for
  the pages of section 3.
- **Forgejo on this machine** is offered from the forges' rows: with
  no owned forge among them, none, rather than the box assumed
  (`../client/pages/README.md`).

## 5. Its order

1. Section 2: the welcome's member, init's bare document and the
   box's parts.
2. Section 4: the fact on the wire, and the welcome's decision reading
   it.
3. Section 3: the pages with no welcome, and the playground's offer.

## 6. Later

- **The deployment's own UX:** what `temper-box init` alone writes and
  says beyond section 2, and how a server is set up, are designed
  later. For now init alone is the cheapest bare deployment, and the
  work goes to the box's mechanics (the user, 2026-11-04).
- **Flags of their own** (`--welcome`, `--playground`) when a
  deployment wants some of the box's parts and not others.

No open question at its writing.

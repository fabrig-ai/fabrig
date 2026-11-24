# Workers

2026-11-16. The Deployment frame's Workers tab, `Builders { project }`
(`settings.md`, 1): the workers as settings (jig's
`domain/engine/workers.md`; `../../draft/config.md`, 6.7), each with
its places and whether it is up, and for an owner the acts that add,
change and remove one. The word on the page is "worker"; the code's
stays host, and the page's `Builders`.

## 1. The tab

- **It reads jig's `deployment` document,** watched while the tab is
  open by a person with a role in the deployment, as `Accounts` watches
  it for an owner, and draws its `worker_settings` (jig's
  `domain/engine/workers.md`, 6). Until the document is read, the title
  "Workers" alone, claiming nothing; a cut document says so, as
  `Accounts` does. With no role in the deployment, one line, "Workers
  are seen by the deployment’s owners.", and no section.
- **A section, "Workers · N",** N its workers not withdrawn; then a line
  per worker, in number order, as Repositories' are: its name; in
  `small`, "4 places · 1 run" ("1 place", "no runs"), its runs those of
  the connected host of its name (`Deployment.hosts`), then " · in the
  box" for the worker with no digest, else " · digest HEX…", HEX the
  digest's first 8 hexadecimal digits; in the line's `.when`, "Draining"
  while its host drains, else "Up since TIME" while it is linked, else
  "Down since TIME", else "Down", TIME as the page's times are
  (`../view.md`). With none, `.empty`, "No worker is added yet.".
- **An owner's acts on a line:** a field whose accessible name is
  "Places for WORKER", holding its places, with "Change", off until the
  field holds another count from 1; and "Remove", but on the worker with
  no digest. Each press reads the settings' newest revision and asks
  jig's `ChangeSettings` with one item, `Worker(number)`, as the worker
  tools do (`../ctl/workers.md`, 2), the line keeping its state until
  the answer. A refusal shows under the line in the words of a change's
  refusal; *linked* reads "WORKER is up: stop it first. Its runs end
  interrupted." Anyone else has the line alone.
- **"Add a worker",** a section, for an owner: a field "The worker’s
  name", drawn by its placeholder "w1"; a field "Places", 4 in it; a
  field "Its secret’s digest", drawn by its placeholder "64 hexadecimal
  digits"; "Add", off until the three hold jig's forms (jig's
  `domain/engine/workers.md`, 2): a name, a count from 1, 64 lowercase
  hexadecimal digits. A name a live worker holds says "WORKER is already
  a worker."; a withdrawn one's comes back at its number. Under them, in
  `small`: "Make its secret on the worker’s machine; temper help server
  says how. A worker that’s running links once it’s added." A press asks
  `ChangeSettings` as above, and the fields empty on its answer; a
  refusal shows under the form.
- **The last line,** in `.facts`: "Each agent’s role names which workers
  it runs on. Ask temper to change it.", as before.
- **In the terminal,** the same sections as lines, and the fields and
  presses as the terminal has them on How work runs (`settings.md`, 2).

## 2. Its tests

- **Focused,** in the page's view crates, as the Models tab's are: a
  document with a worker up, one down since a loss, one never linked,
  one withdrawn and the box's, drawn as 1 says, for an owner and for a
  member; a press's `ChangeSettings` item for a change of places, a
  removal and an add over a free and a withdrawn number; a *linked*
  refusal's words under its line.
- **The client world's story** (`tests/client`): an owner adds a worker on
  the tab, changes its places by the field's submit, reads the changed line
  again on a fresh load at the tab's address, and removes it. Each act sends
  one `ChangeSettings` against the revision it read, and the fresh load sends
  none and reads the same revision. The tab has no browser story: it holds
  nothing a browser alone shows, a field's Enter being its submit binding, the
  view's (`../web/testing.md`, 6).

# look-round-25: How work runs as the final mockup, and the ways page's last misses

The request: `inputs/request.md`, following P3a (`ui-p3a-how`), which drew an earlier `how.html`. `notes/common.md` holds the rules every task follows; `notes/contracts.md` the shapes that cross tasks.

## Parts and tasks

| Part | Task | Request | Needs | What it makes run | Minutes |
|---|---|---|---|---|---|
| `stops` | `tasks/path-stops.md` | 1, 2 | | the engine's path as amended: "conflict" and "resolve" at the queue, "update" in the cards, "catch up", `needed` and `over`; "never merge"; the wire, the service, `see ways` and every reader | 220 |
| `look` | `tasks/ways-misses.md` | 3 | | the open card at 1000 and on a phone; four tabs two by two under 760; the drawing and legend shared with How work runs | 80 |
| `roles` | `tasks/roles-view.md` | 2 | `path-stops` | the view `roles`: a role card per agent the project runs, in order; a reply draws it | 130 |
| `fade` | `tasks/fade.md` | 3 | | skein: a scene's mark may have its stroke painted by a linear fade, its stops coloured by their tokens; the browser paints it as the SVG's own gradient | 80 |
| `marks` | `tasks/path-marks.md` | 1, 3 | `ways-misses`, `fade` | the drawing's stops outlined when needed and their words over, the agent's star in ink, code grey, each rise a fade, the legend's agent when needed | 170 |
| `how` | `tasks/how-drawing.md` | 1 | `path-stops`, `path-marks` | How work runs draws its path as Ask temper's ways do; the lanes and their blue go; both pages and a reply draw the wire's `needed` and `over` | 150 |
| `roles` | `tasks/roles-page.md` | 2 | `how-drawing`, `roles-view` | Roles on How work runs, from the view `roles` | 150 |

Still being written: the forge reading a repository's CI workflows (`workflows-read`) and CI's card naming them (`ci-card`).

**Critical path, as it stands:** `path-stops` (220) → `how-drawing` (150) → `roles-page` (150), about nine and a half hours with `path-marks` (after `ways-misses` and `fade`, about 250 minutes from the start) ending before `path-stops` is far past; `roles-view` (130) runs beside `how-drawing`.

**Who waits on whom, and why:** the tasks that change How work runs' page (`path-stops`, by the stops its table shows; `how-drawing`; `roles-page`) each reprint the journey `chat-draft`, so they run one after another, never side by side. `path-marks` changes the drawing while How work runs still draws its table, so it reprints nothing and runs beside `path-stops`, its callers passing no marks until `how-drawing` passes the wire's. `roles-view` crosses the wire after `path-stops` does, on the same schema.

## What was decided

- **The path's change is the engine's,** and clients draw what it gives (`stock-ways.md`, 4, *One derivation*): `Step::Update` goes and `Conflict` stands where it stood, before "resolve", after the approvals; "catch up" ends a goal's segment where the project names a merge job; every stop says whether it is made only when needed and carries the small words over its own, so that no client guesses them (`forge.md`, 8.13, *Drawn apart*).
- **"Never merge"** is `can`'s, and a charter with none of the four tool families says what its run holds: Ask temper's by its settings family, the status agent's "read the facts in its task" (`forge.md`, 8.13, *What an agent can*). One task with the path, since their readers are the same tests.
- **Roles come from a view, `roles`,** not from the `ways` document: that document is near its decision's bound (the worst small one measured about 59,000 of 65,536 bytes after `look-round-24`), and a role is a charter's card, which the `View` read already answers for `charter N`. How work runs reads it as the role page reads `charter N`; a reply may draw it. Each role is named and placed by code from where its charter runs (`forge.md`, 8.13, *The project's roles*); what it does, past when, waits for P3e's words saved with a charter.
- **How work runs takes Ask temper's drawing,** as `how.html` and its README have it ("How work runs draws the path as Ask temper's ways do"), in place of P3a's table in lanes, which an earlier `how.html` drew: one drawing, its legend and its card, shared in `pages/ways.rs`; the lanes' blue and slate tokens go, so no blue is left.
- **Who acts by mark** (`surfaces.md`, 13): an agent's star and the rise to it in ink, a person's ring and its rise the signal's, code's square grey, on Ask temper's ways and replies too. A stop made only when needed is outlined; its small words stand over its own.
- **The line's gradient is skein's fade,** a generic mechanism before its consumer (`AGENTS.md`, 1): a scene's mark may have its stroke painted by a linear fade, its stops coloured by tokens, so that a scene still names no colour (skein's `ui.md`, 11.2). Each rise fades as the look request's line does, from its mark's colour at the marks' row through a fainter tone at the dotted line to the line's colour: a person's from the signal at .8 and .28, an agent's from ink at .3 and .1, the whole story's lighter tone for an agent.
- **The card positions:** on a phone the narrow path is drawn 360 wide and scaled under 360, as the request's picture is; at 1000, `ways-misses` works out both positions by arithmetic from the request's page and repairs what is past half a pixel. The render check's own findings were not in the request's words, so the task measures them from the pages, not a browser.
- **Trailers:** `fade` and `path-marks`, which repair the look request `ask-temper-ways`' line, carry its trailer (`look-requests.md`, 3); `ways-misses`, released first, does not.
- **A stored format, a protocol or a public interface:** the client wire's `PathStep`, `Stop` and `View` change, with no family version and no compatibility, as the request allows.

## Overlaps with drafts beside this one

- **The client wire's schema, generated code and goldens** are also `ui-reaches`' (`ways-away`, `decide-forge`, `keep-asking`): `path-stops` changes `PathStep` and `Stop` alone. Unavoidable: the path crosses the wire.
- **`temper-client-view`'s `words.rs`, `look.rs` and `temper.css`** are every UI plan's registries: this plan keeps its words' keys (4930, 4931, then 5100 to 5149), its look's slots and its rules' places as `notes/common.md` says.
- **`temper/docs/design/client/README.status.md` and `browser.status.md`:** this plan's entries stand after the ways' paragraphs, not at the files' ends, where `ui-p1-rail`, `ui-p2-home` and `ui-p4a-team` add theirs.
- **The journey `chat-draft`** (How work runs in its `01.html` and `02.html`) is P1's `regenerate`'s too: regenerated on a catch-up, never hand-merged.

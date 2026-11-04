# fetch-progress-1: the plan

From `inputs/request.md`: while an owned Forgejo's set up fetches its binaries, the person sees how far it has gone, in words in both shells, and in the browser a bar that fills.

The plan is released in pieces. This is the first; the temper parts follow, and this file with them.

## Parts and tasks

| Part | Task | Needs | Minutes | What it does |
|---|---|---|---|---|
| progress-value | `progress-value` | | 55 | skein-ui's fraction, a progress's own runs, the web's attributes, the stylesheet's 101 rules |

## Critical path

`progress-value` (55), then the temper tasks, not yet cut.

## What the cut found

- **`aria-valuetext` never reaches a page.** `skein-ui-web/src/mapping.rs` writes it from the progress node's own runs, and `Builder::run` refuses every role but text, so a built progress has no such runs. A text child does not stand in: `role="progressbar"` presents no children to assistive technology. The draft says the mapping writes it "today"; the code holds the line, and no tree reaches it.

## What the cut decided

- **A progress's text is runs of its own** (`skein/docs/design/ui.md`, 3.4 and 5): `Builder::run` takes a progress as well as a text node, so "its text as `aria-valuetext`" (`skein/docs/design/ui-web.md`, section 6) comes true, and `close` can assert that a fraction has its words. It widens a public builder call by one role, in the direction the draft already asks for, so it is decided here and not asked. A text child stays allowed with no fraction, so no temper fixture moves.

## skein

`skein-ui`, `skein-ui-web`, `skein-ui-bundle`, and `skein-ui-text`'s fixture. **skein-ui isn't on hearth's skein surface:** hearth builds nothing of this and nothing of hearth changes.

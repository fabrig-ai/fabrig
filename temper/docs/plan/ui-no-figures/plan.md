# ui-no-figures: the plan

The reference's section 6, "In numbers", goes, at the user's word of 2026-11-06: `temper/docs/plan/ui-no-figures/inputs/request.md`. Figures live on the site or the public-facing version.

## The parts and the task

| Part | Task | Needs | Effort, minutes | What it gives |
|---|---|---|---|---|
| `figures` | `no-figures` | | medium, 30 | the page, the view, the look and their tests without the section; a gallery check at 1440 and 420 |

The critical path is that one task. One slice, no walk: the request asks a browser check, and a gallery state is enough, since `samples.rs` opens the pane by the person's own act.

## The design, moved by this cut

- `draft/own-way.md`: its one-page line, the entry in section 3's list, section 5 and section 7's question go. Section 5's number stays unused; 6 and 7 keep theirs (`DESIGN.md`, 4).
- `draft/own-way.status.md`: says where figures live. Its section 5 entries and its **Open** go. "Several machines" had hung on section 5's reason, so it moves to a section 4 entry, with its own reason.
- `protocol/documentation.md`, section 7: the figures' open question goes, the only citation of `own-way.md`, 5.
- `client/README.md`, 3.5, `client/browser.md`, 2.5: the figures go; `client/README.status.md` drops them from its departure.
- Two **Not yet** entries say the code lags, in `client/README.status.md` and `client/browser.status.md`. The task removes them.

## The decisions

- **`closing` stays.** Section 5's muted note still needs it. Only `numbers` and `figures` go.
- **Token 455 stays held,** as `temper-reserved-455`: `look.rs`'s `NAMES` is positional, and that is how it holds every freed token.
- **The way's no-address test stays,** renamed: the rule is `AGENTS.md`, section 2's, and outlives the figures.
- **`configuring.txt` needs nothing:** it names the page and no figures. The look's account was `browser.md`, 2.5, moved above.
- **Nothing to regenerate:** no print, golden or generated help holds the section (`git grep`), and a journey's print links `temper.css`. The task regenerates only what a test names stale.
- **No stored format, protocol or public interface changes,** so nothing for temper/forge.
- **`own-way.md`'s preamble and section 7's "When"** still say no plan takes the draft up, which its status overrides. The request doesn't ask for them, so they stay.

## Overlaps not avoided

Each is a few lines in a long file, and fab's trunk takes them at the merge.

- `temper/crates/temper-client-shell-web/temper.css`: `bundle-loading/page-temper` and `chat-reading-c/reading-browser` change other rules.
- `temper/crates/temper-client-view/src/look.rs`: `ui-pause-close/close-words` and `ui-repo-link/repo-link`, merged into their trunks, add names elsewhere in the table.
- `temper/crates/temper-client-view/src/tests/look.rs`: `ui-pause-close/close-words`.
- `temper/tools/gallery/src/tests.rs`: `ui-pause-close`, `ui-repo-link`, `chat-reading-c/reading-close` and `long-words/composer-view` own the gallery. Two lines here, in the `configure-reference` rows.
- This cut's design files: `client/README.md`, `README.status.md`, `browser.md`, `browser.status.md` and `STATUS.md` are changing under `chat-reading-c/reading-close`, `model-check/trial-close`, `long-words` and `bundle-loading/page-temper`, each in other paragraphs. This plan leaves `STATUS.md` alone.

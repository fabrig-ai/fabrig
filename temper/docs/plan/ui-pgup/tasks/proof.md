+++
id = "proof"
part = "proof"
kind = "docs"
needs = ["page-follow", "live-kept", "ending-keeps"]
files = [
  "temper/docs/plan/ui-pgup/driver/**",
  "temper/docs/plan/ui-pgup/proof.md",
  "temper/docs/plan/ui-pgup/proof/**",
]
effort = "high"
minutes = 60
+++

# The same steps on the fixed code

## Goal

Run `reproduce`'s driver on the trunk with every fix merged, at 1440 and at 420, and report each step beside what it did before. This report is the plan's proof, in place of a browser story. If a step still fails, find its cause and fix it here, in the lowest tree that shows it, with a focused test. Then this plan has done what its request asked.

Acceptance:

- `temper/docs/plan/ui-pgup/proof.md` has, for every step of `temper/docs/plan/ui-pgup/reproduced.md` at both widths, the offsets, moved, pill, focus, words and caret, beside what `reproduced.md` said, and **holds** for each. The race's counts are there too: no press lost, at either width. A step that can't be made to hold is **fails**, with its cause and why it stayed.
- Two screenshots in `temper/docs/plan/ui-pgup/proof/`, `holding-1440.png` and `holding-420.png`, each of the view held up the conversation while words arrive below it, with "Latest ↓" showing. `proof.md` gives each one's offsets and `scrollHeight`, taken before and after, so the words' arriving shows in the numbers.
- Any fix has its focused test, passing, and no browser story is added.

## Inputs

- `temper/docs/plan/ui-pgup/notes/common.md`.
- `temper/docs/plan/ui-pgup/inputs/request.md`, *What a person does, and must see*.
- `temper/docs/plan/ui-pgup/reproduced.md`, and the driver beside it.
- `temper/docs/design/draft/chat-reading.md`, section 5; `skein/docs/design/ui-web.md`, section 7; `jig/docs/design/client/domain/client.md`, section 10, *Live, then committed*.

## Owns and may change

- **Owns:** the files in this spec's header.
- **May change, how:** where a step still fails, the code that causes it and its focused test, within `skein/crates/skein-ui-web/**`, `jig/crates/jig-client/**`, `temper/crates/temper-client-domain/**`, `temper/crates/temper-client-view/**` and `temper/crates/temper-client-shell-web/temper.css`. New `temper.css` rules go under `/* ==== ui: chats ==== */` or, for 420 alone, `/* ==== ui: phone ==== */`. A design line a fix makes false is yours to correct, in `skein/docs/design/ui-web.md` or `jig/docs/design/client/domain/client.md`, and a departure from the implementer's text.
- **Options:**
  - If the driver needs a change to run on the fixed code, make the smallest one and say what and why: the two reports must still compare.
  - A failure you can't show in a focused test but only in the driver: fix it, say so, and let the driver's report be its proof.
  - A fix in skein stays generic, in skein's own tests. A fix in jig stays in jig's terms.
  - If the fix a step needs goes beyond these files, stop on that step, report it as **fails** with its cause under `## Left`, and finish the rest.
  - A tabbed-to "Latest ↓" hiding as it is pressed, which sends the focus to the page's body, is outside the request, which has the focus in the composer throughout. Report it if you see it, and leave it.

## Steps

1. Build as `reproduce` did: `heavy cargo build -p temper`, and the bundle by the cargo lines of `temper/scripts/bundle.sh` with `--loopback`'s feature, each through heavy.
2. Run the driver, whole, at both widths, on a fresh box each. Say which commit you ran it on: this branch's base holds all three fixes.
3. Check the fixes against `reproduced.md`, and read it for what it left open:
   - **The fast race:** no press lost, for PgUp and for PgDn reaching the foot.
   - **A turn's ending:** moved 0, and the sampler's smallest `scrollHeight` across the ending shows no collapse.
   - **"Latest ↓":** focus and caret stay in the composer after a pointer's press.
   - **Key and wheel:** they now agree.
   - **Sending:** still goes to the foot.
4. Where a step fails, read why before anything reruns. Fix it as the options above say, with its test, and run the driver again at that step and then whole.
5. Take the two screenshots while words arrive, and keep each small: a viewport's PNG, no full-page capture.
6. Write `proof.md`: a table per width, as in `reproduced.md`, with its before beside each after; the race's counts, before and after; the screenshots, each with its numbers; and each fix this task made.
7. Check that the status files hold nothing of this plan still open: `skein/docs/design/ui-web.status.md` and `jig/docs/design/client/domain/client.status.md` carry no **Not yet** from it. If a step failed and stayed, its gap is a **Not yet** line in the status file of the document that promises it, which you add.
8. If you changed code, once: `cargo fmt`; `python3 -I -B <tree>/scripts/old_forms.py` for each tree you touched; `heavy cargo clippy -p PKG --all-targets -- -D warnings` naming each package; `heavy cargo nextest run -p PKG` for each and its direct dependents. Always: `python3 -I -B tools/citations/citations.py --base origin/main`. Read `proof.md`, the driver and the screenshots yourself: no path or address of a machine, and nothing private. Commit and push.

## Report

The first line says whether all steps hold at both widths. Then the commit it ran on, the Chromium version, the race's counts before and after, any fix you made and its test, and your minutes. Anything left goes under `## Left`, with its cause.

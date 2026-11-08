# limits-raise: common rules

Every task of this plan follows these, beside fab's implementer text, `AGENTS.md` and `temper/AGENTS.md`.

- **The request:** `temper/docs/plan/limits-raise/inputs/request.md`. Four figures rise by half, for now, by the user's word of 11-08; no other figure in `tools/tiers/budgets.toml` changes, and no table or key is added to it or removed from it.
- **The new lines are the cut's, word for word.** The task's spec gives every replaced line and its replacement. Copy them as given: the wording ("raised by half by the user's word of 11-08, for now, to be revisited") is the request's, and the cut chose each place. A line the spec does not name stays as it is, even if it shows one of the old figures: the cut read it as a dated measurement, a historical account or the strategy's own rule, and `temper/docs/plan/limits-raise/plan.md` lists those.
- **A stated departure:** the implementer text says no `docs/design/` file is yours. This plan's task edits the design and status lines its spec names, and only those, with the spec's text: the request asks that each document change with its figure, and one commit keeps the file and its statements in step.
- **No slow tier runs.** Never `tools/tiers/run` (it builds and runs a tier; `--verify-all` builds too), a `<tier>-<tree>` profile or `scripts/browser.sh`. No walk, render check or throwaway driver. The raised budgets are read after landing.
- **No figure in prose is invented.** Each new figure is in the spec: 180, 217.5, 127.5, 540, and the browser tier's remainder 82.650100 (127.5 less 44.849900). Times come from `date -u`.
- **Nothing of the lab** that develops fabrig is named (`AGENTS.md`, 1; `tests/boundary/tests/outward.rs`).
- **Before the push:** `python3 -I -B temper/scripts/old_forms.py`; after the commit, `python3 -I -B tools/citations/citations.py --base origin/main` (it reads the commit, not the working tree) and `heavy cargo nextest run -p fabrig-boundary --test outward` (it reads git's tracked files). No `cargo fmt` or clippy: no Rust changes.

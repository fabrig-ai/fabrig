# temper-release-macos: common rules

Every task of this plan follows these, beside fab's implementer text, `AGENTS.md` and `temper/AGENTS.md`.

- **The request:** `temper/docs/plan/temper-release-macos/inputs/request.md`. Its "Not in this plan" holds: no change to `.github/workflows/macos.yml`, to `Cargo.toml`'s release profile, or to any code; no signing, no release, no tag, no secret.
- **Nothing here runs the workflow.** It runs on GitHub's runner, and its first dispatch is after landing, not part of this plan. No build or test of the workflow's own: the review is by reading (the request's "Done when"), against the task's checklist.
- **No lookups beyond the task's.** The pins are in the spec, each checked by the cut: take them as given. No request to GitHub or any other service, with or without a token.
- **The file names nothing of the lab** that develops fabrig: no machine, tool or session (`AGENTS.md`, 1; `tests/boundary/tests/outward.rs` lists the names). Say "the host that serves fabrig.ai" or "the site's host", never a machine's name.
- **Status:** the task deletes its own **Not yet** entry, named in its spec, from `temper/docs/design/release.status.md`: a stated departure from the implementer text's "Not yours: any `docs/design/`", for that entry alone.
- **Before the push:** `python3 -I -B temper/scripts/old_forms.py`; `python3 -I -B tools/citations/citations.py --base origin/main`; after the commit, `heavy cargo nextest run -p fabrig-boundary --test outward`, which reads git's tracked files and so the new one.

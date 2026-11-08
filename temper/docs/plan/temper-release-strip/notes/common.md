# temper-release-strip: common rules

Every task of this plan follows these, beside fab's implementer text, `AGENTS.md` and `temper/AGENTS.md`.

- **The request:** `temper/docs/plan/temper-release-strip/inputs/request-strip.md`. Its "Not in this plan" holds: no gzip and no `.sha256` (the site's host makes them); no change to Linux's file, to `Cargo.toml`'s release profile, to `.github/workflows/macos.yml` or to any code; nothing of the later set under `/temper/v{MAJOR}/`. No Developer ID, no notarization, no secret.
- **Nothing here runs the workflow.** It runs on GitHub's runner, and its next dispatch, by ops after landing, is its check. No build or test of the workflow's own: the review is by reading, against the task's checklist.
- **No lookups beyond the task's.** The commands are in the spec: take them as given. No request to GitHub or any other service, with or without a token.
- **The file names nothing of the lab** that develops fabrig: no machine, tool or session (`AGENTS.md`, 1; `tests/boundary/tests/outward.rs` lists the names). Say "the site's host", never a machine's name.
- **Status:** the task deletes its own **Not yet** entry, named in its spec, from `temper/docs/design/release.status.md`: a stated departure from the implementer text's "Not yours: any `docs/design/`", for that entry alone.
- **Before the push:** `python3 -I -B temper/scripts/old_forms.py`; `python3 -I -B tools/citations/citations.py --base origin/main`; after the commit, `heavy cargo nextest run -p fabrig-boundary --test outward`, which reads git's tracked files and so the changed one.

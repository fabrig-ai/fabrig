# The walk kit, GitHub: what changes from kit.md when the forge is GitHub, connected

Edition 2 (11-01 ~10:0xZ): the connect is `--read-only` and accepts only fabrig-ai's installation
177453345 (edition 1's plain connect let temper's reconcile uninstall the App's installations,
walk qlg-0-2; repaired on main 69fafec04, the App reinstalled by the user).

Edition 3 (11-01 ~10:3xZ; rungs A onward, rung 0 reads no CI): the person's token now reads Actions
runs, jobs and logs and issues, so CI is read on GitHub's side as well (the CI bullet below).

Edition 4 (11-01 ~13:2xZ; B1 and B2, which no GitHub walk had reached): the person's gate goes in
as one commit through the git data API (the contents API writes one file per commit); the token
holds Workflows write for it; commit statuses answer 403 to the token, so CI is read through the
Actions API alone.

Read `kit.md` first; this file replaces its "The machine" host, adds the GitHub hands, and maps
each rung brief's owned-Forgejo steps onto GitHub, connected (`temper/docs/design/draft/github.md`
6.5: an App the person made, its id and key given to temper; `temper help github`). The model stays
ChatGPT, as kit.md says (qa.md 3: the GitHub rung's model is "as the rung below"). Everything else
in kit.md holds. GitHub *managed* (temper's App made from its manifest) needs a person's clicks in
a browser on the engine's machine (github.md 6.1, 6.5) and is not walked unattended.

## The machine

- **The builder your task names** (`HOST`), as kit.md says; GitHub walks run one box at a time,
  since they share the repository kept for QA. Run nothing heavy; read GitHub at a person's pace.

## The GitHub hands

- **Lent to `W/github/`** (a 0700 directory, by the owner before your session): `app.json` (the
  probe App `temper-probe-d0fc06`, its id, permissions and installations: no secret, readable),
  `README.md` (readable), `app.pem` (the App's private key), and `person-fabrig-ai.token` (the
  person's fine-grained token, admin on `fabrig-ai/temper-probe`). Never read, print or copy
  `app.pem` or any `*.token`; check them by `stat` alone. Ignore `person-freeekanayaka.token`.
- **The repository kept for QA:** `fabrig-ai/temper-probe` (the App is installed there). It is the
  one GitHub repository you may touch; never any other `fabrig-ai` repository.
- **The person's header file:** make `W/gh.header`, 0600, holding the one line `Authorization:
  Bearer <the token>`, built from the token file without printing it (e.g. `{ printf
  'Authorization: Bearer '; cat W/github/person-fabrig-ai.token; } > W/gh.header` under `umask
  077`), and use it by its path: `curl -fsS -H @W/gh.header -H 'Accept: application/vnd.github+json'
  https://api.github.com/...` (qa.md 5, GitHub's reads; the person's writes, contents API, the same
  way). Remove `W/gh.header` at the end.

## The repository's state, before and after

- **Before your rung's steps,** as the person: close every open pull request and delete every
  branch but `main` left by earlier runs (`temper/*` and others), and close open issues temper made
  (its goals' tracking issues). Then set `main`'s files for your rung, one contents commit each as
  the brief says:
  - rung 0: leave `main` as it is (it holds the probe's `.github/workflows/ci.yml`: job `ci`, 45 s,
    failing when the head commit's message holds `FAIL`); read its files and say them.
  - rungs A, B0, B1, B2 (the playground's copy): `README.md` = `# widget\n` and `src/main.rs` =
    the playground's widget exactly as temper's source seeds it (`PLAYGROUND_MAIN`,
    temper/crates/temper-forge-setup/src/lib.rs in REPO), no other files but the workflow; delete
    any other file. Message "Reset to the playground's widget".
- **After the box stops,** as the person: close open pull requests and issues your run left,
  delete its branches, and on B1/B2 put back the probe's own `.github/workflows/ci.yml` (save it
  first) and remove the gate's `CONTRIBUTING.md` and `changes/`, one commit "Restore the probe's
  CI" through the git data API. Say what you cleaned.

## The rung's steps on GitHub

- **"A box … its owned Forgejo set up through ctl"** (rung 0 and every rung's box) reads: the box
  started as kit.md says, then GitHub connected and its repository adopted as `temper help github`
  walks it: `forge connect github-app APP W/github/app.pem --read-only --key KEY` (APP from
  `app.json`'s `id`; `--read-only` because the probe App is shared: temper then leaves its hook and
  installations as they are), `forge list --until settled`, `forge accept FORGE INSTALLATION` for
  the installation on `fabrig-ai` (177453345, as `forge list` names it; never accept another
  account's installation), `forge repositories FORGE
  --installation INSTALLATION`, then `repository adopt --project PROJECT --forge FORGE --owner
  fabrig-ai --name temper-probe --landing main --key KEY` as the project's home (no `--no-home`;
  the box needs no Forgejo of its own). Rung 0's checks read it on GitHub: the repository, its
  files and `main`'s head, and the project's home in `project show`.
- **"A copy of playground/widget on the owned Forgejo"** reads: `fabrig-ai/temper-probe` reset to
  the playground's widget (above), adopted as the project's home.
- **CI** is GitHub Actions on GitHub's runners. Read it with the person's token as qa.md 5's GitHub
  table says through the Actions API: runs at a head (`/actions/runs?head_sha=SHA`), a run's jobs
  and a job's log (`curl -L`), the pull request (`merged`, its commits and
  files), `main`'s head after a merge. Check runs and commit statuses (`/commits/SHA/check-runs`,
  `/commits/SHA/status`) answer 403 to this fine-grained token: use the Actions runs instead.
  Issues read and write work, so close temper's tracking issues a run leaves open at its end.
- **The person's gate (B1, B2):** the appendix below in place of the brief's Forgejo appendix, one
  commit with the brief's message through the git data API (blobs, a tree on `main`'s tree, a
  commit, then `PATCH /git/refs/heads/main`); a write under `.github/workflows/` needs the token's
  Workflows permission; its push run passing (`ci` green,
  `changes` skipped).
- Requests and answers name the repository as the brief has them ("widget"); add nothing.

## Appendix: the person's gate on GitHub (B1, B2; edition 2's rule)

`.github/workflows/ci.yml`:

```yaml
on:
  push:
    branches: [main]
  pull_request:
jobs:
  ci:
    runs-on: ubuntu-latest
    steps:
      - run: echo "the probe's starter check passed"
  changes:
    if: ${{ github.event_name == 'pull_request' }}
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          ref: ${{ github.event.pull_request.head.sha }}
      - name: Each pull request adds changes/<its number>.md, naming when it was opened
        run: |
          n='${{ github.event.pull_request.number }}'
          opened='${{ github.event.pull_request.created_at }}'
          echo "pull_request.number=$n opened=$opened"
          test -n "$opened" || { echo 'No pull request time in the event'; exit 2; }
          f="changes/$n.md"
          test -f "$f" && grep -qxF "opened $opened" "$f" || { echo "Missing changes/$n.md with the line: opened $opened"; exit 1; }
```

`CONTRIBUTING.md`: as the brief's appendix has it, exactly.

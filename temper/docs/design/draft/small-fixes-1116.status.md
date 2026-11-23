# small-fixes-1116: status

2026-11-17. Where the design lags or departs from [small-fixes-1116.md](small-fixes-1116.md).

The design carries this draft: its first part in `../../../../tools/pre-pr/design.md`, 1, 6, 7 and 9 (the failures file at `target/temper/pre-pr-failures.json` under the checkout's root, the failures line naming that path from the root, smith's `.git` guard kept); its second in jig's `domain/people/README.md`, 7, and `../domain/agent/tools-change.md`, 1, *A person's report has a floor* (the batch check's refusal as `Refusal::Contract`, the schema's line and the refusal's words naming the bound); its third in `../../../../tools/pre-pr/design.md`, 1, 3, 4, 5, 8 and 9 (a prose-only delivery: the outward text check and citations, no cargo command and no hold) and `../../../../tools/outward/design.md` (the list both scans read, the text check, its verdict and the test that the two agree). Where the code lags is those documents' status.

- **Design lags:** none.
- **Design departs:** 3: a prose-only delivery is one of Markdown under a `docs/` directory alone, not of any Markdown that no registered reader declares: `fabrig-boundary`'s `cli_names` test scans every tracked file outside a `docs/` directory for an environment variable's name, Markdown included (a package's README, `AGENTS.md`), and a glob of `.config/gate/read-by-tests.txt` cannot name that set, so a delivery that changes Markdown outside `docs/` keeps the boundary packages' tests (`../../../../tools/pre-pr/design.md`, 3, *Prose only*). Status files, drafts and design documents are all under `docs/`, so the deliveries the draft names build nothing. The goal's designer, 2026-11-17, keeping the draft's intent.
- **Open:** none.

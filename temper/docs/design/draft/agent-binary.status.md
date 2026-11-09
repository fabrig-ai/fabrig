# agent-binary: status

2026-11-09. Where the design lags or departs from [agent-binary.md](agent-binary.md).

The plan landed whole on main, PR #78, merge `fdc26f71c6`, on
2026-11-09: all six ranked items landed, none dropped. No design lag or
open decision remains within this plan; the four accepted departures
below remain.

**Draft fate:** its vision is folded into `smith/docs/design/shell.md`,
2.6, `temper/docs/design/protocol/README.md`, 4.1 and 4.2, and
`temper/docs/design/protocol/setup.md`, 6.6 and section 11. Removal of the
completed draft is proposed under `DESIGN.md`, section 4; its user's text
is retained read-only, not deleted or marked as an active plan.

**Later:** the draft's "Not in this plan" stays outside this close-out:
M3 names `temper-agent` as the worker's program, M5 integrates the box,
M6 publishes the set, and figures remain undone. The parent's section
11, Later, is unchanged (`deployment-binaries.md`).

- **Design departs:**
  - Item 1, smith's crate: it is two, `smith-agent-profile`, the
    configuration's values, the profile, its checks and the host's
    limits, beneath the service, and `smith-agent-document`, the reader
    and the program helpers, on it (`smith/docs/design/shell.md`, 2.6). The reader
    resolves a document into the service's own values and checks, so a
    reader without the service needs them beneath it. The plan's lead,
    2026-11-08, kept by its second cut, 2026-11-09.
  - Item 2, the launcher's crate: a step crate and its host,
    `temper-fetch` and `temper-fetch-shell`, and the fetch's machine
    and host go with Claude Code's, Forgejo's fetch then running on them
    (`../protocol/setup.md`, 6.6 and section 11). Claude Code's fetch is
    that machine's, so a crate that fetches it without the forge's setup
    holds the machine. The plan's lead, 2026-11-08, kept 2026-11-09.
  - Item 3, its table: `temper-agent` holds `temper-shell`'s `help` row
    beside its modes, where `deployment-binaries.md`, section 4, lists
    its modes alone. It starts from `temper-shell`, whose `--help` names
    `help` on its second line, and the row costs no crate the agent
    doesn't link already (`../protocol/README.md`, 4.1). The plan's
    lead, 2026-11-09.
  - Item 3, its words: `temper-agent`'s failures carry its own name,
    `temper-agent: `, `temper-agent --list: ` and `temper-agent:
    claude: `, where `temper`'s carry `temper internal agent: `,
    `temper internal agent-list: ` and `temper: claude: `
    (`../protocol/README.md`, 4.1). The plan's lead, 2026-11-08.

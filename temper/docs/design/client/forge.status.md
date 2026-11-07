# forge: status

2026-11-07. Where temper's code lags or departs from [forge.md](forge.md).

Section 5.4, *A repository's link*, has no known gap in leaving an empty
OWNER or NAME without an address, for plain and signed-in links alike.
Unlisted sections, including public documents (11), have no known gap:
queue merges read from the landing log as landed lines and as landings
through their changes, filtered by jig's handed tasks. A landing's zero
stale-proof count draws nothing; a positive count remains (`browser.md`,
8.3). Older public history remains deferred in section 10.

- **Departs:** 5.1: GitHub's installation records carry no repository selection
  mode, so the client cannot tell selected repositories from all repositories.
  "Not listed" asks the person to check that the repository exists and grant it
  to the App for either mode.
- **Not yet:** 2.1: "in GitHub's merge queue, n": `Pull.queued` is not
  read (`../domain/forge.md`, 9.1). 2.3:
  amending a change's gates from its panel. 4: approvals, a person
  task's card at a head and the browser panel's counted forge reviews;
  ctl's `change show` carries the counted people and heads, the rules'
  approval deficits and a pull-request instruction while they wait.
  5.1: "Land without CI" for a repository with no CI. 5.2: health. 5.3:
  the forge's part of a project's settings. 5.4: in a row's summary,
  "Forgejo at HOST", WEB and a connected forge's repositories, each with
  its project: the view names another Forgejo "Forgejo", draws "Forge N ·
  KIND" and its connection's words, and its line is its set up's first
  repository alone (`temper-client-view`'s `forge/connect.rs`, `name`, and
  `forge/forges.rs`, `repository_line`); its load, the
  App's setup URL and the `Lacking` line on the row; Forget,
  with no request on the wire; the warning of no git on the
  machine. 5.4, Elsewhere: the dot leaving out the playground's
  forge, which `temper-client-domain`'s `forge/listed.rs` `rows` counts.
  6.2: a drift hold's card, what changed, by whom and when.
  7: a goal's issue, landings as news, findings on a project's page. 9:
  the reads of the forge's other parts. 5.4: a managed Forgejo set up
  failed offers no Connect in the form: `temper-client-domain/src/forge/connect.rs`,
  `offered` and `disconnected`, leave that state out and show its
  `setting_up` step, which says it is connecting, and `forge/admin.rs`,
  `send_target`, sends nothing on it, so `temper ctl`'s `forge connect
  forgejo-admin`, which the engine admits, is the way back.
- **Departs:** 5.4, A managed Forgejo: a stop empties the admin’s
  token whatever connect it ends: with no admin connect sent, any
  stopped managed Forgejo's row empties it, and while the sent
  connect's forge stays stopped, each delivery of the forges watch
  empties a token pasted again (`temper-client-domain`'s
  `forge/admin.rs` `stopped`), where the bot's tokens are emptied by
  their own connect's new stop alone (`forge/tokens.rs` `stopped`); the
  code records no reason, and a code task gives the admin's form the
  tokens' rule.
- **Design departs:** 5.4, from the request `ui-repo-link` of 2026-11-06,
  decided by its plan's cut: a folded forge row's repository stays plain,
  with its link in the body's Repositories, because the summary's description
  is text inside a disclosure's control. The request asks for a link wherever
  the repository is named. A separate control while folded waits in section 10.
- **Design departs:** 5.4, from the request `ui-settings-tabs` of 2026-11-05,
  decided by its plan's cut: a row's first line is the forge's name, and
  "Forge N · KIND" opens its second, before its connection's words, with its
  repository on a third, where the request's row is "Forge 1 · Forgejo", its
  connection and its repository line, and its summary two lines. The names
  stay, as the plan keeps them for the tests of these pages.

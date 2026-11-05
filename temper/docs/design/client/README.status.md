# README: status

2026-11-05. Where temper's code lags or departs from [README.md](README.md).

- **Not yet:** 3.5, The welcome, 5.5, A sign-in: on a fresh `temper box
  --fake-backends`, the ChatGPT sign-in walked in Chromium with the
  recording bundle leaves no model usable, so Ready stays at "Model
  none yet" with "Create your project" disabled, and `journeys.md`,
  9's walk stops there (`journeys.status.md`, 9). Ready's words give
  its endings: `Failed` `Later` with no delay, "Try later", and on a
  retry `Failed` `Misconfigured`, "OpenAI refused temper’s sign-in.
  Check the configuration’s registration.". Each ending covers several
  causes (`jig-protocol-accounts`'s `translate.rs`), and which one this
  is, and so the fault, is not known. The native client's walks on the
  fake box reach a usable model and an enabled Create
  (`temper/crates/temper/tests/fake_backends/walk/sign_in.rs`), and
  Chromium signs in at `/accounts/connect` against the browser
  harness's own issuer and test provider
  (`temper/tests/client-browser/tests/browser_all/browser_accounts.rs`);
  no test joins Chromium and the fake box's issuer, so no test is red
  for it.
  3.5, Ready: a sign-in that ends `Made` or `Renewed`, not declined,
  with no account usable leaves Ready silent, where the document says
  no such sign-in does: `temper-client-view`'s `pages/connect_model.rs`,
  `welcome_end`, draws nothing for them, and 5.5 has no words for it
  (README.md, 11). No walk has reached this ending.
  3.4, 5.5: the composer's
  supplied default, jig's
  `resolved_default`, which neither jig's `Me` nor temper carries. 3.5:
  `Board { project }` with its reordering, and `Changes { project }`: no
  address in `temper-client-domain`'s `address.rs`. 3.5, 5.13, 10:
  `HowWorkRuns`, `Proposal { proposal }` and a proposal's card in a chat,
  with their tests; ctl supplies the review groups and running-work effect
  (`tools.md`, 5.2). 3.5, 4.6, 8: Forget, on the Forges page and as the FORGE
  request `Forget`: no `forget` in `temper-client-protocol-wire`'s schema
  or in `temper-service`'s routing. 3.5: Ready's and Home's model lines name no
  email for a ChatGPT sign-in, whose account's display is empty, since
  the engine keeps no email: it waits on
  `jig/docs/design/draft/account-facts.md`, 6, its second step.
  3.5, 11: a project's creation age and its repository's first CI run
  are not supplied to Home. Its kicker uses the current day and time,
  and its repository step describes checked changes landing through
  the merge queue, per the user's answer to `home-age-and-ci` in
  `temper/docs/plan/look-round-3/plan.toml`; neither claims the missing
  fact (README.md, 11).
  4.4: Home's GitHub step's repository picker and reached count, and
  the picker, which the adoption page does not draw
  (`temper-client-view`'s `forge/page.rs`).
  5.5: a plan's first use, its card in the conversation. 3.5, 3.6, 10:
  Delete on a project's settings, confirmed by its name, and jig's
  `RemoveProject` by `shapes::remove_project`, with the chat
  template's grace: no such act in `temper-client-domain`. 3.5, The welcome:
  for a person whose role does not hold the deployment's forges, the model
  alone deciding;
  `temper-client-domain`'s `pages/welcome.rs` `decision` reads a
  refused `forges` watch as no forge, so a usable model opens Forge.
  The playground offered on Forge's and Model's foot while a real
  account alone is kept; `temper-client-view`'s `pages/welcome.rs`
  `main` offers it only while the store is empty. FORGE in "FORGE · not
  finished" as the forge's name; `forge/connect.rs` `unfinished` prints
  "FORGE" itself. "Go on" after a restart: the row's `app` is the forge
  domain's memory alone (`temper-engine-domain-forge`'s `domain.rs`
  `apps`), so a GitHub forge found unfinished after a restart offers
  "Start over" alone. The Model step showing an account added before
  as ready, where it shows `ConnectModel`'s Choose.
  3.5, With no project
  yet, where the document shows the welcome: its heading, words and acts; `pages/chats.rs` draws the pane
  alone, the new chat's bar with no status and the list under "Chats".
  4.7: the "not installed" outcome, "temper's App is not installed on
  OWNER.", with "Install on another account": `temper-service/src/family.rs`
  maps `forge::AdoptionRefusal::NotInstalled` to `ForgeReason::NoAccess`;
  the client wire has no not-installed reason, pinned as a known gap by
  `temper-service/src/tests/setup_github.rs`,
  `unaccepted_and_other_account_adoptions_keep_the_wire_reason_gap`;
  no reason is recorded.
- **Departs:** 3.5, With no project yet, and 5.3: where the document
  shows the welcome, with no project, the banner's place for it reads "No project yet" followed by the new
  project's form, "New project" and "Create" (`temper-client-view`'s
  `frame.rs`, `switcher`), where 5.3 ends the switcher's list with that
  form and 3.5 leaves the first project to the playground's start or
  the welcome's Ready. The code records no reason; it lets a person make
  "Default" before Ready, the case of the draft's open question
  (`../draft/first-run-projects.status.md`).

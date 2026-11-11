# testing: status

2026-11-10. Where the tree's code lags or departs from [testing.md](testing.md).

- **Not yet:** the whole document: a walk from the sheet through planning, a live report and
  delegate tasks, Pause, Resume, a message and Close, which the request
  `ui-goal-page` of 2026-11-05 asks of the browser tier and of a box. The
  view holds Goals' link to `/chats` (`the_link_shows_with_a_tracked_template`),
  and the joint world's story reaches an
  ended goal's report from its chat through Home's Goals link and its Goals row
  (`tests/client/tests/suite/task.rs`, `a_chats_goal_shows_its_report_from_its_goals_row`). The fake
  box's model (`temper-fake/src/backends.rs`, `model_document`) writes a report
  only for the first chat's goal, task 2, and has it ask, wait or finish with no
  delegates, so it can show no planning and no delegate task: the journey
  `chat-draft` walks a chat's sheet to that goal's page and back to the chat alone.
  The request `ui-final-walk` of 2026-11-05 walked a live report, Pause, Resume and
  Close there by hand, which no test keeps. Of Ask temper, that model scripts
  one proposal, the cue "Show drafts and goals for this project.", and no document,
  and it has no hold of temper's own, so the gallery alone draws those states.
- **Not yet:** the whole document: the renders the request `ui-away` of 2026-11-05 asks,
  `home.html`'s away state beside the client's at 1440, 1000 and 420 wide, the
  switch on and off at each and toasts open. They are saved on a builder, outside
  the repository, so what they show is unknown here; the gallery holds their
  states (`web/gallery.md`, 2).
- **Not yet:** 
  `Board { project }` with its reordering: no
  address in `temper-client-domain`'s `address.rs`.
   a person with a project's role alone can read kinds 6 and 7,
  but cannot change them on the tab: jig's settings inventory requires
  a deployment role (`jig-domain-core`'s `documents/settings.rs`, `make`),
  so `temper-client-domain`'s `forge/ways.rs`, `act`, has no newest
  revision and sends nothing. The controls appear enabled and give no
  refusal, where the request of 2026-11-05 has a person who is no owner refused
  in jig's words, as one with a deployment role is. With no document to say a
  setting changed, that person reads them again only as the project opens or
  the link comes back. A project-scoped settings inventory needs a contract in
  jig before the tab can use it.
   Home draws neither "Show goals" nor "See the settings", so
  nothing opens the tab by them.
  `pages/README.md`, *Goals*: a project whose `me` lists no goal template opens no watch
  (`temper-client-domain`'s `pages/goals.rs`, `query`), so its page draws the
  kicker and the lede alone, with no heading and no filters, where the section
  says "No goals yet."; a project with no tracked template has a lede that
  stops at "landed work.", which the section has no words for.
   Forget, on the Forges page and as the FORGE
  request `Forget`: no `forget` in `temper-client-protocol-wire`'s schema
  or in `temper-service`'s routing.
  `pages/README.md`, *Ready*: two endings leave nothing under "Model none yet"
  (`temper-client-view`'s `pages/connect_model.rs`, `welcome_end`), and no
  test holds either: a sign-in `Made` or `Renewed`, not declined, with no
  account usable, since `pages/accounts.md`, 2 has no words for it and no walk has ended so
  (`README.md`, section 5); and any ending where `me` offers no ChatGPT plan's sign-in,
  since the function finds that way before it draws the ending's words.
   a plan's first use, its card in the conversation.
  `pages/README.md`, The welcome:
  for a person whose role does not hold the deployment's forges, the model
  alone deciding;
  `temper-client-domain`'s `pages/welcome.rs` `decision` reads a
  refused `forges` watch as no forge, so a usable model opens Forge.
  FORGE in "FORGE · not
  finished" as the forge's name; `forge/connect.rs` `unfinished` prints
  "FORGE" itself. "Go on" after a restart: the row's `app` is the forge
  domain's memory alone (`temper-engine-domain-forge`'s `domain.rs`
  `apps`), so a GitHub forge found unfinished after a restart offers
  "Start over" alone.
   the "not installed" outcome, "temper's App is not installed on
  OWNER.", with "Install on another account": `temper-service/src/family.rs`
  maps `forge::AdoptionRefusal::NotInstalled` to `ForgeReason::NoAccess`;
  the client wire has no not-installed reason, so ctl offers repository access
  and Forgejo collaborator-admin recovery rather than installation recovery.
  This gap is pinned by
  `temper-service/src/tests/setup_github.rs`,
  `unaccepted_and_other_account_adoptions_keep_the_wire_reason_gap`;
  no reason is recorded.
- **Design departs:** `pages/README.md` and `pages/settings.md`, section 2 and `testing.md`, from the request `ui-walk-large` of 2026-11-06:
  -  a proposal of settings shows in the Needs you of a project's owner who also
    owns the deployment, those who can decide it, by that plan's cut: a project's owner
    alone has no row of it (jig's `domain/engine/settings.md`, 6, and its status).
  - the whole document: that request's browser walk was made by hand, at 1440 and 420 on fresh fake
    boxes, its driver thrown away as that plan's cut has it, so no test keeps it. The
    client's world holds the card, count and Accept (its story of Ask temper's
    proposal of a project's ways reaching its Needs you and its rail), and
    the joint world a person's plan's return
    (`tests/client/tests/suite/status.rs`,
    `a_plans_project_has_its_brief_on_a_return_past_the_sitting`), past the sitting
    on the world's clock advanced by 31 minutes.

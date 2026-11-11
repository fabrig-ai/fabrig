# engine: status

2026-11-11. Where the tree's code lags or departs from [engine.md](engine.md).

- **Not yet:** the FORGE requests' `SetDraft`: the engine neither routes
  nor answers it (`../domain/forge/drafts.md`) [in flight: change 663].
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
  the whole document: Forget, on the Forges page and as the FORGE
  request `Forget`: no `forget` in `temper-client-protocol-wire`'s schema
  or in `temper-service`'s routing.
  `pages/README.md`, *Ready*: two endings leave nothing under "Model none yet"
  (`temper-client-view`'s `pages/connect_model.rs`, `welcome_end`), and no
  test holds either: a sign-in `Made` or `Renewed`, not declined, with no
  account usable, since `pages/accounts.md`, 2 has no words for it and no walk has ended so
  (`README.md`, section 5); and any ending where `me` offers no ChatGPT plan's sign-in,
  since the function finds that way before it draws the ending's words.
   a plan's first use, its card in the conversation. 
  Delete on a project's settings, confirmed by its name, and jig's
  `RemoveProject` by `shapes::remove_project`, with the chat
  template's grace: no such act in `temper-client-domain`. `pages/README.md`, The welcome:
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

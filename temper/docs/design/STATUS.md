# Status

Updated 2026-10-29. How far temper's code is from its design.

Closing a live Open chat carries jig's durable finish request through live
and Start delivery to smith's wind-down; its next text parks. Close stays
compound, with cancellation after the grace. Jig's tasks store and worker
wire are v9, smith's channel v5; v8 task stores refuse under the existing
policy. Other store families and client versions keep their versions.
Open fake scripts use final text; Finite finish calls and chat bounds hold.

Each design document's status is in `<doc>.status.md` beside it:
- [`README.md`](README.status.md)
- [`domain/README.md`](domain/README.status.md)
- [`domain/root.md`](domain/root.status.md)
- [`domain/forge.md`](domain/forge.status.md)
- [`domain/landing.md`](domain/landing.status.md)
- [`domain/worker.md`](domain/worker.status.md)
- [`domain/agent.md`](domain/agent.status.md)
- [`domain/testing.md`](domain/testing.status.md)
- [`protocol/README.md`](protocol/README.status.md)
- [`protocol/configuration.md`](protocol/configuration.status.md)
- [`protocol/credentials.md`](protocol/credentials.status.md)
- [`protocol/channel.md`](protocol/channel.status.md)
- [`protocol/store.md`](protocol/store.status.md)
- [`protocol/llm.md`](protocol/llm.status.md)
- [`protocol/git.md`](protocol/git.status.md)
- [`protocol/forge.md`](protocol/forge.status.md)
- [`protocol/setup.md`](protocol/setup.status.md)
- [`protocol/sign-in.md`](protocol/sign-in.status.md)
- [`protocol/documentation.md`](protocol/documentation.status.md)
- [`client/README.md`](client/README.status.md)
- [`client/forge.md`](client/forge.status.md)
- [`client/browser.md`](client/browser.status.md)
- [`playground.md`](playground.status.md)
- [`client/journeys.md`](client/journeys.status.md)
- [`client/tools.md`](client/tools.status.md)
- [`qa.md`](qa.status.md)

Not design documents:

| Area | Built | Not yet |
|---|---|---|
| Retired QA lab | The lab was retired for `qa.md`'s process; its shared cluster fixture remains in `crates/temper/tests/fixtures/cluster/`. | — |
| `../help/box.md` | A walk, ctl alone: 1 the configuration by `temper init`; 2 the box, on real or fake backends; 3 ctl's sign-in by the printed link; 4 Forgejo on this machine and a repository adopted; 5 the model's account, imported, connected (with the sign-in's tunnel) or keyed; 6, 7 a change asked in a chat, followed to its landing; 8 the account exported before the stop. Every command line parses (temper's step test); QA follows it on fake backends (`qa.md` 3). | On fake backends it ends at step 6's reply: the fake model starts no change. Other help-to-product coverage unknown. |
| `../help/github.md` | A walk, ctl alone: 1 a shared App read-only, before the start; 2 the box and sign-in; 3 an App connected, by its key or from temper's manifest; 4 an installation accepted (`forge-accept`); 5 a repository adopted; 6 work on it. Every command line parses; QA accepts an existing installation through ctl (`qa.md` 7). | A new App's and an installation's pages on GitHub are a person's. Other help-to-product coverage unknown. |
| `../help/forgejo.md` | A walk of the managed mode, temper as a Forgejo's site admin: 1 the forge configured, before the start; 2 the start and sign-in; 3 connected as its admin, a first repository made and adopted; 4 another repository adopted. Every command line parses. | The connected mode is refused on main and no walk; the owned mode is the box's (`../help/box.md`). Other help-to-product coverage unknown. |
| `../help/server.md` | A walk: 1 the engine's configuration; 2 its secrets; 3 Forgejo, then the engine; 4 the TLS proxy; 5 the workers' host over WireGuard; 6 ctl's sign-in through a tunnel; 7 a redeploy from main, the model's account carried by ctl's export and import. Every command line parses. | Its proxy, WireGuard, units and tunnel are the operator's, walked by no test. Other help-to-product coverage unknown. |

kept: real-loop/qa tests no lower test holds the whole assertion of yet

- `github::a_seeded_app_reopens_with_its_deployments_settings_and_owner`
- `template_bounds::the_two_call_agent_past_smalls_page_is_refused_by_name`
- `qa_adoption_hold::a_chat_created_after_home_adoption_gets_a_writable_run_branch`
- `qa_github_land::a_chats_failed_github_check_is_repaired_at_a_new_head_and_lands_once`
- `qa_github_land::checked_head_lands_once_and_repeated_reads_name_main`
- `qa_github_land::landed_chat_is_readable_on_the_persons_page`
- `qa_plan_chat::words_during_a_turn_are_answered_once`

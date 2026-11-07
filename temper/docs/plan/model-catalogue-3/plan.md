# model-catalogue-3: the plan

Step 3 of `temper/docs/design/draft/model-catalogue.md`, section 7: its section 5. The cut wrote the design: skein's `docs/design/llm.md`, 4.9, `llm-connection.md`, 4.3 and `fake-llm.md`, 3.2; smith's `protocol/agent.md`, 6.1 and `protocol/hosts.md`, 2; jig's `domain/accounts.md`, 3.4, 3.7 and 4.3, `domain/engine.md`, 11.1, 17.1, 17.7, 17.11 and the new 17.12, `domain/hosts.md`, 5.1, `protocol/channel.md`, 5.1, 5.2 and 8, `protocol/store.md`, 4.4, `protocol/configuration.md`, 3, and `client/domain/contract.md`, 3.2 and 5.1; temper's `client/README.md`, 3.5 and `protocol/configuration.md`, 13.1. The contracts between parts are `notes/contracts.md`.

This file grows with each push. Tasks marked *to come* are not yet released.

## Parts and tasks

| Part | Task | Needs | Minutes | What |
|---|---|---|---|---|
| `provider-list` | `list-parse` | | 60 | a route's list body, read |
| `provider-list` | `list-fake` | | 60 | the fake's `GET /v1/models` |
| `provider-list` | `list-exchange` | `list-parse`, `list-fake` | 110 | one bounded GET on an endpoint |
| `agent-read` | `agent-list` | `list-exchange` | 170 | `--list`: the agent's one request |
| `agent-read` | `host-list` | `agent-list` | 110 | the host half runs it, the grant on stdin |
| `worker-read` | `read-wire` | | 110 | `read list` and `list read`, inert |
| `worker-read` | `host-read` *to come* | `read-wire`, `host-list` | 130 | the hub's queue and the worker's component |
| `worker-read` | `temper-worker` *to come* | `host-read` | 80 | temper's worker and binary |
| `store` | `list-surface` | | 140 | `ListRecord`, `first_read`, `Decider::List`, inert |
| `reads` | `list-patterns` | | 80 | `keep`, `drop` and the matcher |
| `reads` | `list-change` | `list-surface`, `list-patterns`, `read-wire` | 90 | `from_list`: what a read's rows change |
| `reads` | `fleet-reads` | `read-wire` | 110 | the fleet holds and places a read |
| `reads` | `account-reads` | `list-surface`, `list-patterns` | 120 | when a read is due, and read again |
| `reads` | `core-reads` *to come* | `list-change`, `fleet-reads`, `account-reads` | 170 | an answer becomes one decision and one commit |
| `reads` | `offer-narrows` *to come* | `list-surface`, `account-reads` | 90 | the offer and a task's first choice read `names` |
| `page` | `client-facts` *to come* | `list-surface`, `account-reads` | 140 | `new_until`, `AccountLine.list`, `ReadLists` on the wire |
| `page` | `models-read` *to come* | `client-facts` | 140 | Settings, Models and All models draw it |
| `on` | `list-on` *to come* | `temper-worker`, `core-reads`, `offer-narrows`, `client-facts`, `list-fake` | 110 | `temper init`'s patterns; temper's fakes list |
| `on` | `list-story` *to come* | `list-on`, `models-read` | 110 | the client world's story |
| `on` | `list-help` *to come* | `list-on`, `models-read` | 40 | the help says so |
| `on` | `list-close` *to come* | everything | 50 | status files; the look's account |

## Critical path

`list-parse` (60), `list-exchange` (110), `agent-list` (170), `host-list` (110), `host-read` (130), `temper-worker` (80), `list-on` (110), `list-story` (110), `list-close` (50): about 930 implementer-minutes. `read-wire`, `list-surface` and `list-patterns` go out at once, and each family beside the path builds on them.

## What the cut decided

- **The agent reads in a process of its own,** `--list`, as `relay.rs` is a process of its own, and smith's host half runs it as it runs a program's status, the grant on standard input. A run's channel, domain and lifecycle stay as they are. The draft says "smith's agent makes the one request": it does.
- **A read is a sibling of a login check,** on the hub's source queue: no slot, no workspace, no attempt.
- **The fleet holds a read until a worker can take it,** so that a box with no worker yet says nothing failed.
- **`names` is a bit per setting,** since a number is never given to another name, so the record is fixed-size and the offer reads it with no name lookup.
- **A provider with no `keep` has no list,** so no fixture's document changes and no world answers a list it wasn't asked for.
- **An account's first source says whether it has a list,** since the order decides who pays.
- **A failed read keeps the last good names.** The draft's section 3 and section 5 pull apart here; section 5 wins, and `model-catalogue.status.md` says so.
- **A person's added name is offered whatever a list says,** since section 6 is the way to what a provider doesn't list.
- **A list's window and output are capped by the stock's.** Draft section 2 has them stay the stock's, and an unbounded one could leave a charter's `rest` with a need no worker's bound holds.
- **A person's removal of a list's model makes it a person's,** so a list doesn't undo it, at one line's cost.
- **A read's revision is any revision:** it withdraws stale proposals and makes charter versions, by 17.6 and 17.11.
- **`me` isn't marked by a read that moves nothing.** A new `Touch` carries a read's times to the accounts document alone, and a day's alarm marks nothing.
- **Reads don't go through `route_account`,** whose marks reach every Frame.
- **A stored format, a protocol and a public interface** all change in place, with no version bump (the request): the accounts and settings families, the workers' wire, the client's wire, `temper.json`.

## Overlaps with drafts beside this one

- **`limits-worlds`** has hunks across `temper/crates/**`, `jig/crates/**`, `smith/crates/**` and `skein/crates/**` on its trunk. Not avoidable where a literal gains a field; each of this plan's literal hunks is one line.
- **`diagrams`:** `temper-client-shell-web/temper.css`. New rules are under the settings marker, away from its.
- **`account-email`** runs beside this on accounts (the request): whichever lands second catches up. `list-surface` holds the record's hunks.
- **`call-room`, `reference-walks`, `v0.1.0-ladder`, `configuring-walks`:** queued or being cut, and none named here touches these files yet. Unknown beyond that.

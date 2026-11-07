# ui-trial-account

The account just connected is the one tried, whichever provider comes first, in either order, and for a second account of one provider; the first usable account keeps today's trial, naming no `llm`. The request: `inputs/request.md`.

## Parts and tasks

- **choice** (`jig/docs/design/domain/accounts.md`, 4.3; `jig/docs/design/domain/authority.md`, 6.1): `account-choice`, about 100 minutes. jig's `Core::account_llm`, `PersonSeed::models` and `Event::PersonTask`'s `llm`; temper's one literal gains `llm: None`.
- **trial** (`temper/docs/design/domain/trial.md`, 2 and 4): `trial-account`, about 150 minutes. The root's `Make` decides the choice; the engine world's story holds both orders and a second account of one provider, and the joined client story holds the later account's own line and its "Try again".

About 250 implementer-minutes in all, two tasks, as the request's limit asks.

## Critical path

`account-choice` (100), then `trial-account` (150): about 250 minutes. The second builds on the first's field and function, so nothing runs beside it; the shapes between them are `notes/contracts.md`.

**The meeting point** is `trial-account`'s client story, `a_later_account_is_the_one_tried` in `temper/tests/client/tests/suite/model_check.rs` (today's `no_model_is_named`): the real client and engine, a key then a program's sign-in, the later account's trial on that account, its line, its failure and its "Try again".

## What the cut decided

- **A later trial names its account's first offered model** (the request's 3, the smaller of its two ways). jig's account preference lives inside an `LlmChoice` beside its main model, and a task given a choice is never moved: it waits for its account, saying why (`jig/docs/design/domain/accounts.md`, 4.3). An account preference with no model would change `LlmChoice` and every reader of it, and the model selection rule; naming a model changes one event's field and adds one read.
- **"First" is the offer's.** A trial names no `llm` when the person's offer in project 0 (jig's `domain/engine.md`, 11.1), for charter 10 and the seed's models, holds no account but the one tried: then what a new chat takes is that account, which is the draft's reason kept. Where it holds any other, the person's own or the deployment's, the trial is held to its account. Each `Make` judges it again, so "Try again" retries that account, a first one included once another is usable.
- **The model is jig's to choose:** `Core::account_llm` takes the charter's first model the offer holds with that account, else the first that account serves, so that its wait names it, each at its default effort, as a chat's default takes it. An account that serves none of charter 10's models is `Refused`: "The try couldn't start."
- **jig gains a read, not a rule.** `account_llm` is generic, beside `preview_llm`, in a module of its own so that `runs/llm.rs` changes only in three helpers' visibility; the trial's "first or later" is temper's, in its root (`trial.md`, 4).
- **The stories run in the focused tier.** The engine world's trials story takes the three orders as rows (Claude and ChatGPT stand as fake providers 1 and 2); the joined client story keeps its one order, a key then a program, and adds the failure and "Try again". No new browser test, and no row of a slow tier.
- **Design:** `trial.md`, 1, 2, 4 and 8; its status gains a **Not yet** on 2 and 4 that `trial-account` closes. `model-check.status.md`'s departure on section 2 narrows: a later account's trial names its model, where the draft has every trial name none. jig's `accounts.md`, 4.3, and `authority.md`, 6.1, gain the choice and the field, each status a **Not yet** that `account-choice` closes.
- **Formats:** no stored format changes. `Event::PersonTask` is an in-memory event between temper's root and jig's core; the trial's task keeps its `llm` in the tasks' family as any task's does.

## Overlaps

- **`model-catalogue-3`** narrows the offer per account and changes `jig-domain-core`'s `runs/llm.rs`, `documents/catalogue.rs`, `vocabulary.rs` and `jig/docs/design/domain/accounts.status.md`, and `temper/tests/client/src/**`. `account-choice` keeps to a new module, three visibility words in `runs/llm.rs` and one field in `vocabulary.rs`; `trial-account` touches `temper/tests/client/src/**` only if a helper must. `account_llm` reads the offer, so a narrower offer narrows it with no change here. Whichever lands second catches up.
- **`limits-fuzzy`** changes `temper-engine-domain`, `temper-engine-domain-trials` and `temper/tests/engine/domain`; `trial-account` changes `route/trials.rs` and the trials story, which I couldn't keep off.
- **`ui-wait-words`**, still being cut, may change `trial.md`'s 6 and `trial.status.md`'s departures on the wait; this plan changes `trial.md`'s 1, 2, 4 and 8 and adds a **Not yet** of its own.
- **Journey prints** conflict with whichever plan re-records next: regenerate on a catch-up, never hand-merge.

## After landing

- A real box with a Claude account and a ChatGPT account, each order, and two of one provider: the later account's own row shows its try. That walk files its findings back to fab.

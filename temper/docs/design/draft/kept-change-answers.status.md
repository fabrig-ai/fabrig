# kept-change-answers: status

2026-11-15. Where the design lags or departs from [kept-change-answers.md](kept-change-answers.md).

The design carries the draft: section 1 in `../domain/forge/change/questions.md`, 1 (*Held for a dependency it did not start from* and *Held for message room*), section 2 in jig's `domain/engine/views.md`, 3 (*its question*), `client/domain/contract/documents.md` and, for temper's pages and `ctl`, `../client/pages/goal.md` and `../client/ctl/results.md`.

- **Design lags:** none.
- **Design departs:**
  - 1: an answer is not delivered at once in every case. It is held while some pending dependency of the change is not one its task started from and is not a change into another repository, and relayed once none is; held for message room apart from that rule, as section 1 asks. So the test `an_answer_with_a_new_dependency_waits_for_it_to_land` stays as it is, and section 4's story keeps the change's dependency marked started from, as an early start marks it. The user, 2026-11-15 (person task 3430), in place of the draft's "delivered at once": the cases that stopped were early producers kept by dependencies they had started from, whose workspaces already hold that work. An answer that comes with a new dependency is the other case: the producer lacks that work, and the hold is what makes its next run start from the landed tip. Moving that wait into the coordinator's charter would put a mechanism in an LLM's rules. The stock coordinator charter stays as it is.
  - 4: the relay's pure function (`temper-engine-domain-forge-change`'s `relay`) takes whether a dependency holds the answer as its input, the root deciding it from the task's pending dependencies and its started-from marks; its unit tests pin an answer held while that input is set and relayed once it is not, and the zero-room hold and retry. Delivery while a started-from dependency is pending, and the hold of none by a change into another repository, are pinned by the root's focused stories in `forge_routes/questions.rs`, where dependencies exist. The goal's designer, 2026-11-15.
- **Open:** none.

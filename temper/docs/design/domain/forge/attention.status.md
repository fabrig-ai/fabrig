# attention: status

2026-11-10. Where the tree's code lags or departs from [attention.md](attention.md).

- **Departs:** 3: a project's first run's words count their span from the
  epoch: `temper-engine-domain-forge/src/status/words.rs`, `specification`,
  takes `Wall::EPOCH` with no brief, so the agent reads "Since the last brief:"
  and some half a million hours, where a brief has never been written. This
  section names no span for a first brief, the plan's test gives its row a brief
  (`its_words_say_what_landed_since_the_brief`), and no reason is recorded.
- **Design departs:** 2, 3, from the request `ui-home-status` of 2026-11-05,
  each decided by its plan's cut, for the user to see:
  - 2: `Looked` keeps `next_seen`, a field the request doesn't name, where the
    request has a new sitting set `seen` to the current `landed`. That would read
    as nothing landed at the moment the person comes back; the request's own
    definition of `seen`, the count at `previous`, is what holds.
  - 3: `ProjectFacts.running`, a field the request doesn't name, so that "none
    running" survives a restart and a run held past an hour never stops the next.
  - 3: a run is a procedure of the project's period that delegates charter 9,
    where the request has an agent's task under the project's pool, because jig's
    connector task takes the project's whole ceiling as its authority and can't
    narrow it, and an agent reading change titles should not hold merge
    authority. So a project whose policy's delegation doesn't name charter 9 has
    no brief, and a period its people's pools reserve whole has none either,
    which a stock project's room holds off (`../../protocol/configuration/stock.md`).
  - 3: a run's task is kept, where the request has it forgotten once read,
    because jig forgets a result only for a person's request (`README.md`, section 11).
  - 3: a live goal's "landed and total changes" are its delegates done and
    total, because the forge keeps no count a goal, and "failures the forge holds
    since" are what each tier holds now, because it keeps no times on them.
- **Design departs:** 4, 5, from the request `ui-away` of 2026-11-05, each
  decided by its plan's cut, for the user to see:
  - 4: `Away` keeps `off` in place of the request's `on`: "23:10 to 08:03" needs
    the time it went off, and a row holding both could disagree with itself. On is
    `off` none; the wire's `SetAway` keeps `on`.
  - 5: `Decided` keeps `party`, a field the request doesn't name, and its document
    is the asking party's alone: "decided for you" would be false on a colleague's
    Home, and Overturn sends the requester's message.
  - 5: the caller's person is found through its chat, where the request has
    `domain.core.tasks.delegation`: a chat's goal has the chat as its requester, so
    the root takes jig's `served_chat`, then the chat's person.

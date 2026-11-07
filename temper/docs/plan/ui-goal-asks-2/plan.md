# ui-goal-asks-2

What a walk of a goal asking its person tripped over. The request: `inputs/request.md`. Three tasks, side by side, each with `needs = []`.

## Parts and tasks

- **question** (`temper/docs/design/client/README.md`, 3.5, 5.4, 5.8): `question-whole`, about 100 minutes. A person task's question reads whole under Needs you and on its goal's band, its choices as buttons; one that stays cut says so and links on. The funds card's words.
- **start** (`temper/docs/design/client/README.md`, 5.14, 3.5, 5.8): `goals-start`, about 110 minutes. Goals' link opens the sheet with a Draft field; the sheet says when nothing funds a goal and won't start it; a goal's page counts landings.
- **decide** (`jig/docs/design/client/domain/contract.md`, 3.2): `decided-first`, about 40 minutes. A decision on an ended proposal answers `decided first:`.

## Critical path

No task waits for another: `goals-start`, about 110 minutes, is the longest.

## What the cut decided

- **Read, not carry.** The whole question comes through `TaskText`, as a goal's page and a chat's first message already read theirs. Carrying it would change the wire for the same words. The bound is `long_text`, which bounds a task's words, so a person task has no real bound to be cut at: only a read that ended short leaves it cut, and that says so.
- **The choices aren't plain by design.** README 3.5 has buttons for a person task in words. They were missing because cut words parse as no line form, so the whole read brings them back. No status entry calls them plain.
- **A `message` question is cut too.** The same document bound cuts it and no `TaskText` serves it, so its card says it's cut and links to its task's page. The request's ruling reads on it, though its walk met a person task.
- **One whole question to a page, at most 16 choices.** The view prices a tree, so a page holds one question at `long_text`: a later band draws its line and "Answer". SHORT stays the bounded words' first line, so Home's price doesn't move.
- **The funds card's words.** A person has no way to add to a task's budget here. A run may have proposed more, so the words offer that proposal if it waits, otherwise closing and starting again once the pool has room. temper's override keeps its hole.
- **Stock can reach a zero budget.** Stock's role 2 has a pool of one chat's spend, so one open chat takes it; an owner's holds eighty. The sheet therefore matters at stock's own figures, not only on the probe's. Funding's rules don't change and no budget field is added. A way with a fixed amount is left to `shapes`, which judges or proposes it as ever. A pool with no reservations has its own plainer line, since open chats don't hold it then.
- **"Landed" counts landings where it can tell.** A goal's page has its delegates' results, so its bar and fold count `ResultValue::Change { kind: 1, .. }`, the test a delegation's card makes. `TaskLine` counts every delegate ended done, so a Goals row and a goal's headline now say "N of M tasks done." A count of landings on `TaskLine` would be a stored and a wire field for a small word. This is a **Design departs** entry and an open question in section 11. The request allows the format change; this cut judged it not worth it here.
- **The ctl error is a fault, not the walker's flag.** ctl sends `running: Some(Stays)` for a settings proposal with or without the flag, and jig's task-proposer branch refuses any `running` before it looks for the ended row. The fix is in jig, so `temper ctl`'s own code doesn't change; `temper/migration` isn't running in any case.
- **More ways → Goal stays.** README 5.3 has it open a new chat for a new draft, so that isn't the fault the request reads it as. Only Goals' link was wrong.

No stored format, protocol or public interface changes. No new browser test.

## Overlaps not avoided

Whichever lands second catches up.

- `temper/docs/design/client/README.md`, `README.status.md`, `browser.md`: `model-catalogue-2/models-proof`, `look-round-13/look-proof`, at other sections. This cut's own design commit carries them.
- `temper/crates/temper-client-view/src/words.rs`: `look-round-13/forge-cards`, `ui-small-1/small-words`, `ui-small-1/link-why`, and `question-whole` beside `goals-start`, at other lines. `notes/contracts.md` gives each its ids, clear of model-catalogue-2's.
- `temper/tools/gallery/**`: `goals-start` beside several of `look-round-13`'s. Regenerate on a catch-up.
- `temper/crates/temper-client-view/src/pages/goal.rs`, `temper/tests/client/tests/suite/chat_draft.rs`, `temper-client-shell-web/temper.css`: `question-whole` beside `goals-start`, at different functions and markers, as each spec says. These two share client crates, against one writer per crate: they touch different files but for these, and holding one behind the other would add about 100 minutes.

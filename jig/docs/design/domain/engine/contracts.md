# Bounded waits in jig: contracts

Provisional, 2026-11-24. The shapes that the changes of bounded waits'
jig goal share while they run, and that no other area reads
(`temper/docs/design/draft/design-economy.md`, 4.3). The lasting rules
are `../tasks/lifecycle-holds.md`, 1 to 3, `../tasks/messages.md`, 2,
`../tasks/proposals.md`, `../people/README.md`, 7, and `waits.md`, 4 and
5. This document goes once those changes have landed.

## 1. The hold at a bound

- **`Hold::Waited`,** a unit variant of `jig_domain_tasks::Hold`, after
  `Circle`: a wait of the task's reached its bound
  (`../tasks/lifecycle-holds.md`, 1). It is no placed hold, is valid,
  escalates as every cause does, and its release's effect is the one
  `Deadline`'s release has (judged afresh).
- **`HoldSource::Wait { words: Box<[u8]> }`,** a variant of
  `jig_domain_tasks::HoldSource`, after `Account`: the words of the waits
  the task was in at the bound, UTF-8, 1 to `jig_domain_tasks::WAIT_WORDS`
  bytes, 512. It goes with `Hold::Waited` and `Hold::Deadline` alone; the
  tasks child refuses it with another cause as it refuses a source that
  doesn't fit its cause.
- **The core holds** with `tasks::Event::HoldFrom { task, why, source }`,
  `why` `Waited` or `Deadline` and `source` `Wait { words }`, as it holds
  for any cause it classifies.
- **Every store and wire that carries `Hold`** carries `Waited`, and its
  category in the client's wire, `HoldCategory::Waited`, with the words on
  the chip's existing `words`: no new member carries them. In each
  schema, `waited` and `wait` come last in their enums (after `exceeded`
  and after `preparation`), so that every tag before them keeps its
  number.

## 2. The core's bound alarm

- **One place takes every outcome** the alarm takes
  (`../tasks/lifecycle-holds.md`, 1): a match on the wait's kind and
  outcome, which says both which waits arm the alarm and what is done at
  their bound. The change that builds the alarm takes the holds; it leaves
  `question` waits, and a `party` wait whose entry is a question,
  unarmed and untaken, at an arm the change that passes questions up
  fills (`../tasks/messages.md`, 2), and names that place in its report.

## 3. The client's wire

- **One move:** this goal moves jig's client wire once, from version 44
  to 45, in the change that adds `HoldCategory::Waited`. The changes after
  it that change the client's wire (a stop's reason, a question's
  refusal naming who answered) change version 45's schema in place,
  moving no version again. The public wire's `PublicHold` gains `waited`
  in the same change, which moves the public wire's version as its layout
  says (`../../protocol/client/versions.md`, 1).
- **Until a hold's words are made** (`../tasks/lifecycle-holds.md`, 3), a
  `Wait` source's words reach the chip's `words` as a preparation's do.

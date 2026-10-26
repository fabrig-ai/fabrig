# Supplement temper-03.s2: jig 13.2's repin, with its engine, store and conformance repairs

Added by ruling 199, from temper 03's block of 2026-10-17T01:32:47Z
(report `reports/temper-03-20261017T013223Z.md`), as ruling 153 added
temper-03.s1. In session 03, before 03.4. Read first: this plan's
README, "Rules of this plan" and its Pins; `temper/03-client.md`, 03.4.

Shape: the repin 03.4's Needs name (jig 13.2's landing), made on
`client-api/temper-03-4` (the local repin `eab4babab` and its saved WIP
`1f60a44a9` move to this step), with every repair it needs in temper's
engine, store and conformance adapters and their tests, the four new
conformance scenarios (`Compound`, `Grace`, `ConversationEndsWhileRead`,
`QueryWatchedWhileTasksEnd`) and their 12 actions included: each runs
against temper's adapter and passes. By ruling 201, it also carries
03.4's Scope and Files as the saved WIP holds them: the client crates'
and tests' path rewrites (`jig_domain_client::` and `jig_client_domain::`,
`pages::` to `page::`, `limits::Backoff` to `limits::ui::Backoff`), the
link surface (`link::Saved` is a struct at jig 13.2), and 03.4's Shape
and Removes (`store::Record::Jig`, `Key::Jig`, the root's `Save` and
`Erase` of the reference UI's records). A scenario that needs behaviour
temper's engine lacks is reported as a question, never skipped.
Test shape: no new test; the conformance suite and temper's gate.
Proof: temper's gate, at the jig 13.2 pin.
Size: aim for at most 550 added production lines (415 counted with
path rewrites at 01:32Z, `logs/temper-03/03.4-repin-counts.txt`, the
client's about 179 included, before the four scenarios' adapters); test
lines reported apart; past it, report why. The README's 200 lines do
not bind this step; they bind 03.4 again from its next commit.
Don't: 03.4's new unit test and its Proof (they follow on this
landing); keep `jig-domain-client` in a manifest or alias its paths; a
lint exception; a skipped or ignored scenario.
Ruling 216: the pin stays at jig `c92d4a09d`; the fuzzy merge story
`a_merge_in_progress_is_pushed_with_both_parents` takes ruling 163's
`#[ignore]` (its point 2), and temper 04.4 removes it. That mark is not
a conformance scenario's skip.
Done when: `git -C ~/src/rust/temper log origin/main --all-match
--grep='^Plan: client-api$' --grep='^Step: temper-03.s2$' --format=%H -1`
prints its landing.

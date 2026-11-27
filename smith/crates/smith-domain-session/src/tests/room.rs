//! Observe the session's reservation owners through valid transitions.

use super::*;
use crate::llm::{Evidence, Purpose};
use crate::session::result_credit_for_test;

pub(super) fn held(h: &Harness, owner: Token) -> (u64, Option<u64>, u64, u64) {
    let room @ (total, provider, batch, program) = result_credit_for_test(&h.domain, owner);
    let sum = provider.unwrap_or(0).checked_add(batch).expect("provider and batch fit");
    assert_eq!(Some(total), sum.checked_add(program), "the total is the sum of the holders");
    room
}

#[test]
fn reverse_read_terminals_return_only_their_own_batch_shares() {
    let limits = Limits { parallel_tools: 2, ..LIMITS };
    let mut h = Harness::new(&limits);
    let (owner, _) = h.open(1);
    let before = held(&h, owner);
    let content = Box::new([tool_call(b"a", cat(b"a")), tool_call(b"b", cat(b"b")), tool_call(b"c", cat(b"c"))]);
    let first = h.batch(Event::Completed { owner, completion: completion(content, Stop::ToolUse) });
    let &[a, b] = first.as_slice() else { panic!("two reads") };
    let share = tools::result_bound(&cat(b"a"), &limits.tools).expect("read cap");
    let paired = share.checked_mul(2).expect("two bounded read shares");
    assert_eq!(held(&h, owner), (paired, None, paired, 0));
    assert_eq!(h.step(ran(b, b"second")), None);
    assert_eq!(held(&h, owner), (share, None, share, 0));
    let (c, op) = running(h.step(ran(a, b"first")));
    assert_eq!(loading(&op), place(b"c"));
    assert_eq!(held(&h, owner), (share, None, share, 0), "only the next batch's read holds a share");
    let (_, prompt) = calling(h.step(ran(c, b"third")));
    assert_eq!(held(&h, owner), before);
    assert_eq!(
        prompt.messages.last().expect("settled result message").content.as_ref(),
        [result(b"a", read(b"first")), result(b"b", read(b"second")), result(b"c", read(b"third")),]
    );
}

#[test]
fn a_cut_batch_releases_all_of_its_smaller_shares() {
    let content = Box::new([tool_call(b"a", cat(b"file")), tool_call(b"b", cat(b"file"))]);
    let limits = exact_read_batch_limits(&content[..], true);
    let mut h = Harness::new(&limits);
    let (owner, _) = h.open(1);
    let (_, _, before_batch, _) = held(&h, owner);
    let runs = h.batch(Event::Completed { owner, completion: completion(content, Stop::ToolUse) });
    let &[a, b] = runs.as_slice() else { panic!("two cuts") };
    let share = 4095_u64;
    let caps = tools::result_bound(&cat(b"file"), &limits.tools).expect("read cap");
    assert!(share < caps);
    let paired = share.checked_mul(2).expect("two bounded cut shares");
    assert_eq!(held(&h, owner), (paired, None, paired, 0), "the smaller shares are reserved");
    assert_eq!(h.step(ran(b, &[b'b'; 4096])), None);
    assert_eq!(held(&h, owner), (share, None, share, 0));
    let (_, prompt) = calling(h.step(ran(a, &[b'a'; 4096])));
    let Purpose::Compaction { .. } = prompt.purpose else { panic!("history is full after the reads") };
    assert_eq!(held(&h, owner), (before_batch, Some(0), before_batch, 0));
    let turn = h.turn_records.get(0).expect("ordinary turn before compaction");
    let results = &turn.messages.last().expect("read results").content;
    assert_eq!(results.len(), 2);
    for block in results {
        let Block::ToolResult {
            result: Returned::Owned { outcome: Outcome::Read { content, window, cut, .. } }, ..
        } = block
        else {
            panic!("cut read result")
        };
        assert_eq!(content.len(), 4095);
        assert_eq!(*window, tools::BytesClamp::Clamped { asked: 4096, ceiling: 4095 });
        assert_eq!(*cut, tools::Cut::Line);
    }
}

#[test]
fn a_batch_answered_at_the_entrance_returns_its_share_before_the_next_read() {
    let mut h = Harness::new(&Limits { parallel_tools: 1, ..LIMITS });
    let mut selected = spec();
    selected.authority.grants.shell = true;
    let (owner, _) = h.open_with(1, selected);
    let before = held(&h, owner);
    let content = Box::new([
        tool_call(
            b"unknown",
            Call::Job {
                job: tools::JobNumber(17),
                wait: tools::Wait::Default,
                output: tools::Budget::Default,
                stop: false,
            },
        ),
        tool_call(b"read", cat(b"file")),
    ]);
    assert_eq!(h.step(Event::Completed { owner, completion: completion(content, Stop::ToolUse) }), None);
    assert_eq!(held(&h, owner), (0, None, 0, 0), "the unknown job has no outstanding share");
    let (read_run, op) = running(h.after_reclaim());
    assert_eq!(loading(&op), place(b"file"));
    let (_, prompt) = calling(h.step(ran(read_run, b"kept")));
    assert_eq!(
        prompt.messages.last().expect("settled result message").content.as_ref(),
        [
            Block::ToolResult {
                id: bytes(b"unknown"),
                result: Returned::Invalid { problem: Problem::UnknownJob { job: 17 } }
            },
            result(b"read", read(b"kept")),
        ]
    );
    assert_eq!(held(&h, owner), before);
}

#[test]
fn a_withheld_call_releases_its_share_and_leaves_a_yielded_session() {
    for closing in [false, true] {
        let mut h = Harness::new(&LIMITS);
        let (session, _) = h.open(1);
        let content = Box::new([delegated(b"parked", FINISH.ticket.raw(), Effect::Write)]);
        let Some(Request::Delegate { owner, .. }) =
            h.step(Event::Completed { owner: session, completion: completion(content, Stop::ToolUse) })
        else {
            panic!("opener call");
        };
        let credit = LIMITS.delegated_result_bytes;
        assert_eq!(held(&h, session), (credit, None, credit, 0));
        if closing {
            assert_eq!(h.step(Event::Close { session }), Some(Request::Withdraw { owner }));
        }
        let terminal = h.step(Event::AnswerWithheld { owner });
        assert_eq!(terminal, if closing { Some(ended(End::Closed, 1)) } else { None });
        assert_eq!(held(&h, session), (0, None, 0, 0));
        assert_eq!(h.step(Event::AnswerWithheld { owner }), None);
        let answer = &h.turn_records.last().expect("withheld turn").messages;
        assert_eq!(answer.len(), 2, "the saved assistant call has no result message");
        let assistant = answer.last().expect("assistant message");
        let block = assistant.content.first().expect("parked call block");
        assert_eq!(block.tool_call().expect("parked call").id.as_ref(), b"parked");
        if !closing {
            assert_eq!(
                h.step(Event::Close { session }),
                Some(ended(End::Closed, 1)),
                "yielded close has nothing left to withdraw"
            );
        }
    }
}

#[test]
fn failed_cancelled_and_refused_attempts_release_their_receiving_credit() {
    let mut h = Harness::new(&LIMITS);
    let (owner, _) = h.open(1);
    let before = held(&h, owner);
    assert!(before.0 > 0);
    assert_eq!(
        h.step(Event::Failed {
            owner,
            failure: Failure::Overloaded,
            evidence: Evidence::Unknown,
            detail: Box::default(),
            retry_ok: true,
        }),
        None
    );
    assert_eq!(held(&h, owner), (0, None, 0, 0), "backoff holds no receiving credit");
    h.env.now = h.domain.next_deadline().expect("retry is armed");
    drop(calling(h.fire()));
    assert_eq!(held(&h, owner), before, "the retry holds exactly its own credit");
    assert_eq!(h.step(Event::Close { session: owner }), Some(Request::Cancel { owner }));
    assert_eq!(held(&h, owner), before, "a cancel request is not a terminal");
    assert_eq!(h.step(Event::Cancelled { owner }), Some(ended(End::Closed, 0)));
    assert_eq!(held(&h, owner), (0, None, 0, 0));

    h.domain.reclaim();
    let (owner, _) = h.open(1);
    assert_eq!(held(&h, owner), before);
    assert_eq!(h.step(Event::Overflowed { owner, end: End::UsageOverflow }), Some(ended(End::UsageOverflow, 0)));
    assert_eq!(held(&h, owner), (0, None, 0, 0));

    h.domain.reclaim();
    let (owner, _) = h.open(1);
    let (run, _) = running(h.step(Event::Completed { owner, completion: reading() }));
    drop(calling(h.step(ran(run, b"file"))));
    assert_eq!(held(&h, owner), before);
    let (_, prompt) = calling(h.step(Event::Failed {
        owner,
        failure: Failure::ContextTooLong,
        evidence: Evidence::Unsent,
        detail: Box::default(),
        retry_ok: true,
    }));
    let Purpose::Compaction { .. } = prompt.purpose else { panic!("context refusal compacts") };
    assert_eq!(held(&h, owner), (0, Some(0), 0, 0), "compaction replaces the ordinary receiving credit");
}

#[test]
fn an_offer_with_no_transcript_place_waits_for_the_replacement_window() {
    let limits = Limits { history_messages: 4, ..LIMITS };
    let mut h = Harness::new(&limits);
    let (owner, _) = h.open(1);
    let before = held(&h, owner);
    let (run, _) = running(h.step(Event::Completed { owner, completion: reading() }));
    let (_, summary) = calling(h.step(ran(run, b"file")));
    let Purpose::Compaction { .. } = summary.purpose else { panic!("no place for another ordinary attempt") };
    assert_eq!(summary.messages.len(), 4, "the summary instruction takes the last place");
    assert_eq!(held(&h, owner), (0, Some(0), 0, 0));

    let offer = b"opener offer";
    crate::append_messages(&mut h.domain, &h.env, owner, bytes(offer), &mut h.out);
    let (_, reissued) = calling(h.one());
    assert_eq!(reissued.messages, summary.messages, "the offer occupies no place in the old window");
    assert_eq!(held(&h, owner), (0, Some(0), 0, 0), "replacing an unsent compaction holds one attempt");
    let (_, next) = calling(
        h.step(Event::Completed { owner, completion: completion(Box::new([text(b"summary")]), Stop::EndTurn) }),
    );
    let Purpose::Ordinary { .. } = next.purpose else { panic!("the replacement window can begin") };
    let opening = next.messages.first().expect("the replacement window opens with a message");
    assert!(opening.content.contains(&text(offer)));
    assert_eq!(held(&h, owner), before, "the replacement window admits a fresh receiving credit");
}

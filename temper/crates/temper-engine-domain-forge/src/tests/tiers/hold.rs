//! A person's hold and the independent task hold (after-landing.md, 3).
use super::*;
const PARTY: u64 = 19;
const TOKEN: Token = Token::new(91);
pub(super) fn act(d: &mut Domain, act: TierAct) -> Box<[Request]> {
    outputs(
        d,
        Event::TierAct {
            request: Token::new(90),
            token: TOKEN,
            party: PARTY,
            call: TierActRequest { project: 5, repository: None, tier: TIER.into(), act },
        },
    )
}
fn held(d: &mut Domain) {
    act(d, TierAct::Hold { reason: b"maintenance".as_slice().into() });
    assert_eq!(d.application_answer(TOKEN), Some(Answered::Tier(Ok(()))));
}
#[test]
fn a_persons_hold_blocks_a_baseline_until_release() {
    let mut d = restoring(None);
    held(&mut d);
    assert!(d.tier_ops.reads.is_empty(), "no baseline read while held");
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    assert!(d.tier_ops.reads.is_empty());
    seen(&mut d, Some(BASE), NEXT_TIP);
    assert!(row(&d).record.run.is_none());
    assert!(row(&d).baseline.is_none());
    assert_eq!(row(&d).hold, Some(TierHold { party: PARTY, at: env().wall, reason: b"maintenance".as_slice().into() }));
    act(&mut d, TierAct::Release);
    assert!(row(&d).hold.is_none());
    baseline_tip(&mut d, NEXT_TIP);
    assert_eq!(requests(&key_trees(&mut d)), [(1, NEXT_TIP)]);
}
#[test]
fn cancelling_a_person_held_tiers_run_drops_it_without_a_replacement() {
    let mut d = dispatched();
    held(&mut d);
    seen(&mut d, Some(BASE), NEXT_TIP);
    let out = close_tier(&mut d, TIER_TASK, ReleaseEnding::Cancelled);
    assert_eq!(requests(&out), []);
    assert!(row(&d).record.run.is_none());
    assert!(row(&d).task.is_none());
    assert_eq!(row(&d).record.sequence, 1);
    assert!(row(&d).hold.is_some());
    key_trees(&mut d);
    assert!(row(&d).record.run.is_none());
    act(&mut d, TierAct::Release);
    baseline_tip(&mut d, NEXT_TIP);
    assert_eq!(requests(&key_trees(&mut d)), [(2, NEXT_TIP)]);
}
#[test]
fn tier_acts_refuse_unknown_names_repeated_acts_and_invalid_reasons() {
    for (repository, tier, act_value, already, expected) in [
        (
            Some(DraftRepository { owner: b"missing".as_slice().into(), name: b"repo".as_slice().into() }),
            TIER,
            TierAct::Release,
            false,
            TierRefusal::NoRepository,
        ),
        (None, b"missing".as_slice(), TierAct::Release, false, TierRefusal::NoTier),
        (
            None,
            TIER,
            TierAct::Hold { reason: b"again".as_slice().into() },
            true,
            TierRefusal::Held { party: PARTY, reason: b"maintenance".as_slice().into() },
        ),
        (None, TIER, TierAct::Release, false, TierRefusal::NotHeld),
        (None, TIER, TierAct::Hold { reason: Box::new([]) }, false, TierRefusal::Words),
        (None, TIER, TierAct::Hold { reason: vec![b'a'; 1025].into_boxed_slice() }, false, TierRefusal::Words),
        (None, TIER, TierAct::Hold { reason: Box::new([255]) }, false, TierRefusal::Words),
    ] {
        let mut d = restoring(None);
        if already {
            held(&mut d);
        }
        let before = d.tiers.get(&(REPO, TIER.into())).cloned();
        let out = outputs(
            &mut d,
            Event::TierAct {
                request: Token::new(90),
                token: TOKEN,
                party: PARTY,
                call: TierActRequest { project: 5, repository, tier: tier.into(), act: act_value },
            },
        );
        assert_eq!(d.application_answer(TOKEN), Some(Answered::Tier(Err(expected))));
        assert_eq!(d.tiers.get(&(REPO, TIER.into())), before.as_ref());
        assert!(!out.iter().any(|request| matches!(request, Request::Save { .. })), "refusal writes nothing");
    }
    let mut d = restoring(None);
    for reason in [b"a".as_slice().to_vec(), vec![b'a'; 1024]] {
        act(&mut d, TierAct::Hold { reason: reason.into_boxed_slice() });
        assert_eq!(d.application_answer(TOKEN), Some(Answered::Tier(Ok(()))));
        act(&mut d, TierAct::Release);
    }
    held(&mut d);
    d.landing.projects.get_mut(&5).unwrap().1[0].tiers = Box::new([]);
    assert!(row(&d).hold.is_some(), "removing settings does not remove the record's hold");
    act(&mut d, TierAct::Release);
    assert_eq!(d.application_answer(TOKEN), Some(Answered::Tier(Ok(()))));
    assert!(row(&d).hold.is_none());
}

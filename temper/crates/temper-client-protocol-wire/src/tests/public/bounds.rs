//! Maximal visitor answers and entrance refusals (temper/docs/design/client/forge.md, 11.4).
use crate::{Bounds, public_bounds, public_limits, public_v1 as w};
use alloc::boxed::Box;
use skein_lib::{List, Reader, Writer};

const BOUNDS: Bounds = Bounds {
    segments: 3,
    segment_bytes: 8,
    gates: 2,
    approvals: 1,
    field_bytes: 16,
    checks: 2,
    contexts: 2,
    refusal_bytes: 64,
};

fn address() -> Box<[u8]> {
    Box::new([b'a'; 2048])
}

fn gate() -> w::PublicGate {
    w::PublicGate::new(
        &w::CEILINGS,
        w::PublicGateParts {
            number: u64::MAX,
            kind: w::PublicGateKind::Check,
            blocks: true,
            verdict: w::PublicVerdict::Changes,
        },
    )
    .unwrap()
}

fn used() -> w::PublicUsed {
    w::PublicUsed::new(&w::CEILINGS, w::PublicUsedParts { used: u32::MAX, limit: u32::MAX }).unwrap()
}

fn change(limits: &w::Limits) -> w::PublicChange {
    let mut gates = List::with_capacity(limits.public_change_gates);
    for _ in 0..limits.public_change_gates {
        gates.push(gate()).unwrap();
    }
    w::PublicChange::new(
        limits,
        w::PublicChangeParts {
            task: u16::MAX,
            repository: address(),
            pull: Some(address()),
            head: Some([255; 32]),
            step: w::PublicStep::Repairing,
            repair: Some(w::PublicRepair::Gate(
                w::PublicGateNumber::new(limits, w::PublicGateNumberParts { gate: u64::MAX }).unwrap(),
            )),
            merge: Some([255; 32]),
            tiers: u32::MAX,
            hold: Some(w::PublicHold::Authority),
            ci: w::PublicCi::Failed,
            gates,
            repairs: used(),
            resolutions: used(),
            updates: used(),
            way: w::PublicWay::Turn,
            in_landing: true,
        },
    )
    .unwrap()
}

fn line(limits: &w::Limits) -> w::PublicLandingLine {
    let mut members = List::with_capacity(limits.public_landing_line_members);
    for _ in 0..limits.public_landing_line_members {
        members.push(u16::MAX).unwrap();
    }
    let mut landed = List::with_capacity(limits.public_landing_line_landed);
    for _ in 0..limits.public_landing_line_landed {
        landed.push([255; 32]).unwrap();
    }
    w::PublicLandingLine::new(
        limits,
        w::PublicLandingLineParts { members, phase: w::PublicLandingPhase::Landed, landed },
    )
    .unwrap()
}

fn queued() -> w::PublicQueued {
    w::PublicQueued::new(&w::CEILINGS, w::PublicQueuedParts { task: u16::MAX, goal: u16::MAX, ready_since: u64::MAX })
        .unwrap()
}

fn queue(limits: &w::Limits) -> w::PublicQueue {
    let mut changes = List::with_capacity(limits.public_queue_changes);
    for _ in 0..limits.public_queue_changes {
        changes.push(queued()).unwrap();
    }
    w::PublicQueue::new(limits, w::PublicQueueParts { repository: address(), changes }).unwrap()
}

fn landings(limits: &w::Limits) -> w::PublicLandings {
    let mut landings = List::with_capacity(limits.public_landings_landings);
    for _ in 0..limits.public_landings_landings {
        landings.push(line(limits)).unwrap();
    }
    let mut queues = List::with_capacity(limits.public_landings_queues);
    for _ in 0..limits.public_landings_queues {
        queues.push(queue(limits)).unwrap();
    }
    w::PublicLandings::new(limits, w::PublicLandingsParts { turn: Some(u16::MAX), landings, queues }).unwrap()
}

fn proof() -> w::PublicGateProof {
    w::PublicGateProof::new(
        &w::CEILINGS,
        w::PublicGateProofParts {
            number: u64::MAX,
            kind: w::PublicGateKind::Check,
            blocks: true,
            proof: w::PublicProof::Failed,
            run: Some(address()),
        },
    )
    .unwrap()
}

fn member(limits: &w::Limits) -> w::PublicMember {
    let mut gates = List::with_capacity(limits.public_member_gates);
    for _ in 0..limits.public_member_gates {
        gates.push(proof()).unwrap();
    }
    w::PublicMember::new(
        limits,
        w::PublicMemberParts {
            task: u16::MAX,
            repository: address(),
            pull: Some(address()),
            base: Some([255; 32]),
            candidate: Some([255; 32]),
            landed: Some([255; 32]),
            gates,
        },
    )
    .unwrap()
}

fn landing(limits: &w::Limits) -> w::PublicLanding {
    let mut members = List::with_capacity(limits.public_landing_members);
    for _ in 0..limits.public_landing_members {
        members.push(member(limits)).unwrap();
    }
    w::PublicLanding::new(
        limits,
        w::PublicLandingParts {
            phase: w::PublicLandingPhase::Held,
            hold: Some(w::PublicLandingHold::Faults),
            preparation: u32::MAX,
            members,
            stale_proofs: u32::MAX,
            holds_turn: true,
        },
    )
    .unwrap()
}

pub(super) fn largest_answers() {
    let limits = public_limits(&BOUNDS).unwrap();
    let [first_bound, list_bound, detail_bound] = public_bounds(&BOUNDS);
    let change = change(&limits);
    let mut writer = Writer::new(usize::try_from(first_bound).unwrap());
    change.encode(&mut writer).unwrap();
    let bytes = writer.finish();
    assert_eq!(bytes.len(), usize::try_from(first_bound).unwrap());
    assert_eq!(w::PublicChange::decode(&limits, &mut Reader::new(&bytes)), Ok(change.clone()));
    refuse_change_overages(&change, &bytes, limits);

    let landings = landings(&limits);
    let mut writer = Writer::new(usize::try_from(list_bound).unwrap());
    landings.encode(&mut writer).unwrap();
    let bytes = writer.finish();
    assert_eq!(bytes.len(), usize::try_from(list_bound).unwrap());
    assert_eq!(w::PublicLandings::decode(&limits, &mut Reader::new(&bytes)), Ok(landings.clone()));
    refuse_landings_overages(&landings, &bytes, limits);

    let landing = landing(&limits);
    let mut writer = Writer::new(usize::try_from(detail_bound).unwrap());
    landing.encode(&mut writer).unwrap();
    let bytes = writer.finish();
    assert_eq!(bytes.len(), usize::try_from(detail_bound).unwrap());
    assert_eq!(w::PublicLanding::decode(&limits, &mut Reader::new(&bytes)), Ok(landing.clone()));
    refuse_landing_overages(&landing, &bytes, limits);
}

// A maximal value is one entry/byte beyond a receiver whose corresponding limit is one smaller.
fn refuse_change_overages(value: &w::PublicChange, bytes: &[u8], limits: w::Limits) {
    for smaller in [
        w::Limits { public_change_gates: limits.public_change_gates.checked_sub(1).unwrap(), ..limits },
        w::Limits { public_change_repository: limits.public_change_repository.checked_sub(1).unwrap(), ..limits },
        w::Limits { public_change_pull_some: limits.public_change_pull_some.checked_sub(1).unwrap(), ..limits },
    ] {
        assert_eq!(
            w::PublicChange::new(&smaller, value.clone().into_parts()).unwrap_err().reason,
            skein_codec::Reason::Bound
        );
        assert_eq!(
            w::PublicChange::decode(&smaller, &mut Reader::new(bytes)).unwrap_err().reason,
            skein_codec::Reason::Bound
        );
    }
}

fn refuse_landings_overages(value: &w::PublicLandings, bytes: &[u8], limits: w::Limits) {
    for smaller in [
        w::Limits { public_landing_line_members: limits.public_landing_line_members.checked_sub(1).unwrap(), ..limits },
        w::Limits { public_landing_line_landed: limits.public_landing_line_landed.checked_sub(1).unwrap(), ..limits },
        w::Limits { public_queue_repository: limits.public_queue_repository.checked_sub(1).unwrap(), ..limits },
        w::Limits { public_queue_changes: limits.public_queue_changes.checked_sub(1).unwrap(), ..limits },
        w::Limits { public_landings_landings: limits.public_landings_landings.checked_sub(1).unwrap(), ..limits },
        w::Limits { public_landings_queues: limits.public_landings_queues.checked_sub(1).unwrap(), ..limits },
    ] {
        assert_eq!(
            w::PublicLandings::new(&smaller, value.clone().into_parts()).unwrap_err().reason,
            skein_codec::Reason::Bound
        );
        assert_eq!(
            w::PublicLandings::decode(&smaller, &mut Reader::new(bytes)).unwrap_err().reason,
            skein_codec::Reason::Bound
        );
    }
}

fn refuse_landing_overages(value: &w::PublicLanding, bytes: &[u8], limits: w::Limits) {
    for smaller in [
        w::Limits { public_member_repository: limits.public_member_repository.checked_sub(1).unwrap(), ..limits },
        w::Limits { public_member_pull_some: limits.public_member_pull_some.checked_sub(1).unwrap(), ..limits },
        w::Limits { public_member_gates: limits.public_member_gates.checked_sub(1).unwrap(), ..limits },
        w::Limits { public_landing_members: limits.public_landing_members.checked_sub(1).unwrap(), ..limits },
        w::Limits { public_gate_proof_run_some: limits.public_gate_proof_run_some.checked_sub(1).unwrap(), ..limits },
    ] {
        assert_eq!(
            w::PublicLanding::new(&smaller, value.clone().into_parts()).unwrap_err().reason,
            skein_codec::Reason::Bound
        );
        assert_eq!(
            w::PublicLanding::decode(&smaller, &mut Reader::new(bytes)).unwrap_err().reason,
            skein_codec::Reason::Bound
        );
    }
}

#[test]
fn invalid_panel_bounds_are_refused() {
    let invalid = Bounds { segments: u32::MAX, ..BOUNDS };
    assert!(public_limits(&invalid).is_none());
    assert_eq!(public_bounds(&invalid), [0; 3]);
}

#[test]
fn public_limits_follow_the_panels() {
    let limits = public_limits(&BOUNDS).unwrap();
    assert_eq!(limits, w::CEILINGS);
    let panels = crate::v1::CEILINGS;
    let ceilings = Bounds {
        segments: panels.landing_pattern_segments,
        segment_bytes: panels.landing_pattern_segments_item,
        gates: panels.landing_rule_gates,
        approvals: panels.landing_rule_approvals,
        field_bytes: panels.adopt_owner,
        checks: panels.adopt_checks,
        contexts: panels.rule_contexts,
        refusal_bytes: panels.core_refused_refusal,
    };
    assert_eq!(public_limits(&ceilings), Some(limits));
    assert_eq!(public_bounds(&ceilings), public_bounds(&BOUNDS));
    assert_eq!(
        (crate::PUBLIC_VERSION, crate::PUBLIC_CHANGE, crate::PUBLIC_LANDINGS, crate::PUBLIC_LANDING),
        (1, 1, 2, 3)
    );
    assert_eq!(public_bounds(&BOUNDS), [4927, 198_733, 165_794]);
}

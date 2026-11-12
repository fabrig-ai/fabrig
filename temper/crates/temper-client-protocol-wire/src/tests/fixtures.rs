use crate::{Bounds, v1};
use alloc::boxed::Box;
use skein_lib::List;

pub const BOUNDS: Bounds = Bounds {
    segments: 3,
    segment_bytes: 8,
    gates: 2,
    approvals: 1,
    field_bytes: 16,
    checks: 2,
    contexts: 2,
    refusal_bytes: 64,
};

// Encode deliberately unchecked rules in the final fields of a smallest panel.
// The panel's suffix is PathRules (two counts, both zero), then may_change (one byte).
pub fn ways_with_unchecked_rules(panel: &v1::WaysPanel, rows: u32, row: &v1::PathRuleParts, note: &[u8]) -> Box<[u8]> {
    assert!(panel.rules().rows().is_empty());
    assert_eq!(panel.rules().note(), []);
    let bytes = crate::ways_panel(panel);
    let prefix = bytes.get(..bytes.len().checked_sub(9).expect("rules and permission suffix")).expect("prefix");
    let fields = [&row.paths, &row.review, &row.sign_off, &row.lands];
    let mut row_bytes = 0_usize;
    for field in fields {
        row_bytes = row_bytes.checked_add(4).unwrap().checked_add(field.len()).unwrap();
    }
    let size = usize::try_from(rows)
        .unwrap()
        .checked_mul(row_bytes)
        .unwrap()
        .checked_add(prefix.len())
        .unwrap()
        .checked_add(9)
        .unwrap()
        .checked_add(note.len())
        .unwrap();
    let mut writer = skein_lib::Writer::new(size);
    writer.put(prefix).expect("panel prefix");
    writer.put(&rows.to_be_bytes()).expect("row count");
    for _ in 0..rows {
        for field in fields {
            writer.put(&u32::try_from(field.len()).unwrap().to_be_bytes()).expect("field length");
            writer.put(field).expect("field bytes");
        }
    }
    writer.put(&u32::try_from(note.len()).unwrap().to_be_bytes()).expect("note length");
    writer.put(note).expect("note bytes");
    writer.put(&[u8::from(panel.may_change())]).expect("permission");
    writer.finish()
}

pub fn path_rules(
    limits: &v1::Limits,
    rows: u32,
    row: v1::PathRuleParts,
    note: Box<[u8]>,
) -> Result<v1::PathRules, v1::Problem> {
    let row = v1::PathRule::new(limits, row)?;
    let mut items = List::with_capacity(rows);
    for _ in 0..rows {
        items.push(row.clone()).expect("room for rows");
    }
    v1::PathRules::new(limits, v1::PathRulesParts { rows: items, note })
}

pub fn limits() -> v1::Limits {
    crate::limits(&BOUNDS).expect("test bounds within ceilings")
}

pub fn adopt() -> v1::Adopt {
    v1::Adopt::new(
        &limits(),
        v1::AdoptParts {
            project: 1,
            home: true,
            forge: crate::FORGE,
            owner: Box::from(&b"acme"[..]),
            name: Box::from(&b"shop"[..]),
            prefix: Box::from(&b"temper/"[..]),
            role: v1::Role::Owned,
            landing: Box::from(&b"main"[..]),
            ci: true,
            checks: List::with_capacity(2),
        },
    )
    .expect("default adoption")
}

pub fn adopted() -> v1::Adopted {
    let limits = limits();
    let kinds = v1::Kinds::new(&limits, v1::KindsParts { skein_bools: [true; 9] }).expect("observed capabilities");
    v1::Adopted::new(
        &limits,
        v1::AdoptedParts {
            forge: crate::FORGE,
            repository: 7,
            owner: Box::from(&b"acme"[..]),
            name: Box::from(&b"shop"[..]),
            role: v1::Role::Owned,
            home: true,
            kinds,
            protection: v1::Protection::Unknown,
        },
    )
    .expect("adopted repository")
}

pub fn outcome_rows() -> [v1::Outcome; 20] {
    let limits = limits();
    let original = adopted().into_parts();
    let mut contexts = List::with_capacity(1);
    contexts.push(Box::from(&b"build"[..])).expect("one context");
    let rule = v1::Rule::new(
        &limits,
        v1::RuleParts { branch: Box::from(&b"main"[..]), contexts, approvals: 1, dismiss_stale: true },
    )
    .expect("protection rule");
    let absent = v1::Adopted::new(&limits, v1::AdoptedParts { protection: v1::Protection::Absent, ..original.clone() })
        .expect("no protection");
    let protected = v1::Adopted::new(&limits, v1::AdoptedParts { protection: v1::Protection::Rule(rule), ..original })
        .expect("protection observed");
    let refusal = v1::CoreRefused::new(&limits, v1::CoreRefusedParts { refusal: Box::from(&b"core refusal"[..]) })
        .expect("core refusal");
    [
        v1::Outcome::Adopted(adopted()),
        v1::Outcome::Adopted(absent),
        v1::Outcome::Adopted(protected),
        v1::Outcome::CoreRefused(refusal),
        forge_refused(v1::ForgeReason::NoSuchForge),
        forge_refused(v1::ForgeReason::NotFound),
        forge_refused(v1::ForgeReason::NoAccess),
        forge_refused(v1::ForgeReason::Refused),
        forge_refused(v1::ForgeReason::InFlight),
        forge_refused(v1::ForgeReason::TooLarge),
        forge_refused(v1::ForgeReason::Unreachable),
        forge_refused(v1::ForgeReason::InvalidAnswer),
        forge_refused(v1::ForgeReason::AnotherProject),
        forge_refused(v1::ForgeReason::HomeHeld),
        forge_refused(v1::ForgeReason::PrefixOccupied),
        v1::Outcome::Unknown,
        v1::Outcome::SettingUp,
        set_up_refused(v1::SetUpRefusal::NotOffered),
        set_up_refused(v1::SetUpRefusal::NotUnderTheBox),
        set_up_refused(v1::SetUpRefusal::Busy),
    ]
}

fn forge_refused(reason: v1::ForgeReason) -> v1::Outcome {
    v1::Outcome::ForgeRefused(
        v1::ForgeRefused::new(&limits(), v1::ForgeRefusedParts { reason }).expect("forge refusal"),
    )
}

pub fn pull(address_bytes: u32, limits: &v1::Limits) -> v1::Pull {
    v1::Pull::new(limits, v1::PullParts { number: 23, address: field_bytes(address_bytes) })
        .expect("pull within limits")
}

pub fn queued(place: u32, count: u32, limits: &v1::Limits) -> v1::Queued {
    let mut ahead = List::with_capacity(count);
    for task in 1..=count {
        ahead.push(u64::from(task)).expect("counted tasks");
    }
    v1::Queued::new(limits, v1::QueuedParts { place, ahead }).expect("queue within limits")
}

pub fn gates(count: u32) -> List<v1::Gate> {
    let mut gates = List::with_capacity(count);
    for number in 0..count {
        gates
            .push(
                v1::Gate::new(
                    &limits(),
                    v1::GateParts {
                        number: u64::from(number) + 1,
                        kind: v1::GateKind::Review,
                        blocks: true,
                        verdict: v1::Verdict::Pending,
                    },
                )
                .expect("gate"),
            )
            .expect("counted gates");
    }
    gates
}

pub fn panel_parts() -> v1::ChangePanelParts {
    let limits = limits();
    v1::ChangePanelParts {
        way: v1::Way::Queue,
        in_landing: None,
        task: 17,
        forge: crate::FORGE,
        owner: b"acme".as_slice().into(),
        name: b"shop".as_slice().into(),
        branch: b"change/17".as_slice().into(),
        landing: b"main".as_slice().into(),
        head: None,
        pull: None,
        step: v1::Step::Producing,
        hold: None,
        runs: None,
        ci_required: true,
        ci: v1::Ci::Unknown,
        approvals: List::with_capacity(0),
        unmapped_reviews: List::with_capacity(0),
        gates: gates(0),
        repairs: v1::Used::new(&limits, v1::UsedParts { used: 1, limit: 2 }).expect("repairs"),
        resolutions: v1::Used::new(&limits, v1::UsedParts { used: 3, limit: 4 }).expect("resolutions"),
        updates: v1::Used::new(&limits, v1::UsedParts { used: 5, limit: 6 }).expect("updates"),
        questions: questions(),
    }
}

#[expect(clippy::too_many_lines, reason = "table covers every step and the optional panel facts")]
pub fn panel_rows() -> List<v1::ChangePanel> {
    use v1::{Ci, Hold, Reason, Step};
    let limits = limits();
    let gate = Reason::Gate(v1::GateReason::new(&limits, v1::GateReasonParts { gate: 9 }).expect("gate reason"));
    let mut panels = List::uncapped();
    for (step, hold, ci, [head, pull, runs]) in [
        (
            Step::Ready(v1::InLanding::new(&limits, v1::InLandingParts { landing: None }).unwrap()),
            None,
            Ci::Passed,
            [true, true, true],
        ),
        (
            Step::Ready(v1::InLanding::new(&limits, v1::InLandingParts { landing: Some(61) }).unwrap()),
            None,
            Ci::Passed,
            [true, true, true],
        ),
        (
            Step::Preparing(v1::InLanding::new(&limits, v1::InLandingParts { landing: Some(61) }).unwrap()),
            None,
            Ci::Passed,
            [true, true, true],
        ),
        (Step::Producing, None, Ci::Unknown, [false, false, false]),
        (Step::Opening, Some(Hold::Stalled), Ci::Pending, [true, false, false]),
        (Step::Recreating, Some(Hold::Repairs), Ci::Passed, [false, true, false]),
        (Step::Recreating, Some(Hold::Questions), Ci::Passed, [false, true, false]),
        (Step::Reopening, Some(Hold::Resolutions), Ci::Failed, [false, false, true]),
        (Step::Checking, Some(Hold::Updates), Ci::Unknown, [true, true, false]),
        (Step::Gating, Some(Hold::Moved), Ci::Pending, [true, false, true]),
        (Step::Queued(queued(1, 2, &limits)), Some(Hold::Missing), Ci::Passed, [false, true, true]),
        (Step::First, Some(Hold::Closed), Ci::Failed, [true, true, true]),
        (Step::Updating, Some(Hold::Retargeted), Ci::Unknown, [true, true, true]),
        (Step::Resolving, Some(Hold::Failed), Ci::Pending, [true, true, true]),
        (
            Step::Repairing(v1::Repairing::new(&limits, v1::RepairingParts { reason: Reason::Ci }).expect("CI repair")),
            Some(Hold::Rejected),
            Ci::Failed,
            [true, true, true],
        ),
        (
            Step::Repairing(v1::Repairing::new(&limits, v1::RepairingParts { reason: gate }).expect("gate repair")),
            None,
            Ci::Passed,
            [true, true, true],
        ),
        (
            Step::Repairing(
                v1::Repairing::new(&limits, v1::RepairingParts { reason: Reason::Semantic }).expect("semantic repair"),
            ),
            None,
            Ci::Passed,
            [true, true, true],
        ),
        (
            Step::Repairing(
                v1::Repairing::new(&limits, v1::RepairingParts { reason: Reason::Review }).expect("review repair"),
            ),
            None,
            Ci::Passed,
            [true, true, true],
        ),
        (Step::Landing, None, Ci::Passed, [true, true, true]),
        (
            Step::Verifying(
                v1::Verifying::new(&limits, v1::VerifyingParts { merge: [8; 32], tiers: tier_lines() })
                    .expect("verifying"),
            ),
            None,
            Ci::Passed,
            [true; 3],
        ),
        (
            Step::Red(v1::Red::new(&limits, v1::RedParts { merge: [8; 32], tiers: tier_lines() }).expect("red")),
            None,
            Ci::Passed,
            [true; 3],
        ),
        (
            Step::Cleared(
                v1::Cleared::new(&limits, v1::ClearedParts { merge: [8; 32], tiers: tier_lines(), uncleared: false })
                    .expect("cleared"),
            ),
            None,
            Ci::Passed,
            [true; 3],
        ),
        (
            Step::Cleared(
                v1::Cleared::new(&limits, v1::ClearedParts { merge: [8; 32], tiers: tier_lines(), uncleared: true })
                    .expect("uncleared"),
            ),
            None,
            Ci::Passed,
            [true; 3],
        ),
        (
            Step::Landed(v1::Landed::new(&limits, v1::LandedParts { merge: [0x47; 32] }).expect("merge")),
            None,
            Ci::Passed,
            [true, true, true],
        ),
    ] {
        let mut parts = panel_parts();
        match &step {
            Step::Ready(member) | Step::Preparing(member) => {
                parts.way = v1::Way::Turn;
                parts.in_landing = *member.landing();
            }
            Step::Producing
            | Step::Opening
            | Step::Recreating
            | Step::Reopening
            | Step::Checking
            | Step::Gating
            | Step::Queued(_)
            | Step::First
            | Step::Updating
            | Step::Resolving
            | Step::Repairing(_)
            | Step::Landing
            | Step::Ended
            | Step::Landed(_)
            | Step::Verifying(_)
            | Step::Red(_)
            | Step::Cleared(_) => {}
        }
        parts.step = step;
        parts.hold = hold;
        parts.ci = ci;
        if head {
            parts.head = Some([0x39; 32]);
        }
        if pull {
            parts.pull = Some(self::pull(32, &limits));
        }
        if runs {
            parts.runs = Some(29);
        }
        parts.gates = List::with_capacity(12);
        for kind in [v1::GateKind::Review, v1::GateKind::Approval, v1::GateKind::Check] {
            for verdict in [v1::Verdict::None, v1::Verdict::Pending, v1::Verdict::Approved, v1::Verdict::Changes] {
                // Both blocking and advisory gates, every kind and verdict.
                parts
                    .gates
                    .push(
                        v1::Gate::new(
                            &limits,
                            v1::GateParts {
                                number: u64::from(parts.gates.len()) + 1,
                                kind: kind.clone(),
                                blocks: verdict == v1::Verdict::Approved,
                                verdict,
                            },
                        )
                        .expect("gate"),
                    )
                    .expect("twelve gates");
            }
        }
        panels.push(v1::ChangePanel::new(&limits, parts).expect("panel row")).expect("panel rows");
    }
    panels
}

pub fn panel_at_bound(field: PanelBound, count: u32) -> Result<v1::ChangePanel, v1::Problem> {
    let limits = limits();
    let mut parts = panel_parts();
    match field {
        PanelBound::Address => {
            parts.pull = Some(v1::Pull::new(&limits, v1::PullParts { number: 23, address: field_bytes(count) })?);
        }
        PanelBound::Ahead => {
            let mut ahead = List::with_capacity(count);
            for task in 1..=count {
                ahead.push(u64::from(task)).expect("counted tasks");
            }
            parts.step = v1::Step::Queued(v1::Queued::new(&limits, v1::QueuedParts { place: 1, ahead })?);
        }
        PanelBound::Gates => parts.gates = gates(count),
        PanelBound::Approvals | PanelBound::Counted => parts.approvals = approval_lines(field, count)?,
        PanelBound::Unmapped => {
            parts.unmapped_reviews = List::with_capacity(count);
            for account in 0..count {
                parts
                    .unmapped_reviews
                    .push(v1::UnmappedReview::new(
                        &limits,
                        v1::UnmappedReviewParts {
                            account: u64::from(account),
                            head: [1; 32],
                            verdict: v1::Verdict::Approved,
                        },
                    )?)
                    .expect("review slots");
            }
        }
        PanelBound::Owner => {
            parts.owner = field_bytes(count);
        }
        PanelBound::Name => {
            parts.name = field_bytes(count);
        }
        PanelBound::Branch => {
            parts.branch = field_bytes(count);
        }
        PanelBound::Landing => {
            parts.landing = field_bytes(count);
        }
    }
    v1::ChangePanel::new(&limits, parts)
}

#[derive(Clone, Copy, Debug)]
pub enum PanelBound {
    Address,
    Ahead,
    Gates,
    Approvals,
    Counted,
    Unmapped,
    Owner,
    Name,
    Branch,
    Landing,
}

pub fn field_bytes(count: u32) -> Box<[u8]> {
    let mut bytes = skein_lib::bytes::zeroed(usize::try_from(count).expect("field length fits usize"));
    bytes.fill(b'a');
    bytes
}

pub fn trailing(bytes: &[u8]) -> Box<[u8]> {
    let mut writer = skein_lib::Writer::new(bytes.len().checked_add(1).expect("one trailing byte fits"));
    writer.put(bytes).expect("record room");
    writer.put(&[0]).expect("trailing byte room");
    writer.finish()
}

fn set_up_refused(why: v1::SetUpRefusal) -> v1::Outcome {
    v1::Outcome::SetUpRefused(v1::SetUpRefused::new(&limits(), v1::SetUpRefusedParts { why }).expect("setup refusal"))
}

fn tier_lines() -> List<Box<[u8]>> {
    let mut lines = List::with_capacity(1);
    lines.push(Box::from(b"fast run 2".as_slice())).expect("one line");
    lines
}

pub fn questions() -> v1::Questions {
    let limits = limits();
    let mut open = List::with_capacity(5);
    for (index, stands) in [v1::Stands::Taken, v1::Stands::Asked, v1::Stands::Held].into_iter().enumerate() {
        open.push(
            v1::Question::new(
                &limits,
                v1::QuestionParts {
                    asker: 71,
                    number: u64::try_from(index.checked_add(1).expect("question index fits")).expect("question number"),
                    line: Box::from(b"Which branch?".as_slice()),
                    stands,
                },
            )
            .expect("question"),
        )
        .expect("five stands fit");
    }
    v1::Questions::new(
        &limits,
        v1::QuestionsParts {
            route: v1::Route::Up,
            taken: v1::Used::new(&limits, v1::UsedParts { used: 2, limit: 4 }).expect("questions taken"),
            open,
        },
    )
    .expect("questions")
}

#[test]
fn question_bounds_accept_at_and_refuse_one_past() {
    let limits = limits();
    let base = v1::QuestionParts { asker: 7, number: 1, line: Box::new([b'q'; 256]), stands: v1::Stands::Taken };
    let at = v1::Question::new(&limits, base.clone()).expect("line at bound");
    let past = v1::Question::new(&limits, v1::QuestionParts { line: Box::new([b'q'; 257]), ..base });
    assert_eq!(
        past.expect_err("line past bound"),
        v1::Problem { path: v1::Path::QuestionLine, reason: skein_codec::Reason::Bound }
    );
    for count in [8, 9] {
        let mut open = List::with_capacity(count);
        for _ in 0..count {
            open.push(at.clone()).expect("counted questions");
        }
        let made = v1::Questions::new(&limits, v1::QuestionsParts { open, ..questions().into_parts() });
        if count == 8 {
            made.expect("open questions at bound");
        } else {
            assert_eq!(
                made.expect_err("one question past bound"),
                v1::Problem { path: v1::Path::QuestionsOpen, reason: skein_codec::Reason::Bound }
            );
        }
    }
}

fn approval_lines(field: PanelBound, count: u32) -> Result<List<v1::Approval>, v1::Problem> {
    let limits = limits();
    let (lines, reviews) = match field {
        PanelBound::Counted => (1, count),
        PanelBound::Address
        | PanelBound::Ahead
        | PanelBound::Gates
        | PanelBound::Approvals
        | PanelBound::Unmapped
        | PanelBound::Owner
        | PanelBound::Name
        | PanelBound::Branch
        | PanelBound::Landing => (count, 0),
    };
    let mut approvals = List::with_capacity(lines);
    for number in 0..lines {
        let mut counted = List::with_capacity(reviews);
        for person in 0..reviews {
            counted
                .push(v1::CountedReview::new(
                    &limits,
                    v1::CountedReviewParts { person: u64::from(person), head: [1; 32] },
                )?)
                .expect("person slots");
        }
        approvals
            .push(v1::Approval::new(
                &limits,
                v1::ApprovalParts {
                    scope: 0,
                    number,
                    role: 0,
                    freshness: v1::RuleFreshness::Clean,
                    required: 1,
                    read: v1::ReviewRead::Complete,
                    counted,
                    remaining: Some(1),
                    refused: false,
                },
            )?)
            .expect("rule slots");
    }
    Ok(approvals)
}

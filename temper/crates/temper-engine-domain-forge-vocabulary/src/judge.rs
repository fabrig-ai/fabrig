//! Forge judgements over a change's own facts (domain/forge/change/gates.md, 2).
//!
//! The connector judges the criteria of its held rules. The root supplies verified
//! project roles for authenticated reviewers; it does not interpret the forge's
//! CI, containment, gate or review facts. Judgements are for one named head.

use alloc::boxed::Box;
use skein_lib::List;
use temper_engine_domain_forge_change as change;

use crate::{ChangeEvidence, Repository};

/// Which prior head can still carry a gate or review verdict.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Freshness {
    /// The report names the landing head itself.
    Exact,
    /// The report names the head or a verified clean predecessor.
    Clean,
}

/// Parameters for one forge requirement, retained by the connector.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Criterion {
    /// CI at the named head, or all adopted checks where CI is unavailable.
    Ci,
    /// The change contains the observed base tip.
    UpToDate,
    /// One blocking check or agent gate.
    Gate { number: u32, freshness: Freshness },
    /// Distinct approvals by authenticated people in a project role.
    Approval { role: u32, people: u32, freshness: Freshness },
}

/// Verdict returned to the root for one criterion.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Verdict {
    /// The criterion holds at this head.
    Met,
    /// Facts are absent, incomplete or pending.
    Wait,
    /// A required fact failed.
    Refuse,
}

/// A reviewer identity and project role verified by the root's people child.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Reviewer {
    pub at: skein_lib::Wall,
    pub person: u64,
    pub role: u32,
    pub head: skein_forge::api::Commit,
    pub verdict: skein_forge::api::Verdict,
}

fn status(value: change::Status) -> Verdict {
    match value {
        change::Status::Unknown | change::Status::Pending => Verdict::Wait,
        change::Status::Passed => Verdict::Met,
        change::Status::Failed => Verdict::Refuse,
    }
}

fn valid(
    report: skein_forge::api::Commit,
    head: skein_forge::api::Commit,
    clean: &[skein_forge::api::Commit],
    freshness: Freshness,
) -> bool {
    if report == head {
        return true;
    }
    match freshness {
        Freshness::Exact => false,
        Freshness::Clean => clean.contains(&report),
    }
}

fn gate(
    number: u32,
    freshness: Freshness,
    head: skein_forge::api::Commit,
    clean: &[skein_forge::api::Commit],
    evidence: &ChangeEvidence,
) -> Verdict {
    let mut met = false;
    let mut waiting = false;
    for report in &evidence.gates {
        if report.number != u64::from(number) || !valid(report.head, head, clean, freshness) {
            continue;
        }
        match status(report.status) {
            Verdict::Met => met = true,
            Verdict::Wait => waiting = true,
            Verdict::Refuse => return Verdict::Refuse,
        }
    }
    if waiting || !met { Verdict::Wait } else { Verdict::Met }
}

fn approval(
    role: u32,
    people: u32,
    freshness: Freshness,
    head: skein_forge::api::Commit,
    clean: &[skein_forge::api::Commit],
    reviewers: &[Reviewer],
    complete: bool,
) -> Verdict {
    if !complete {
        return Verdict::Wait;
    }
    let (counted, refused) = approval_count(role, freshness, head, clean, reviewers);
    if refused.is_some() {
        Verdict::Refuse
    } else if skein_lib::len_u64(counted.len()) < u64::from(people) {
        Verdict::Wait
    } else {
        Verdict::Met
    }
}

/// Count each qualifying person once, using the same conflicts and heads as enforcement.
pub fn approval_count(
    role: u32,
    freshness: Freshness,
    head: skein_forge::api::Commit,
    clean: &[skein_forge::api::Commit],
    reviewers: &[Reviewer],
) -> (Box<[Reviewer]>, Option<skein_forge::api::Commit>) {
    let mut counted = List::uncapped();
    for reviewer in reviewers {
        if reviewer.role != role || !valid(reviewer.head, head, clean, freshness) {
            continue;
        }
        match reviewer.verdict {
            skein_forge::api::Verdict::RequestChanges | skein_forge::api::Verdict::Comment => {}
            skein_forge::api::Verdict::Approve => {
                let mut conflicting = false;
                for other in reviewers {
                    if other.person == reviewer.person
                        && other.role == role
                        && valid(other.head, head, clean, freshness)
                        && other.verdict != skein_forge::api::Verdict::Approve
                    {
                        conflicting = true;
                    }
                }
                let mut duplicate = false;
                for earlier in &counted {
                    let earlier: &Reviewer = earlier;
                    duplicate |= earlier.person == reviewer.person;
                }
                if !conflicting && !duplicate {
                    counted.push(*reviewer).expect("admitted reviewers");
                }
            }
        }
    }
    // Prefer the change's head, then the latest clean predecessor in the
    // change's order, independent of the order the forge returned reviewers.
    for candidate in core::iter::once(&head).chain(clean.iter().rev()) {
        for reviewer in reviewers {
            if reviewer.role == role
                && reviewer.head == *candidate
                && valid(reviewer.head, head, clean, freshness)
                && reviewer.verdict == skein_forge::api::Verdict::RequestChanges
            {
                return (counted.into_boxed(), Some(*candidate));
            }
        }
    }
    (counted.into_boxed(), None)
}

/// Union exactly the counted people, retaining their counted review and the earliest eight.
#[must_use]
pub fn sign_offs(
    approvals: &[(u32, Freshness)],
    head: skein_forge::api::Commit,
    clean: &[skein_forge::api::Commit],
    reviewers: &[Reviewer],
) -> Box<[crate::SignOff]> {
    let mut kept = skein_lib::Map::uncapped();
    for (role, freshness) in approvals {
        let (counted, _) = approval_count(*role, *freshness, head, clean, reviewers);
        for review in counted {
            if !kept.contains_key(&review.person) {
                kept.insert(review.person, crate::SignOff { person: review.person, head: review.head, at: review.at })
                    .expect("admitted reviewers");
            }
        }
    }
    let mut ordered = skein_lib::Map::uncapped();
    for (_, signed) in &kept {
        ordered.insert((signed.at, signed.person), *signed).expect("distinct people");
    }
    let mut signed = List::uncapped();
    for (_, review) in &ordered {
        if skein_lib::usize_of(signed.len()) == crate::SIGN_OFFS {
            break;
        }
        signed.push(*review).expect("bounded sign-offs");
    }
    signed.into_boxed()
}

/// Judge one configured criterion against the change's coherent evidence.
#[must_use]
pub fn judge(
    criterion: Criterion,
    repository: &Repository,
    head: skein_forge::api::Commit,
    clean: &[skein_forge::api::Commit],
    evidence: &ChangeEvidence,
    reviewers: &[Reviewer],
) -> Verdict {
    if evidence.head != Some(head) {
        return Verdict::Wait;
    }
    match criterion {
        Criterion::Ci => {
            if repository.ci {
                status(evidence.ci)
            } else if repository.checks.is_empty() {
                Verdict::Refuse
            } else {
                let mut result = Verdict::Met;
                for number in &repository.checks {
                    match gate(*number, Freshness::Exact, head, clean, evidence) {
                        Verdict::Refuse => return Verdict::Refuse,
                        Verdict::Wait => result = Verdict::Wait,
                        Verdict::Met => {}
                    }
                }
                result
            }
        }
        Criterion::UpToDate => status(evidence.contains_base),
        Criterion::Gate { number, freshness } => gate(number, freshness, head, clean, evidence),
        Criterion::Approval { role, people, freshness } => {
            approval(role, people, freshness, head, clean, reviewers, evidence.reviews_complete)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Freshness, Reviewer, Verdict, approval};
    use skein_forge::api;

    #[test]
    fn approvals_count_distinct_people_at_the_named_head() {
        let head = [1; 32];
        let old = [2; 32];
        let review =
            |person, role, head, verdict| Reviewer { at: skein_lib::Wall::from_nanos(0), person, role, head, verdict };
        let approved = api::Verdict::Approve;
        let first = review(7, 3, head, approved);
        let second = review(8, 3, head, approved);
        assert_eq!(approval(3, 2, Freshness::Exact, head, &[], &[first, first], true), Verdict::Wait);
        assert_eq!(approval(3, 2, Freshness::Exact, head, &[], &[first, second], false), Verdict::Wait);
        assert_eq!(approval(3, 2, Freshness::Exact, head, &[], &[first, second], true), Verdict::Met);
        assert_eq!(
            approval(3, 2, Freshness::Exact, head, &[], &[first, review(8, 4, head, approved)], true),
            Verdict::Wait
        );
        assert_eq!(
            approval(3, 2, Freshness::Exact, head, &[], &[first, review(8, 3, old, approved)], true),
            Verdict::Wait
        );
        assert_eq!(
            approval(3, 2, Freshness::Clean, head, &[old], &[first, review(8, 3, old, approved)], true),
            Verdict::Met
        );
        assert_eq!(
            approval(
                3,
                2,
                Freshness::Exact,
                head,
                &[],
                &[first, second, review(7, 3, head, api::Verdict::RequestChanges)],
                true
            ),
            Verdict::Refuse
        );
    }

    #[test]
    fn sign_offs_keep_exactly_the_counted_people_and_the_earliest_eight() {
        use super::sign_offs;
        use crate::SignOff;
        use alloc::boxed::Box;
        use skein_lib::List;
        use skein_lib::Wall;
        type Case = (&'static str, Box<[(u32, Freshness)]>, Box<[Reviewer]>, Box<[SignOff]>);
        let head = [1; 32];
        let clean = [2; 32];
        let stale = [3; 32];
        let review =
            |person, role, head, verdict, at| Reviewer { person, role, head, verdict, at: Wall::from_nanos(at) };
        let signed = |person, head, at| SignOff { person, head, at: Wall::from_nanos(at) };
        let approve = api::Verdict::Approve;
        let first = review(4, 1, head, approve, 10);
        let other = review(7, 2, head, approve, 5);
        let exact: Box<[_]> = Box::new([(1, Freshness::Exact)]);
        let clean_approval: Box<[_]> = Box::new([(1, Freshness::Clean)]);
        let mut nine = List::with_capacity(9);
        let mut eight = List::with_capacity(8);
        for person in (1..=9).rev() {
            nine.push(review(person, 1, head, approve, person / 2)).unwrap();
        }
        for person in 1..=8 {
            eight.push(signed(person, head, person / 2)).unwrap();
        }
        let cases: [Case; 13] = [
            ("exact role", exact.clone(), Box::new([first, other]), Box::new([signed(4, head, 10)])),
            ("no approvals", Box::new([]), Box::new([first]), Box::new([])),
            ("other role", exact.clone(), Box::new([other]), Box::new([])),
            ("comment", exact.clone(), Box::new([review(4, 1, head, api::Verdict::Comment, 10)]), Box::new([])),
            ("changes", exact.clone(), Box::new([review(4, 1, head, api::Verdict::RequestChanges, 10)]), Box::new([])),
            ("conflict", exact.clone(), Box::new([first, review(4, 1, head, api::Verdict::Comment, 11)]), Box::new([])),
            (
                "clean conflict",
                clean_approval.clone(),
                Box::new([first, review(4, 1, clean, api::Verdict::RequestChanges, 11)]),
                Box::new([]),
            ),
            (
                "uncounted conflict",
                exact.clone(),
                Box::new([first, review(4, 1, stale, api::Verdict::RequestChanges, 11)]),
                Box::new([signed(4, head, 10)]),
            ),
            ("exact predecessor", exact.clone(), Box::new([review(4, 1, clean, approve, 10)]), Box::new([])),
            (
                "clean predecessor",
                clean_approval.clone(),
                Box::new([review(4, 1, clean, approve, 10)]),
                Box::new([signed(4, clean, 10)]),
            ),
            ("stale", clean_approval, Box::new([review(4, 1, stale, approve, 10)]), Box::new([])),
            (
                "union",
                Box::new([(1, Freshness::Exact), (1, Freshness::Clean), (2, Freshness::Exact)]),
                Box::new([first, first, other]),
                Box::new([signed(7, head, 5), signed(4, head, 10)]),
            ),
            ("earliest eight with ties", exact, nine.into_boxed(), eight.into_boxed()),
        ];
        for (name, approvals, reviewers, expected) in cases {
            assert_eq!(sign_offs(&approvals, head, &[clean], &reviewers).as_ref(), expected.as_ref(), "{name}");
        }
    }
}

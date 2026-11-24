//! Read conversion from dev's kept text (temper/docs/design/protocol/floor.md, 6).
#![expect(clippy::disallowed_macros, reason = "ordinary Rust deployed-layout fixtures")]
use super::fixtures;
use crate::{keys, records};
use jig_protocol_store::JobRow;
use skein_codec::{
    layout::{Plan, Workspace},
    schema::{self, Schema, Shape},
    value::Value,
};
use skein_lib::{Buffer, Wall};
use temper_engine_domain_forge_vocabulary as d;

pub(super) const DEV: &str = include_str!("fixtures/dev-store-v25.schema");

#[test]
fn the_dev_text_equals_the_deployed_layout_guards_copy() {
    assert!(
        DEV == include_str!("../../../../tests/codecs/deployed/13efa77461/temper-store-v25.schema"),
        "the step fixture and guard keep the same verbatim dev store-v25 text"
    );
}

/// Encode with the binary's codec, then drop only what dev's text does not hold.
pub(super) struct Dev {
    binary: Schema,
    old: Schema,
    work: Workspace,
    out: Buffer,
}
impl Dev {
    pub(super) fn new() -> Self {
        Self {
            binary: schema::parse(fixtures::layouts::BINARY.as_bytes()).unwrap(),
            old: schema::parse(DEV.as_bytes()).unwrap(),
            work: Workspace::new(),
            out: Buffer::with_capacity(1_048_576),
        }
    }
    pub(super) fn row(&mut self, stored: &d::Stored, variant: &str) -> JobRow {
        let old_root = self.old.root(b"Row", Some(variant.as_bytes())).unwrap();
        let binary_root = self.binary.root(b"Row", Some(variant.as_bytes())).unwrap();
        let plan = Plan::derive_dropping(&self.binary, binary_root, &[], &self.old, old_root, &[])
            .expect("the binary writes dev's layout directly");
        assert!(self.work.fit(&plan));
        plan.convert(&records::encode(stored), &mut self.out, &mut self.work).unwrap();
        let value: Box<[u8]> = self.out.as_slice().into();
        let mut generic = Value::with_room(100_000);
        let node = generic.decode(&self.old, old_root.decl, value.clone()).expect("bytes are a Row at dev's root");
        let Shape::Record { first, count: 1, .. } = self.old.declaration(old_root.decl).shape else {
            panic!("Row holds the value enumeration")
        };
        let enumeration = generic.field(node, first).unwrap();
        assert_eq!(Some(generic.tag(enumeration).unwrap()), old_root.variant, "dev's Row has the requested variant");
        JobRow { key: keys::encode(&d::stored_key(stored), u32::MAX).unwrap(), value }
    }
    pub(super) fn read(&mut self, row: &JobRow, variant: &str) -> d::Stored {
        let old_root = self.old.root(b"Row", Some(variant.as_bytes())).unwrap();
        let binary_root = self.binary.root(b"Row", Some(variant.as_bytes())).unwrap();
        let plan = Plan::derive(&self.old, old_root, &[], &self.binary, binary_root, &[])
            .expect("the registry derives dev's layout to the binary's");
        assert!(self.work.fit(&plan));
        plan.convert(&row.value, &mut self.out, &mut self.work).unwrap();
        let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
        records::decode(self.out.as_slice(), &bounds).unwrap()
    }
}

pub(super) fn landing_log() -> (d::Stored, d::Stored) {
    let log = d::LandingLog {
        repository: fixtures::REPO,
        landings: Box::new([d::LoggedLanding {
            commit: [7; 32],
            branch: b"main".as_slice().into(),
            at: Wall::from_nanos(123),
            changes: Box::new([d::LoggedChange {
                fixes: Box::new([7, 12]),
                signed: Box::new([d::SignOff { person: 7, head: [8; 32], at: Wall::from_nanos(122) }]),
                task: 51,
                pull: Some(101),
                title: b"member".as_slice().into(),
                goal: Some(42),
                milestone: Some(40),
                person: Some(7),
                goal_title: Some(b"Goal".as_slice().into()),
                draft: Some(b"temper/docs/design/draft/after-landing-repairs.md".as_slice().into()),
            }]),
            tiers: Box::new([
                d::LoggedTier {
                    tier: b"focused".as_slice().into(),
                    verdict: d::LoggedVerdict::Red {
                        run: 7,
                        failing: Some(1),
                        tests: Box::new([b"suite::a".as_slice().into()]),
                        many: None,
                    },
                },
                d::LoggedTier { tier: b"fuzzy".as_slice().into(), verdict: d::LoggedVerdict::Cleared { run: 6 } },
            ]),
        }]),
    };
    let mut expected = log.clone();
    for landing in &mut expected.landings {
        landing.tiers = Box::new([]);
        for member in &mut landing.changes {
            member.fixes = Box::new([]);
            member.goal = None;
            member.milestone = None;
            member.person = None;
            member.goal_title = None;
            member.draft = None;
        }
    }
    (d::Stored::LandingLog(log), d::Stored::LandingLog(expected))
}

pub(super) fn change_cold() -> (d::Stored, d::Stored) {
    let (mut cold, _) = fixtures::change_row(temper_engine_domain_forge_change::State::Ended).split();
    cold.goal = Some(42);
    cold.milestone = Some(40);
    cold.person = Some(7);
    cold.goal_title = Some(b"Goal".as_slice().into());
    cold.draft = Some(b"temper/docs/design/draft/after-landing-repairs.md".as_slice().into());
    let mut expected = cold.clone();
    expected.person = None;
    expected.goal_title = None;
    expected.draft = None;
    (d::Stored::ChangeCold(Box::new(cold)), d::Stored::ChangeCold(Box::new(expected)))
}

pub(super) fn finding() -> (d::Stored, d::Stored) {
    let finding = d::Finding {
        project: 1,
        number: 7,
        kind: d::FindingKind::Red,
        severity: d::Severity::High,
        area: Some(b"store".as_slice().into()),
        state: d::FindingState::Routed,
        title: b"A red finding".as_slice().into(),
        body: b"The failing test's report".as_slice().into(),
        filer: d::Filer::Task(51),
        at: Wall::from_nanos(123),
        why: Some(b"Assigned".as_slice().into()),
        hander: Some(d::Filer::Party(7)),
        goal: Some(42),
        reason: Some(b"A repair is running".as_slice().into()),
        signature: Some(d::FindingSignature {
            repository: fixtures::REPO,
            source: b"focused".as_slice().into(),
            ids: Box::new([b"suite::a".as_slice().into()]),
            first: b"suite::a".as_slice().into(),
            many: true,
        }),
        seen: Some(2),
        blame: Some(d::FindingBlame {
            strength: d::BlameStrength::Owned,
            landings: Box::new([d::BlamedLanding {
                commit: [7; 32],
                goals: Box::new([d::BlamedGoal { goal: 42, milestone: Some(40), person: Some(7) }]),
            }]),
            moved: None,
        }),
        cleared: Some(d::FindingCleared { run: 8, at: Wall::from_nanos(124) }),
        repair: None,
    };
    let mut expected = finding.clone();
    expected.blame = None;
    expected.cleared = None;
    expected.signature.as_mut().unwrap().many = false;
    (d::Stored::Finding(finding), d::Stored::Finding(expected))
}

pub(super) fn tier() -> (d::Stored, d::Stored) {
    let mut tier = fixtures::tier_row();
    let (d::Stored::Finding(finding), _) = finding() else { panic!("finding fixture") };
    tier.waiting = Box::new([d::VerdictFiling {
        project: finding.project,
        kind: finding.kind,
        severity: finding.severity,
        title: finding.title,
        body: finding.body,
        signature: d::FindingSignature { many: false, ..finding.signature.unwrap() },
        blame: finding.blame,
        follow: Some(7),
    }]);
    tier.unkept = 12;
    let mut expected = tier.clone();
    expected.waiting = Box::new([]);
    expected.unkept = 0;
    (d::Stored::Tier(tier), d::Stored::Tier(expected))
}

fn converts(variant: &str, (stored, expected): (d::Stored, d::Stored)) {
    let mut dev = Dev::new();
    let row = dev.row(&stored, variant);
    assert_eq!(dev.read(&row, variant), expected, "{variant} defaults additions and preserves every other field");
}
#[test]
fn dev_landing_members_default_goal_facts_fixes_and_tiers() {
    converts("landing_log", landing_log());
}
#[test]
fn dev_cold_changes_default_person_goal_title_and_draft() {
    converts("change_cold", change_cold());
}
#[test]
fn dev_findings_default_blame_clearing_and_many() {
    converts("finding", finding());
}
#[test]
fn dev_tiers_default_waiting_and_unkept() {
    converts("tier", tier());
}

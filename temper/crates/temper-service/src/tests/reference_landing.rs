//! The reference's landing member (temper/docs/design/protocol/configuration.md, 3 and 13.9).
#![expect(clippy::disallowed_macros, reason = "the reference JSON is compiled into its parser regression")]
use super::fixtures;
use crate::startup::{Problems, application, lines};
use jig_domain_authority::Last;
use jig_protocol_config::{self as config, read};
use temper_engine_domain::{Freshness, Gate};

const PROJECT: u32 = 1;
const CONNECTOR: u16 = 1;
const MERGE: u16 = 4;
// The reference configures project 1 alone.
const PROJECTS: u32 = 1;
// Project 1 has exactly a main rule and a goal branches rule.
const RULES: usize = 2;
// docs/examples/owned.json's authority.segments accommodates the repository path.
const SEGMENTS: u32 = 8;
// The startup fixture's parsed() document bounds.
const DOCUMENT: config::DocumentLimits = config::DocumentLimits { bytes: 1_048_576, tokens: 16_384, depth: 32 };

#[test]
fn the_reference_landing_member_parses() {
    let input = fixtures::with(
        &fixtures::document(),
        b"application.landing",
        include_bytes!("../../../../docs/examples/reference/landing.json"),
    );
    let document = config::document(&input, &DOCUMENT).expect("startup document");
    let at = read::Object::new(&document, 0).expect("document object").member(&document, b"application").unwrap();
    let mut jig = fixtures::engine().jig;
    jig.limits.core.authority.segments = SEGMENTS;
    let mut problems = Problems::empty();
    let members = application::parse(&document, at, &jig, &config::Directories::working(), &mut problems);
    assert_eq!(lines(&problems).as_ref(), b"", "reference landing accepted");
    let policy = members.expect("application members").landing;
    assert!(policy.deployment.is_empty());
    assert_eq!(policy.projects.len(), PROJECTS);
    let rules = policy.projects.get(&PROJECT).expect("reference project");
    assert_eq!(rules.len(), RULES);
    let expected: [(Last, bool, &[Gate]); RULES] = [
        (
            Last::Exact(b"main".as_slice().into()),
            true,
            &[Gate { number: 1, blocking: true, freshness: Freshness::Clean }],
        ),
        (Last::Open(b"temper".as_slice().into()), false, &[]),
    ];
    for (rule, (last, checks, gates)) in rules.iter().zip(expected) {
        assert_eq!(rule.connector, CONNECTOR);
        assert_eq!(rule.kind, MERGE);
        assert_eq!(
            rule.pattern.segments.as_ref(),
            [
                Box::from(b"forge".as_slice()),
                Box::from(b"127.0.0.1:3000".as_slice()),
                Box::from(b"acme".as_slice()),
                Box::from(b"widget".as_slice()),
                Box::from(b"branch".as_slice()),
            ]
        );
        assert_eq!(rule.pattern.last, last);
        assert!(rule.enforce);
        assert_eq!(rule.ci, checks);
        assert_eq!(rule.up_to_date, checks);
        assert_eq!(rule.gates.as_ref(), gates);
        assert!(rule.approvals.is_empty());
    }
}

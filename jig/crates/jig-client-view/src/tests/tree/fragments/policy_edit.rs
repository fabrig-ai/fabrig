use crate::tests::kit::Goldens;
use crate::tests::tree::cards::words;
use crate::tree::fragments::policy_edit;
use jig_client::contract::*;
use skein_lib::{Duration, List};
use skein_ui::{Builder, Role};
fn pattern(last: PatternEnd) -> Pattern {
    let segments = crate::tests::kit::list([Box::from(b"literal/slash".as_slice()), Box::from(b"\xff".as_slice())]);
    Pattern { segments, last }
}
fn shown(value: &PolicyEdit) -> skein_ui::Tree {
    let words = words();
    let required = crate::tests::render_limits(&crate::tests::tree::cards::CLIENT, words).unwrap();
    let mut b = Builder::new(&skein_ui::Limits {
        nodes: required.nodes.checked_add(1).unwrap(),
        depth: required.depth.checked_add(1).unwrap(),
        text_per_node: required.text_per_node,
        text_bytes: required.text_bytes,
        drawing_bytes: 0,
        styles_per_node: 0,
        patches: 0,
        bindings: 0,
        slots: 0,
        markdown_depth: 0,
        scene_marks: 0,
        scene_bytes: 0,
    });
    b.open(Role::Page);
    policy_edit::render(
        &mut b,
        value,
        words,
        crate::tests::tree::cards::CLIENT.client.text,
        crate::tests::tree::cards::CLIENT.client.rows,
    );
    b.close();
    b.finish()
}
#[test]
fn policy_requirements_keep_guard_freshness_judge_and_segment_boundaries_in_the_public_fragment() {
    let mut goldens = Goldens::new("tree/fragments/policy_edit/policy_requirements_keep");
    for (guard, must_be_guarded) in
        [(PolicyGuard::Guarded, true), (PolicyGuard::Observed { freshness: Duration::from_nanos(u64::MAX) }, false)]
    {
        for last in [PatternEnd::Exact(Box::from(b"end".as_slice())), PatternEnd::Open(Box::from(b"end".as_slice()))] {
            let requirements = crate::tests::kit::list([PolicyRequirement {
                connector: u16::MAX,
                kind: u16::MAX,
                pattern: pattern(last),
                judge: PolicyJudge { connector: u16::MAX, requirement: u16::MAX, parameters: u32::MAX },
                guard,
                must_be_guarded,
            }]);
            let tree = shown(&PolicyEdit::Requirements { requirements });
            goldens.check(&tree);
        }
    }
}
#[test]
fn policy_role_projection_permission_and_empty_replacement_reviews_keep_every_supplied_field() {
    let mut goldens = Goldens::new("tree/fragments/policy_edit/policy_role_projection");
    let mappings =
        crate::tests::kit::list([PolicyPermission { connector: u16::MAX, permission: u16::MAX, role: u32::MAX }]);
    let tree = shown(&PolicyEdit::Permissions { mappings });
    goldens.check(&tree);
    let grants = crate::tests::kit::list([Grant {
        connector: 7,
        kind: 9,
        pattern: pattern(PatternEnd::Open(Box::from(b"end".as_slice()))),
    }]);
    goldens.check(&shown(&PolicyEdit::Projections { grants }));
    let tree = shown(&PolicyEdit::Role(PolicyEditRole {
        number: u32::MAX,
        authority: Authority {
            models: None,
            tools: u64::MAX,
            grants: List::with_capacity(0),
            delegation: Delegation { kinds: List::with_capacity(0), tasks: u32::MAX, depth: u32::MAX },
            spend: u64::MAX,
            deadline: None,
            notes: u8::MAX,
            note_resources: List::with_capacity(0),
        },
        period_spend: u64::MAX,
        requests: u16::MAX,
        decides: u8::MAX,
    }));
    goldens.check(&tree);
    goldens.check(&shown(&PolicyEdit::ProjectSpend { period_spend: u64::MAX }));
    let tree = shown(&PolicyEdit::RoleSpend { role: 4, period_spend: u64::MAX });
    goldens.check(&tree);
    for edit in [
        PolicyEdit::Requirements { requirements: List::with_capacity(0) },
        PolicyEdit::Projections { grants: List::with_capacity(0) },
        PolicyEdit::Permissions { mappings: List::with_capacity(0) },
    ] {
        assert_eq!(shown(&edit).len(), 3);
    }
}

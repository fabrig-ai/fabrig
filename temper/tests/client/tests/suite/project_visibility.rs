//! A member cannot widen a project through ctl (protocol/sign-in.md, 2.5).
use temper_client_world::fixtures;

#[test]
fn a_members_project_amend_visibility_exits_two_and_leaves_the_project_private() {
    let mut world = fixtures::asking_world(fixtures::SEED, true);
    let member = fixtures::asking_agent(&mut world);
    let number = fixtures::give(
        &mut world,
        member,
        &[b"project", b"amend", b"1", b"--visibility", b"public", b"--key", b"member-cannot-open"],
    );
    let refused = fixtures::answered(&mut world, member, number);
    assert_eq!(refused.status, 2, "{refused:?}");
    assert!(refused.stdout.is_empty(), "{refused:?}");
    assert_eq!(refused.stderr.as_ref(), b"project amend: The held role does not permit this action\n");
    // A fresh ctl reads the engine's decision, not the caller's retained document.
    let fresh = fixtures::asking_agent(&mut world);
    let number = fixtures::give(&mut world, fresh, &[b"project", b"show", b"1"]);
    let shown = fixtures::answered(&mut world, fresh, number);
    assert_eq!(shown.status, 0, "{shown:?}");
    let text = String::from_utf8(shown.stdout.into_vec()).expect("ctl UTF-8");
    assert!(text.contains("\npublic: no\n"), "{text}");
}

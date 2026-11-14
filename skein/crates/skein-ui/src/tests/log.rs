use super::fixtures::LIMITS;
use crate::{Act, Binding, BoundAct, Builder, LogEdge, NodeId, Owner, Patch, Role, State, View, takes};
use skein_lib::{Queue, Token};

const BINDING: Binding = Binding { owner: Owner { number: 2 }, token: Token::new(7) };

#[test]
fn a_log_takes_near_and_nothing_else() {
    for &(role, _) in super::tree::BINDING_ROLES {
        for submits in [false, true] {
            for edge in [LogEdge::Head, LogEdge::Foot] {
                assert_eq!(
                    takes(role, submits, &Act::Near { edge }),
                    role == Role::Log || (role == Role::List && edge == LogEdge::Foot),
                    "{role:?}"
                );
            }
        }
    }
    for act in
        [Act::Press, Act::Input { value: Box::new([]) }, Act::Submit, Act::Toggle, Act::Drawing { data: Box::new([]) }]
    {
        assert!(!takes(Role::Log, true, &act));
    }
}

#[test]
fn near_decodes_on_a_bound_log_and_is_dropped_on_an_unbound_one() {
    let mut view = View::new(&LIMITS);
    let mut b = view.builder();
    b.open(Role::Page);
    for bound in [true, false] {
        b.open(Role::Log);
        if bound {
            b.bind(BINDING);
        }
        b.close();
    }
    b.close();
    view.finish(b, 0, &mut Queue::with_capacity(LIMITS.patches));
    for edge in [LogEdge::Head, LogEdge::Foot] {
        assert_eq!(
            view.act(NodeId(2), Act::Near { edge }),
            Some(BoundAct { binding: BINDING, act: Act::Near { edge } })
        );
        assert_eq!(view.act(NodeId(3), Act::Near { edge }), None);
    }
    assert_eq!(view.act(NodeId(2), Act::Press), None);
}

#[test]
#[should_panic(expected = "continues belongs to logs or lists")]
fn continues_belongs_to_a_log() {
    let mut b = Builder::new(&LIMITS);
    b.open(Role::Button);
    b.state(State::Continues);
}

#[test]
#[should_panic(expected = "end requests belong to logs")]
fn an_end_request_belongs_to_a_log() {
    let mut b = Builder::new(&LIMITS);
    b.open(Role::Button);
    b.end_request(1);
}

#[test]
fn logs_and_lists_keep_continues_and_logs_keep_end_requests() {
    for role in [Role::Log, Role::List] {
        let mut b = Builder::new(&LIMITS);
        b.open(role);
        b.state(State::Continues);
        b.close();
        assert_eq!(b.finish().get(0).unwrap().states.as_slice(), [State::Continues], "{role:?}");
    }
    let mut b = Builder::new(&LIMITS);
    b.open(Role::Page);
    for count in [1, 2] {
        b.open(Role::Log);
        b.state(State::Continues);
        b.state(State::Continues);
        b.end_request(count);
        b.close();
    }
    b.close();
    let tree = b.finish();
    for (node, count) in tree.nodes().iter().skip(1).zip([1, 2]) {
        assert_eq!(node.end_request, Some(count));
        assert_eq!(node.states.as_slice(), [State::Continues]);
    }
}

#[test]
fn an_end_request_that_moves_patches_the_log() {
    let mut view = View::new(&LIMITS);
    let mut patches = Queue::with_capacity(LIMITS.patches);
    for (revision, request) in [None, Some(1), Some(1), Some(2), None].into_iter().enumerate() {
        let mut b = view.builder();
        b.open(Role::Log);
        if let Some(count) = request {
            b.end_request(count);
        }
        b.close();
        view.finish(b, u64::try_from(revision).unwrap(), &mut patches);
        if revision == 0 {
            match patches.pop().unwrap() {
                Patch::Insert { nodes, .. } => assert_eq!(nodes[0].attribute.end_request, None),
                patch @ (Patch::Remove { .. }
                | Patch::Move { .. }
                | Patch::Text { .. }
                | Patch::Attribute { .. }
                | Patch::Value { .. }) => panic!("expected insertion, got {patch:?}"),
            }
        } else if revision == 2 {
            assert!(patches.is_empty());
        } else {
            assert_eq!(patches.len(), 1);
            match patches.pop().unwrap() {
                Patch::Attribute { node, attribute } => assert_eq!((node, attribute.end_request), (NodeId(1), request)),
                patch @ (Patch::Insert { .. }
                | Patch::Remove { .. }
                | Patch::Move { .. }
                | Patch::Text { .. }
                | Patch::Value { .. }) => panic!("expected end request attribute, got {patch:?}"),
            }
        }
        assert!(patches.is_empty());
    }
}

#[test]
fn a_bound_list_takes_only_its_foot() {
    let mut view = View::new(&LIMITS);
    let mut b = view.builder();
    b.open(Role::List);
    b.bind(BINDING);
    b.state(State::Continues);
    b.close();
    view.finish(b, 0, &mut Queue::with_capacity(LIMITS.patches));
    assert_eq!(
        view.act(NodeId(1), Act::Near { edge: LogEdge::Foot }),
        Some(BoundAct { binding: BINDING, act: Act::Near { edge: LogEdge::Foot } })
    );
    for act in [
        Act::Near { edge: LogEdge::Head },
        Act::Press,
        Act::Input { value: Box::new([]) },
        Act::Submit,
        Act::Toggle,
        Act::Drawing { data: Box::new([]) },
    ] {
        for submits in [false, true] {
            assert!(!takes(Role::List, submits, &act), "{act:?}");
        }
        assert_eq!(view.act(NodeId(1), act), None);
    }
}

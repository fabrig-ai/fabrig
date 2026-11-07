use crate::{Exit, Split, passed, split};

#[test]
fn a_passed_child_reaches_its_parent_as_code_zero() {
    passed();
    match split() {
        Split::Child => {
            passed();
            panic!("passed returned in the child");
        }
        Split::Parent(exit) => assert_eq!(exit, Exit::Code(0), "child passed"),
    }
}

#[test]
fn a_panicking_child_reaches_its_parent_as_code_101() {
    match split() {
        Split::Child => panic!("the child's failed check"),
        Split::Parent(exit) => assert_eq!(exit, Exit::Code(101), "child panicked"),
    }
}

#[test]
fn a_childs_writes_do_not_reach_its_parent() {
    let mut writes = vec![1];
    match split() {
        Split::Child => {
            writes.push(2);
            assert_eq!(writes, [1, 2], "child wrote its own memory");
            passed();
        }
        Split::Parent(exit) => assert_eq!(exit, Exit::Code(0), "child completed its writes"),
    }
    assert_eq!(writes, [1], "child writes must not reach the parent");
}

#[test]
fn a_child_cannot_split_again() {
    match split() {
        Split::Child => match split() {
            Split::Child | Split::Parent(_) => passed(),
        },
        Split::Parent(exit) => assert_eq!(exit, Exit::Code(101), "nested split is refused"),
    }
}

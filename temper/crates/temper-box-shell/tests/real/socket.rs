use temper_box_kit::{socket, story_socket};

#[test]
fn story_socket_blocks() {
    story_socket::tests::story_socket_blocks();
}

#[test]
fn owned_socket_blocks() {
    socket::tests::owned_socket_blocks();
}

#[test]
fn deferred_socket_blocks() {
    socket::tests::deferred_socket_blocks();
}

#[test]
fn socket_with_shared_deadline_blocks() {
    socket::tests::socket_with_shared_deadline_blocks();
}

#[test]
fn read_bounded_socket_blocks() {
    socket::tests::read_bounded_socket_blocks();
}

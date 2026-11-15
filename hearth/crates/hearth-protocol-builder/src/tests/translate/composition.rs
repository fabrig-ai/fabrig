#[test]
fn a_document_s_members_keep_their_names_positions_and_layers() {
    use crate::render::revision::{Layer, Member, Members, Position};
    use skein_lib::bytes::copy_of;
    let document = crate::translate::documents::document(crate::tests::fixtures::member_document());
    assert_eq!(
        document.members,
        Some(Members {
            at: Position { line: 1, column: 12 },
            each: Box::new([
                Member {
                    name: copy_of(b"1"),
                    at: Position { line: 1, column: 13 },
                    layer: Some(Layer { debian: None, placement: None, tasks: Box::new([]) })
                },
                Member { name: copy_of(b"2"), at: Position { line: 1, column: 20 }, layer: None },
            ])
        })
    );
}

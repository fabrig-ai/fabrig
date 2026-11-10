//! Literal event documents retain their bytes and framing
//! (skein/docs/design/testing/README.md, 6).

#[test]
fn events_keep_literal_documents_and_delimiters() {
    assert_eq!(skein_llm_kit::events(&[]), b"");
    assert_eq!(
        skein_llm_kit::events(&["", "{\"text\":\"é\"}", "first\nsecond"]),
        "data: \n\ndata: {\"text\":\"é\"}\n\ndata: first\nsecond\n\n".as_bytes()
    );
}

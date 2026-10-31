use super::*;

#[test]
fn repeated_names_keep_each_original_label_and_text_until_its_receipt() {
    let mut book = Book::new(Capture::Everything);
    let name = Token::new(4);
    book.received(name, b"task", b"first input");
    book.received(name, b"relay", b"second, refused input");
    let first = book.take(name).unwrap();
    assert_eq!(first.label.as_ref(), b"task");
    assert_eq!(first.text.as_ref(), b"first input");
    let second = book.take(name).unwrap();
    assert_eq!(second.label.as_ref(), b"relay");
    assert_eq!(second.text.as_ref(), b"second, refused input");
    assert!(book.pending.is_empty());
    book.received(name, b"never admitted", b"start refused");
    book.clear();
    assert!(book.pending.is_empty());
    book.received(name, b"task", b"next run");
    assert_eq!(book.take(name).unwrap().text.as_ref(), b"next run");
}

#[test]
fn absent_content_selection_owns_no_message_copy() {
    for capture in [Capture::None, Capture::Calls] {
        let mut book = Book::new(capture);
        book.received(Token::new(4), b"task", b"text");
        assert!(book.take(Token::new(4)).is_none());
        assert!(book.pending.is_empty());
    }
}

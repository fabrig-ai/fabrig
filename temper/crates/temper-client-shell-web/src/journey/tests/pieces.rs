use super::super::{Line, record};
use crate::host::Duties;
use jig_client_protocol_http::{Mapping, exchange};
use jig_client_shell_web::adapter::{Asked, Heard};

#[test]
fn pieced_exchange_lines_read_print_and_record() {
    let ask = Line::Exchange {
        id: 7,
        bound: 64,
        pieces: Some(16),
        method: b"POST".as_slice().into(),
        path: b"/jig/v1/batch".as_slice().into(),
        body: b"batch".as_slice().into(),
        headers: vec![(b"Content-Type".as_slice().into(), b"application/octet-stream".as_slice().into())],
    };
    let rows = [
        (r#"pieces 7 64 16 POST "/jig/v1/batch" "batch" "Content-Type" "application/octet-stream""#, ask.clone()),
        ("heard begun 7 200", Line::Heard(Heard::Begun { exchange: 7, status: 200 })),
        (r#"heard part 7 "abc""#, Line::Heard(Heard::Part { exchange: 7, bytes: b"abc".as_slice().into() })),
        ("heard done 7", Line::Heard(Heard::Done { exchange: 7 })),
        ("heard failed 7", Line::Heard(Heard::Failed { exchange: 7 })),
    ];
    for (text, expected) in rows {
        let line = Line::read(text.as_bytes()).expect("pieced exchange line reads");
        assert_eq!(line, expected);
        assert_eq!(line.clone(), line);
        let mut out = Vec::new();
        line.write(&mut out);
        assert_eq!(out, [text.as_bytes(), b"\n"].concat());
    }
    let call = exchange(
        &Mapping::new(b"/jig").expect("mapping"),
        None,
        jig_client::Message::Batch(b"batch".as_slice().into()),
    );
    let duties = Duties {
        asked: vec![Asked::Exchange {
            id: 7,
            call,
            bound: 64,
            pieces: Some(16),
            cache: skein_ui_web::fetch::Cache::Browser,
            keep: &[],
        }],
        ..Duties::default()
    };
    assert_eq!(record::asked(&duties), vec![ask]);
}

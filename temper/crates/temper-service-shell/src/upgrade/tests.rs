//! The binaries' shared --check report words (protocol/binaries.md).
use super::*;
use skein_lib::{Duration, Queue, Token};

#[test]
fn check_lines_name_decoded_count_and_undecodable_places() {
    let directories =
        temper_shell::directories::resolve(&[], &[(b"HOME".as_slice().into(), b"/home/test".as_slice().into())], b"/")
            .expect("directories")
            .0;
    for binary in ["temper", "temper-box", "temper-engine"] {
        let mut report =
            Report { events: Queue::uncapped(), elapsed: Duration::ZERO, end: None, restored: true, done: true };
        report.events.push(Event::Decoded { owner: Token::new(1), records: 17 });
        let (text, code) = lines(binary, &report, &directories, b"/state");
        assert_eq!(code, 0);
        assert!(text.starts_with(&format!("{binary} upgrade: decoded 17 records\n")), "{text}");
        for (at, place) in [
            (Undecoded::Record { map: KeptMap::Store, key: Box::new([0, 0xab, 0xff]) }, "store key 00abff"),
            (Undecoded::Record { map: KeptMap::Secrets, key: Box::new([2, 0, 1]) }, "secrets key 020001"),
            (Undecoded::Frame { transcript: "7".into(), offset: 256 }, "transcript 7 at 256"),
        ] {
            let mut report =
                Report { events: Queue::uncapped(), elapsed: Duration::ZERO, end: None, restored: false, done: true };
            report.events.push(Event::Undecodable {
                owner: Token::new(1),
                family: "temper-store".into(),
                version: 11,
                at,
            });
            report.events.push(Event::Closed { owner: Token::new(1), removed: true });
            let (text, code) = lines(binary, &report, &directories, b"/state");
            assert_eq!(code, 1);
            assert!(
                text.starts_with(&format!("{binary} upgrade: record not decodable: temper-store 11, {place}\n")),
                "{text}"
            );
            assert!(!text.contains("patches and restores pass"));
        }
    }
}

use crate::{
    front::{self, LINE_BYTES, Line},
    layout::Placed,
    machine::Why,
    terminal::{Fact, Formatter, LinkForm},
};
use skein_lib::{Duration, List, bytes};

#[test]
fn each_line_is_its_word_its_field_and_a_line_feed() {
    let full: Box<[u8]> = Box::new([b'x'; 8187]);
    let short: Box<[u8]> = Box::new([b'x'; 8186]);
    let mut rows = List::uncapped();
    for row in [
        (
            Line::Link { url: bytes::copy_of(b"http://local/first?secret=fixture") },
            bytes::copy_of(b"link http://local/first?secret=fixture\n"),
        ),
        (Line::Log { text: bytes::copy_of(b"00:00:01Z Stopping") }, bytes::copy_of(b"log 00:00:01Z Stopping\n")),
        (Line::Restarting { step: bytes::copy_of(b"forgejo-3") }, bytes::copy_of(b"restarting forgejo-3\n")),
        (Line::Running { step: bytes::copy_of(b"forgejo-3") }, bytes::copy_of(b"running forgejo-3\n")),
        (Line::GaveUp { step: bytes::copy_of(b"forgejo-3") }, bytes::copy_of(b"gave-up forgejo-3\n")),
        (Line::Log { text: bytes::copy_of(b" a  b ") }, bytes::copy_of(b"log  a  b \n")),
        (Line::Log { text: bytes::copy_of(&full) }, crate::lines::joined(&[b"log ", &full, b"\n"])),
        (Line::Log { text: Box::new([b'x'; 8188]) }, crate::lines::joined(&[b"log ", &full, b"\n"])),
    ] {
        rows.push(row).unwrap();
    }
    for character in ["é", "€", "𝄞"] {
        rows.push((
            Line::Log { text: crate::lines::joined(&[&short, character.as_bytes()]) },
            crate::lines::joined(&[b"log ", &short, b"\n"]),
        ))
        .unwrap();
    }
    for (line, expected) in &rows {
        let output = line.bytes();
        assert!(&output == expected, "a front line keeps its word and field within a UTF-8 boundary");
        assert!(output.len() <= LINE_BYTES);
        assert_eq!(bytes::count(&output, b"\n", 2), 1);
        assert_eq!(output.last(), Some(&b'\n'));
    }
}

fn labelled() -> Box<[Placed]> {
    use temper_forge_setup::{Group, Kind, Stage, Steps};
    crate::layout::flatten(Steps {
        always: Box::new([]),
        asked: Box::new([Group {
            kind: None,
            remover: None,
            forge: 3,
            before: 0,
            recorded: false,
            label: bytes::copy_of(b"Forge 3 (Forgejo)"),
            steps: Box::new([(Stage::Run, super::fixtures::step(b"forgejo-3", Kind::Long))]),
        }]),
    })
}

#[test]
fn items_and_facts_become_the_fronts_lines() {
    for (input, expected) in [
        (b"00:00:01Z Stopping\n".as_slice(), [b"00:00:01Z Stopping".as_slice()].as_slice()),
        (
            b"01:02:03Z serve exited 1; restarting in 0.5 s\n01:02:03Z     serve: boom\n01:02:03Z     Log: /data/box/logs/serve.log\n",
            [
                b"01:02:03Z serve exited 1; restarting in 0.5 s".as_slice(),
                b"01:02:03Z     serve: boom",
                b"01:02:03Z     Log: /data/box/logs/serve.log",
            ].as_slice(),
        ),
        (b"a\n\nb", [b"a".as_slice(), b"b"].as_slice()),
        (b"\n\n", [].as_slice()),
        (b"", [].as_slice()),
    ] {
        let mut lines = List::uncapped();
        for text in expected {
            lines.push(Line::Log { text: bytes::copy_of(text) }).unwrap();
        }
        assert_eq!(front::logs(input).as_slice(), lines.as_slice());
    }
    let steps = labelled();
    let why = Why::NotStarted { step: 0 };
    for (fact, expected) in [
        (Fact::Starting { step: 0 }, None),
        (Fact::GroupStarting { forge: 3 }, None),
        (Fact::GroupStarted { forge: 3 }, None),
        (Fact::GroupFailed { forge: 3, why }, None),
        (Fact::GroupStopped { forge: 3 }, None),
        (
            Fact::Restarting { step: 0, why, delay: Duration::from_millis(500) },
            Some(Line::Restarting { step: bytes::copy_of(b"forgejo-3") }),
        ),
        (Fact::GivingUp { step: 0, why }, Some(Line::GaveUp { step: bytes::copy_of(b"forgejo-3") })),
        (Fact::Stopping, None),
        (Fact::Stopped, None),
    ] {
        assert_eq!(front::state(fact, &steps), expected);
    }
    for form in [LinkForm::Bare, LinkForm::Osc8] {
        let mut formatter = Formatter::new(form, bytes::copy_of(b"temper"));
        let url = b"http://local/first?secret=fixture";
        let words = front::block_words(&formatter, url);
        assert!(
            words.as_ref() == b"Open this link in a browser on this machine.\nIt signs you in as this temper's owner, once, within 2 hours.\nOpened after that, it prints a fresh one here.\nIn a terminal: temper http://local\n",
            "the block's words omit the printed link and blank lines"
        );
        assert!(!super::fixtures::Harness::contains(&words, b"secret"), "the block's words contain no secret");
        formatter.admitted_block();
        assert!(front::block_words(&formatter, url).is_empty(), "a fresh block has no words");
    }
}

#[test]
#[expect(clippy::disallowed_macros, reason = "the unit test observes Debug output with format!")]
fn a_links_debug_names_no_url() {
    let url = b"http://local/first?secret=fixture";
    let debug = format!("{:?}", Line::Link { url: bytes::copy_of(url) });
    assert!(debug == "Link { .. }", "a link's debug names its kind alone");
    assert!(!super::fixtures::Harness::contains(debug.as_bytes(), url), "debug contains no printed link");
    let debug = format!("{:?}", Line::Log { text: bytes::copy_of(b"Stopping") });
    assert!(debug.contains("text"));
    assert!(debug.contains("83, 116, 111, 112, 112, 105, 110, 103"), "a log's debug keeps its bytes");
}

//! The host's world: the platform's reports given by the test, through the seam
//! (client/journeys.md, 2 and 6).
use jig_client_shell_web::adapter::Asked;
use skein_lib::{Time, Wall};
use skein_ui_web::mapping::Primary;
use temper_client_shell_web::host::{self, Duties, Host, Report};

fn started(path: &[u8]) -> Host {
    let mut host = Host::new(11, Time::ZERO, Wall::EPOCH, temper_client_view::sign_in::Ways::NONE);
    host.start(path, Vec::new(), None);
    host
}

#[test]
fn a_start_opens_the_event_stream() {
    let mut host = started(b"/accounts/connect");
    let Duties { asked, .. } = host.report(Report::Start);
    assert_eq!(asked.len(), 1);
    assert!(matches!(asked[0], Asked::OpenStream { .. }));
}

#[test]
fn a_start_at_a_stale_filter_shows_its_printed_address() {
    let mut host = started(b"/?kind=reply&project=3");
    let duties = host.report(Report::Start);
    assert_eq!(duties.show, Some((Box::from(b"/?project=3".as_slice()), true)));
}

#[test]
fn a_drawn_page_carries_the_views_hrefs() {
    let mut host = started(b"/accounts/connect");
    host.report(Report::Start);
    let mut page = host::page(&host.bounds.tree, Primary::Control);
    host.draw(&mut page, &mut Vec::new());
    let html = page.html();
    assert!(
        html.split("</a>").any(|anchor| anchor.contains("href=\"/chats\"") && anchor.contains(">temper<")),
        "{html}"
    );
}

#[path = "host/walk.rs"]
mod walk;
use temper_client_shell_web::journey::{Line, Refusal, Replay, Stale, lines, prove};

fn written(records: &[Line]) -> Box<[u8]> {
    let mut out = Vec::new();
    for record in records {
        record.write(&mut out);
    }
    out.into()
}
fn edited(recording: &[u8], at: impl Fn(&Line) -> bool, to: Line) -> Box<[u8]> {
    let mut records = lines(recording).expect("walk's lines");
    let index = records.iter().position(at).expect("record to edit");
    records[index] = to;
    written(&records)
}
fn stopped(recording: &[u8]) -> Stale {
    let mut replay = Replay::new(recording).expect("read changed recording");
    loop {
        match replay.next() {
            Err(stale) => return stale,
            Ok(Some(_)) => {}
            Ok(None) => panic!("changed recording should be stale"),
        }
    }
}
fn line_text(line: &Line) -> String {
    let mut bytes = Vec::new();
    line.write(&mut bytes);
    bytes.pop();
    String::from_utf8(bytes).expect("line is ASCII")
}
fn at_line(recording: &[u8], at: impl Fn(&Line) -> bool) -> (u32, u32, Line) {
    let records = lines(recording).expect("walk's lines");
    let index = records.iter().position(at).expect("record to change");
    let turn = records[..=index].iter().filter(|line| matches!(line, Line::Turn { .. })).count();
    (u32::try_from(turn).expect("turn fits"), u32::try_from(index + 1).expect("line fits"), records[index].clone())
}
#[test]
fn a_recorded_walk_proves_and_replays_its_marks() {
    let recording = prove(&walk::walked()).expect("live walk proves");
    let mut replay = Replay::new(&recording).expect("read proved recording");
    let act = replay.next().expect("replay act").expect("act mark");
    assert_eq!(act.number, 1);
    assert!(!act.end);
    assert!(act.recorded.is_none());
    let node = act.tree.find(act.acted.expect("acted control")).expect("control in tree");
    assert_eq!(node.role, skein_ui::Role::Button);
    assert!(act.page.html().contains("Continue with ChatGPT"));
    let end = replay.next().expect("replay end").expect("end mark");
    assert_eq!(end.number, 2);
    assert!(end.end);
    assert!(end.acted.is_none());
    assert!(end.recorded.is_none());
    assert!(replay.next().expect("after end").is_none());
}
#[test]
fn a_proved_recording_keeps_no_html() {
    let recording = prove(&walk::walked()).expect("live walk proves");
    let marks: Vec<_> = lines(&recording)
        .expect("proved lines")
        .into_iter()
        .filter(|line| matches!(line, Line::Mark(_) | Line::End(_)))
        .collect();
    assert_eq!(marks, [Line::Mark(None), Line::End(None)]);
}
#[test]
fn a_changed_ask_is_stale_at_its_line() {
    let recording = walk::walked();
    let is_press_exchange =
        |line: &Line| matches!(line, Line::Exchange { path, .. } if path.as_ref() == b"/jig/v1/request");
    let (turn, number, original) = at_line(&recording, is_press_exchange);
    let mut changed = original.clone();
    let Line::Exchange { body, .. } = &mut changed else { unreachable!() };
    *body = b"changed".as_slice().into();
    let stale = stopped(&edited(&recording, is_press_exchange, changed.clone()));
    assert_eq!(stale, Stale { turn, line: number, recorded: line_text(&changed), asked: line_text(&original) });
}
#[test]
fn a_changed_deadline_is_stale_at_its_line() {
    let recording = walk::walked();
    let is_deadline = |line: &Line| matches!(line, Line::Deadline(_));
    let (turn, number, original) = at_line(&recording, is_deadline);
    let changed =
        if original == Line::Deadline(None) { Line::Deadline(Some(Time::from_nanos(1))) } else { Line::Deadline(None) };
    assert_eq!(
        stopped(&edited(&recording, is_deadline, changed.clone())),
        Stale { turn, line: number, recorded: line_text(&changed), asked: line_text(&original) }
    );
}
#[test]
fn a_departure_not_recorded_is_stale() {
    let recording = walk::walked();
    let (turn, number, navigate) = at_line(&recording, |line| matches!(line, Line::Navigate(_)));
    let mut records = lines(&recording).expect("walk's lines");
    records.remove(usize::try_from(number - 1).expect("index fits"));
    let recorded = line_text(&records[usize::try_from(number - 1).expect("index fits")]);
    assert!(matches!(records.last(), Some(Line::End(_))));
    assert_eq!(stopped(&written(&records)), Stale { turn, line: number, recorded, asked: line_text(&navigate) });
}
#[test]
fn a_control_gone_is_stale_at_its_act() {
    let recording = walk::walked();
    let is_act = |line: &Line| matches!(line, Line::Act { .. });
    let (turn, number, mut act) = at_line(&recording, is_act);
    let Line::Act { control, .. } = &mut act else { unreachable!() };
    control.name = b"Gone".as_slice().into();
    assert_eq!(
        stopped(&edited(&recording, is_act, act.clone())),
        Stale { turn, line: number, recorded: line_text(&act), asked: "no such control".into() }
    );
}
#[test]
fn the_proof_refuses_a_mark_whose_html_differs() {
    let recording = walk::walked();
    let is_mark = |line: &Line| matches!(line, Line::Mark(_));
    let (_, _, mut mark) = at_line(&recording, is_mark);
    let Line::Mark(Some(html)) = &mut mark else { unreachable!() };
    let byte = html.len() / 2;
    html[byte] ^= 1;
    assert_eq!(prove(&edited(&recording, is_mark, mark)), Err(Refusal::Differs { mark: 1, byte }));
}

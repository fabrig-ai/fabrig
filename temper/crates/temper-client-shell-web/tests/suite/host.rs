//! The host's world: the platform's reports given by the test, through the seam
//! (client/web/journeys.md, 2 and 6).
use jig_client_shell_web::adapter::Asked;
use skein_lib::{Time, Wall};
use skein_ui_web::mapping::Primary;
use temper_client_shell_web::host::{self, Duties, Host, Report};

fn started(path: &[u8]) -> Host {
    let mut host = Host::new(
        temper_client_shell_web::limits::client(),
        11,
        Time::ZERO,
        Wall::EPOCH,
        0,
        temper_client_view::sign_in::Ways::NONE,
    );
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
    let mut page = host::page(&host.bounds.tree, Primary::Control);
    host.draw(&mut page, &mut Vec::new());
    let html = page.html();
    assert!(html.contains("temper-top"));
    let (_, main) = html.split_once("<main").expect("main while me is read");
    let (_, children) = main.split_once('>').expect("main opening tag");
    assert!(children.starts_with("</main>"), "main is empty while me is read");
    assert!(!html.contains("temper-needs"));
    assert!(!html.contains("<h1"));
    assert!(!html.contains("temper-filters"));
}

#[test]
fn a_drawn_page_carries_the_views_hrefs() {
    let mut host = started(b"/accounts/connect");
    host.report(Report::Start);
    let mut page = host::page(&host.bounds.tree, Primary::Control);
    host.draw(&mut page, &mut Vec::new());
    let html = page.html();
    assert!(html.split("</a>").any(|anchor| anchor.contains("href=\"/\"") && anchor.contains(">temper<")), "{html}");
}

#[path = "host/redacted.rs"]
mod redacted;
#[path = "host/walk.rs"]
mod walk;
use temper_client_shell_web::journey::{Line, Refusal, Replay, Stale, lines, prove, prove_drawn};

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
    let live = walk::walked();
    let tabs: Vec<_> = lines(&live)
        .expect("walk records")
        .into_iter()
        .filter(|line| matches!(line, Line::TabOpen(_) | Line::TabPoint(_)))
        .collect();
    assert_eq!(
        tabs,
        [Line::TabOpen("Opening ChatGPT’s sign-in…".as_bytes().into()), Line::TabPoint(walk::ADDRESS.into()),]
    );
    let recording = prove(&live).expect("live walk proves");
    let mut replay = Replay::new(&recording).expect("read proved recording");
    let act = replay.next().expect("replay act").expect("act mark");
    assert_eq!(act.number, 1);
    assert!(!act.end);
    assert!(act.recorded.is_none());
    let node = act.tree.find(*act.acted.first().expect("acted control")).expect("control in tree");
    assert_eq!(node.role, skein_ui::Role::Button);
    assert!(act.page.html().contains("Continue with ChatGPT"));
    let end = replay.next().expect("replay end").expect("end mark");
    assert_eq!(end.number, 2);
    assert!(end.end);
    assert!(end.acted.is_empty());
    assert!(end.recorded.is_none());
    assert!(end.page.html().contains("Added your ChatGPT plan."), "the accounts watch completes the sign-in");
    assert!(replay.next().expect("after end").is_none());
}
#[test]
fn the_proof_gives_each_turns_drawn_page() {
    let recording = walk::walked();
    let records = lines(&recording).expect("walk's lines");
    let deadlines = records.iter().filter(|line| matches!(line, Line::Deadline(_))).count();
    let mut turns = Vec::new();
    let mut last_page = None;
    prove_drawn(&recording, &mut |turn, page| {
        turns.push(turn);
        last_page = Some(page.html());
    })
    .expect("live walk proves with drawn pages");
    assert_eq!(turns.len(), deadlines, "one drawn page per deadline");
    assert!(turns.into_iter().eq(1..=u32::try_from(deadlines).expect("turn count fits")));
    let Some(Line::End(Some(html))) = records.last() else { panic!("walk ends with a live print") };
    assert!(last_page.is_some_and(|page| page.as_bytes() == html.as_ref()), "last drawn page is the end print");
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
fn a_tab_not_recorded_is_stale() {
    let recording = walk::walked();
    let cases: [fn(&Line) -> bool; 2] =
        [|line| matches!(line, Line::TabOpen(_)), |line| matches!(line, Line::TabPoint(_))];
    for is_tab in cases {
        let (turn, number, tab) = at_line(&recording, is_tab);
        let mut records = lines(&recording).expect("walk's lines");
        records.remove(usize::try_from(number - 1).expect("index fits"));
        let recorded = line_text(&records[usize::try_from(number - 1).expect("index fits")]);
        assert_eq!(stopped(&written(&records)), Stale { turn, line: number, recorded, asked: line_text(&tab) });
    }
}
#[test]
fn an_event_no_control_gives_is_stale_at_its_act() {
    let recording = walk::walked();
    let is_act = |line: &Line| matches!(line, Line::Act { .. });
    let (turn, number, mut act) = at_line(&recording, is_act);
    let Line::Act { event, .. } = &mut act else { unreachable!() };
    *event = b"Gone".as_slice().into();
    assert_eq!(
        stopped(&edited(&recording, is_act, act.clone())),
        Stale { turn, line: number, recorded: line_text(&act), asked: "no control gives the event".into() }
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

#[test]
fn a_version_1_recording_is_unreadable() {
    let recording = edited(&walk::walked(), |line| matches!(line, Line::Journey { .. }), Line::Journey { version: 1 });
    assert!(matches!(Replay::new(&recording), Err(Refusal::Unreadable(error))
        if error.line == 1 && error.why == "expected ordered start facts for journey 2"));
}
#[test]
fn a_renamed_control_holds() {
    let original = walk::walked();
    let is_act = |line: &Line| matches!(line, Line::Act { .. });
    let (_, _, mut act) = at_line(&original, is_act);
    let Line::Act { name, .. } = &mut act else { unreachable!() };
    *name = b"Renamed".as_slice().into();
    let recording = prove(&edited(&original, is_act, act)).expect("renamed act proves");
    let mut original = Replay::new(&prove(&original).expect("original proves")).expect("original replay");
    let mut renamed = Replay::new(&recording).expect("renamed replay");
    let mut marks = 0;
    while let Some(before) = original.next().expect("original mark") {
        let after = renamed.next().expect("renamed mark").expect("same marks");
        assert_eq!(before.page.html(), after.page.html());
        assert_eq!(before.acted, after.acted);
        marks += 1;
    }
    assert_eq!(marks, 2);
    assert!(renamed.next().expect("after end").is_none());
}
#[test]
fn an_act_with_no_event_keeps_nothing() {
    let recording = walk::walked_with_no_event(true);
    let records = lines(&recording).expect("walk lines");
    assert_eq!(records.iter().filter(|line| matches!(line, Line::Act { .. })).count(), 1);
    assert_eq!(records.iter().filter(|line| matches!(line, Line::Mark(_))).count(), 1);
    prove(&recording).expect("walk without an event's act replays faithfully");
}

#[test]
fn a_save_is_whole_and_the_recording_goes_on() {
    let saves = walk::walked_with_saves(false);
    prove(&saves.first).expect("first act's save proves");
    prove(&saves.later).expect("later save proves");
    prove(&saves.recording).expect("completed sign-in still proves");
    let first = lines(&saves.first).expect("first records");
    let later = lines(&saves.later).expect("later records");
    assert_eq!(first.iter().filter(|line| matches!(line, Line::Act { .. })).count(), 1);
    assert!(later.len() > first.len());
    assert_eq!(&first[..first.len() - 1], &later[..first.len() - 1]);
    assert!(later[first.len() - 1..].iter().any(|line| matches!(line, Line::Heard(_))));
    let completed = lines(&saves.recording).expect("completed records");
    assert!(completed.len() > later.len());
    assert_eq!(&later[..later.len() - 1], &completed[..later.len() - 1]);
}
#[test]
fn a_saves_control_is_found_by_its_event() {
    use skein_ui::{Act, Role};
    use temper_client_shell_web::journey::{Control, control};
    let saves = walk::walked_with_saves(false);
    let records = lines(&saves.recording).expect("records");
    let event = records
        .iter()
        .find_map(|line| if let Line::Act { event, .. } = line { Some(event) } else { None })
        .expect("press event");
    assert_eq!(
        control(&saves.before_press, &Act::Press, event),
        Ok(Some(Control { role: Role::Button, name: b"Continue with ChatGPT".as_slice().into(), order: 0 }))
    );
    assert_eq!(control(&saves.before_press, &Act::Press, b"Gone"), Ok(None));
    let stale = edited(&saves.before_press, |line| matches!(line, Line::Deadline(_)), Line::Deadline(Some(Time::ZERO)));
    assert!(matches!(control(&stale, &Act::Press, event), Err(Refusal::Stale(_))));
    assert!(matches!(control(b"bad", &Act::Press, event), Err(Refusal::Unreadable(_))));
}

#[test]
fn a_saves_control_skips_a_menu_item_for_the_pages_link() {
    use skein_ui::{Act, Role};
    use temper_client_shell_web::journey::{Control, control, giving};
    let mut records = lines(include_bytes!("../../../../docs/design/client/web/journeys/chat-draft/recording"))
        .expect("chat records");
    let at = records
        .iter()
        .position(|line| matches!(line, Line::Act { event, .. } if event.as_ref() == b"Act { action: Pages(Go { address: NewChat { draft: false } }) }"))
        .expect("new chat press");
    records.truncate(at);
    records.push(Line::End(None));
    let save = written(&records);
    let mut replay = Replay::new(&save).expect("chat save");
    while replay.next().expect("chat prefix replays").is_some() {}
    let host = replay.host();
    let event = b"Act { action: Pages(Go { address: Ask { project: 1 } }) }";
    let nodes = giving(host, &Act::Press, event);
    assert!(nodes.iter().any(|node| Control::of(host.tree(), *node).expect("control").role == Role::MenuItem));
    assert_eq!(
        control(&save, &Act::Press, event),
        Ok(Some(Control { role: Role::Link, name: b"Ask temper".as_slice().into(), order: 0 }))
    );
    let draft = b"Act { action: Pages(Go { address: NewChat { draft: true } }) }";
    assert_eq!(giving(host, &Act::Press, draft).len(), 1, "only the menu offers a new draft");
    assert_eq!(control(&save, &Act::Press, draft), Ok(None));
}

#[test]
fn a_refused_tab_replays_without_pointing() {
    let recording = walk::walked_with_refused_tab();
    let records = lines(&recording).expect("refused tab records");
    assert_eq!(records.iter().filter(|line| matches!(line, Line::TabOpen(_))).count(), 1);
    assert_eq!(records.iter().filter(|line| matches!(line, Line::TabRefused)).count(), 1);
    assert!(!records.iter().any(|line| matches!(line, Line::TabPoint(_))));
    prove(&recording).expect("refused tab walk proves and replays");
}

#[test]
fn an_authorization_departure_ends_the_recording() {
    let recording = walk::departed();
    prove(&recording).expect("authorization departure proves");
    let (turn, number, _) = at_line(&recording, |line| matches!(line, Line::Replace(_)));
    let mut records = lines(&recording).expect("authorization records");
    let at = usize::try_from(number).expect("index fits");
    assert!(matches!(records[at], Line::End(_)), "departure ends before any more reports or drawing");
    records.insert(at, Line::Due);
    assert_eq!(
        stopped(&written(&records)),
        Stale { turn, line: number + 1, recorded: "due".into(), asked: "end".into() }
    );
}

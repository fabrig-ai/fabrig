use super::*;

fn fake() -> Forge {
    Forge { number: 0, playground: true, kind: Kind::Forgejo, ..records::forge() }
}
fn configured() -> Domain {
    let mut d = domain();
    d.under_box = true;
    d.configure_demo(Some(DemoConfig {
        forge: Some(fake()),
        provider: 1,
        adoption: fixtures::adoption(),
        deadline: Duration::from_secs(10),
    }));
    d
}
fn start(d: &mut Domain, number: u64) -> Box<[Request]> {
    outputs(
        d,
        Event::Application {
            role: None,
            carried: None,
            request: Token::new(number),
            token: Token::new(number.checked_add(100).expect("fixture token")),
            retention: Duration::from_secs(60),
            ask: Ask::StartPlayground,
        },
    )
}
fn answer(d: &Domain, number: u64) -> Option<Answered> {
    d.application_answer(Token::new(number.checked_add(100).expect("fixture token")))
}
fn row(forge: Option<u16>, project: Option<u32>, account: Option<u32>) -> Playground {
    Playground { state: PlaygroundState::Running, forge, project, account }
}
fn save(row: Playground) -> Request {
    Request::Save { record: Stored::Playground(row), read_afresh: None, release: None }
}
fn not_offered(mut d: Domain) {
    assert_eq!(
        start(&mut d, 1).as_ref(),
        [Request::ApplicationDone { request: Token::new(1), token: Token::new(101) }]
    );
    assert_eq!(answer(&d, 1), Some(Answered::StartPlayground(StartAnswer::NotOffered)));
}
#[test]
fn no_demo_config_is_not_offered() {
    not_offered(domain());
}
#[test]
fn a_kept_real_forge_withdraws_the_playground() {
    let mut d = configured();
    outputs(&mut d, Event::Restore { record: Stored::Forge(records::forge()) });
    not_offered(d);
}
#[test]
fn a_resetting_playground_is_not_offered() {
    let mut d = configured();
    outputs(
        &mut d,
        Event::Restore {
            record: Stored::Playground(Playground { state: PlaygroundState::Resetting, ..row(None, None, None) }),
        },
    );
    not_offered(d);
}
#[test]
fn a_first_start_saves_the_row_and_makes_its_forge() {
    let mut d = configured();
    let sent = start(&mut d, 1);
    let forge = Forge { number: 1, identity: record::Identity { number: 1, ..fake().identity }, ..fake() };
    assert_eq!(
        sent.as_ref(),
        [
            save(row(None, None, None)),
            Request::Save { record: Stored::Forge(forge.clone()), read_afresh: None, release: None },
            Request::Save { record: Stored::Forges { next: 2 }, read_afresh: None, release: None },
            Request::MarkedForges,
            Request::ForgeKept { forge: Box::new(forge) },
            save(row(Some(1), None, None)),
            Request::SetUpGroup { forge: 1, request: Token::new(1) },
            Request::PlaygroundStart { request: Token::new(1) },
        ]
    );
    assert_eq!(d.playground(), Some(&row(Some(1), None, None)));
}
#[test]
fn a_seeded_playground_forge_is_taken_not_made() {
    let mut d = configured();
    let forge = Forge { number: 7, ..fake() };
    outputs(&mut d, Event::Restore { record: Stored::Forge(forge) });
    assert_eq!(
        start(&mut d, 1).as_ref(),
        [
            save(row(None, None, None)),
            save(row(Some(7), None, None)),
            Request::SetUpGroup { forge: 7, request: Token::new(1) },
            Request::PlaygroundStart { request: Token::new(1) },
        ]
    );
}
#[test]
fn a_refused_make_fails_the_start_and_leaves_no_row() {
    for (forge, why) in [(None, b"invalid".as_slice()), (Some(fake()), b"exhausted".as_slice())] {
        let mut d = configured();
        d.demo.as_mut().unwrap().forge = forge;
        outputs(&mut d, Event::Restore { record: Stored::Forges { next: u16::MAX } });
        let sent = start(&mut d, 1);
        assert_eq!(
            sent.as_ref(),
            [
                save(row(None, None, None)),
                Request::Erase { key: Key::Playground },
                Request::ApplicationDone { request: Token::new(1), token: Token::new(101) }
            ]
        );
        assert_eq!(
            answer(&d, 1),
            Some(Answered::StartPlayground(StartAnswer::Failed(StartFailure::Group { why: why.into() })))
        );
        assert_eq!(d.playground(), None);
    }
}
#[test]
fn made_steps_are_saved_in_the_row() {
    let mut d = configured();
    start(&mut d, 1);
    assert_eq!(
        outputs(&mut d, Event::PlaygroundMade { project: Some(2), account: None }).as_ref(),
        [save(row(Some(1), Some(2), None))]
    );
    assert_eq!(
        outputs(&mut d, Event::PlaygroundMade { project: None, account: Some(3) }).as_ref(),
        [save(row(Some(1), Some(2), Some(3)))]
    );
    assert_eq!(d.playground(), Some(&row(Some(1), Some(2), Some(3))));
}
#[test]
fn pending_starts_share_the_answer_and_resume_without_making_again() {
    let mut d = configured();
    start(&mut d, 1);
    assert!(start(&mut d, 2).is_empty());
    let sent = outputs(&mut d, Event::PlaygroundAnswered { answer: StartAnswer::Running });
    assert_eq!(
        sent.as_ref(),
        [
            Request::ApplicationDone { request: Token::new(1), token: Token::new(101) },
            Request::ApplicationDone { request: Token::new(2), token: Token::new(102) },
        ]
    );
    for number in [1, 2] {
        assert_eq!(answer(&d, number), Some(Answered::StartPlayground(StartAnswer::Running)));
    }
    assert_eq!(
        start(&mut d, 3).as_ref(),
        [Request::SetUpGroup { forge: 1, request: Token::new(3) }, Request::PlaygroundStart { request: Token::new(3) },]
    );
}

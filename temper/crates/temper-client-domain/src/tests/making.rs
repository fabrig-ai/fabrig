use super::{
    connect::{event, opened, reply, sent},
    fixtures as f,
};
use crate::{
    Address, Choice, Event, Place, Request, Step,
    forge::{
        self,
        making::{MakeAnswer, MakeChoice},
    },
};
use temper_client_protocol_wire::v1 as w;

fn made(forge: u16) -> w::Outcome {
    w::Outcome::Made(w::MadeForge::new(&w::CEILINGS, w::MadeForgeParts { forge }).unwrap())
}
fn go(root: &mut crate::Domain, address: Address) {
    event(root, Event::Went { place: Place::Address(address) });
}
fn replaced(out: &skein_lib::Queue<Request>, expected: Address) {
    let mut found = false;
    for request in out {
        if let Request::Show { address, replace: true } = request
            && *address == expected
        {
            found = true;
        }
    }
    assert!(found, "made forge replaces the sending page's address");
}
fn no_navigation(out: &skein_lib::Queue<Request>) {
    for request in out {
        if let Request::Show { .. } = request {
            panic!("the person left the sending page");
        }
    }
}
#[test]
fn here_with_no_record_makes_then_sets_up_the_made_number() {
    let (mut root, _, _) = opened(true);
    go(&mut root, Address::ConnectForge { forge: Some(Choice::Here) });
    for (field, text) in [
        (forge::forges::SetupField::Organization, b"team".as_slice()),
        (forge::forges::SetupField::Repository, b"sample"),
        (forge::forges::SetupField::Project, b"Project"),
    ] {
        f::form_act(&mut root, forge::Act::SetupWrite { field, text: text.into() });
    }
    let (attempt, _, w::Request::MakeForge(request)) = sent(f::form_act(&mut root, forge::Act::MakeAndSetUp)) else {
        panic!("make forge")
    };
    assert_eq!(request.choice(), &w::ForgeChoice::Owned);
    let out = reply(&mut root, attempt, made(9));
    replaced(&out, Address::ConnectForge { forge: Some(Choice::Forge(9)) });
    let (_, _, w::Request::SetUp(request)) = sent(out) else { panic!("set up") };
    assert_eq!((request.forge(), request.org(), request.repository()), (9, b"team".as_slice(), b"sample".as_slice()));
    let w::SetUpProject::New(project) = request.project() else { panic!("new project") };
    assert_eq!(project.name(), b"Project");
}
#[test]
fn the_welcomes_run_here_makes_then_sets_up_at_once_and_moves_to_model() {
    let (mut root, _, _) = opened(true);
    go(&mut root, Address::Welcome { step: Step::Forge { forge: None } });
    let (attempt, _, w::Request::MakeForge(request)) = sent(f::form_act(&mut root, forge::Act::MakeAndSetUpAtOnce))
    else {
        panic!("make forge")
    };
    assert_eq!(request.choice(), &w::ForgeChoice::Owned);
    let out = reply(&mut root, attempt, made(7));
    let address = Address::Welcome { step: Step::Forge { forge: Some(Choice::Forge(7)) } };
    replaced(&out, address.clone());
    let (attempt, _, w::Request::SetUp(request)) = sent(out) else { panic!("set up") };
    assert_eq!((request.forge(), request.org(), request.repository()), (7, b"work".as_slice(), b"hello".as_slice()));
    let w::SetUpProject::New(project) = request.project() else { panic!("new project") };
    assert_eq!(project.name(), b"Default");
    go(&mut root, address);
    let out = reply(&mut root, attempt, w::Outcome::SettingUp);
    let mut moved = false;
    for request in &out {
        if let Request::Show { address: Address::Welcome { step: Step::Model { way: None } }, replace: false } = request
        {
            moved = true;
        }
    }
    assert!(moved, "accepted setup opens Model");
}
#[test]
fn github_with_no_record_makes_then_sends_its_manifest_connect() {
    let (mut root, _, _) = opened(true);
    go(&mut root, Address::ConnectForge { forge: Some(Choice::GitHub) });
    let back = b"/forges/connect?forge=github";
    let (attempt, _, w::Request::MakeForge(request)) =
        sent(f::form_act(&mut root, forge::Act::MakeAndConnectGitHub { back: back.as_slice().into() }))
    else {
        panic!("make forge")
    };
    assert_eq!(request.choice(), &w::ForgeChoice::Github);
    let out = reply(&mut root, attempt, made(8));
    replaced(&out, Address::ConnectForge { forge: Some(Choice::Forge(8)) });
    let (_, _, w::Request::Connect(request)) = sent(out) else { panic!("connect") };
    assert_eq!(request.forge(), 8);
    let w::How::GithubManifest(manifest) = request.how() else { panic!("manifest") };
    assert_eq!(manifest.back().as_deref(), Some(back.as_slice()));
}
#[test]
fn elsewhere_makes_a_forgejo_from_its_fields_and_opens_its_number() {
    let (mut root, _, _) = opened(true);
    go(&mut root, Address::ConnectForge { forge: Some(Choice::Elsewhere) });
    for (field, text) in [
        (forge::making::Field::Api, b"forge.example:443".as_slice()),
        (forge::making::Field::Git, b"https://forge.example"),
        (forge::making::Field::Web, b"https://web.example"),
    ] {
        f::form_act(&mut root, forge::Act::ElsewhereWrite { field, text: text.into() });
    }
    let (attempt, _, w::Request::MakeForge(request)) = sent(f::form_act(&mut root, forge::Act::MakeElsewhere)) else {
        panic!("make forge")
    };
    let w::ForgeChoice::Forgejo(choice) = request.choice() else { panic!("forgejo") };
    assert_eq!(
        (choice.api(), choice.git(), choice.web().as_deref()),
        (b"forge.example:443".as_slice(), b"https://forge.example".as_slice(), Some(b"https://web.example".as_slice()))
    );
    let out = reply(&mut root, attempt, made(5));
    replaced(&out, Address::ConnectForge { forge: Some(Choice::Forge(5)) });
    let mut out = out;
    assert_eq!(f::sends(&mut out), 0);
}
#[test]
fn an_empty_elsewhere_address_sends_nothing() {
    for field in [forge::making::Field::Api, forge::making::Field::Git] {
        let (mut root, _, _) = opened(true);
        f::form_act(&mut root, forge::Act::ElsewhereWrite { field, text: b"address".as_slice().into() });
        assert!(f::form_act(&mut root, forge::Act::MakeElsewhere).is_empty());
        assert!(root.forge().elsewhere_form().checked);
        assert!(root.forge().making().is_none());
    }
}
#[test]
fn full_and_not_offered_are_kept_and_send_nothing_more() {
    for (outcome, answer) in [(w::Outcome::Full, MakeAnswer::Full), (w::Outcome::NotOffered, MakeAnswer::NotOffered)] {
        let (mut root, _, _) = opened(true);
        let (attempt, _, _) = sent(f::form_act(&mut root, forge::Act::MakeAndSetUpAtOnce));
        let mut out = reply(&mut root, attempt, outcome);
        assert_eq!(f::sends(&mut out), 0);
        no_navigation(&out);
        let making = root.forge().making().unwrap();
        assert_eq!(making.answer, Some(answer));
        assert_eq!(making.progress, None);
        assert!(root.forge().set_up().is_none());
    }
}
#[test]
fn one_make_at_a_time() {
    let (mut root, _, _) = opened(true);
    sent(f::form_act(&mut root, forge::Act::MakeAndSetUpAtOnce));
    assert!(f::form_act(&mut root, forge::Act::MakeAndConnectGitHub { back: Box::default() }).is_empty());
    assert_eq!(root.forge().making().unwrap().choice, MakeChoice::Here);
}
#[test]
fn leaving_the_flow_keeps_the_follow_up_without_navigation() {
    for address in
        [Address::Forges, Address::ConnectForge { forge: None }, Address::Welcome { step: Step::Forge { forge: None } }]
    {
        let (mut root, _, _) = opened(true);
        go(&mut root, Address::ConnectForge { forge: Some(Choice::Here) });
        let (attempt, _, _) = sent(f::form_act(&mut root, forge::Act::MakeAndSetUpAtOnce));
        go(&mut root, address);
        let out = reply(&mut root, attempt, made(9));
        no_navigation(&out);
        let w::Request::SetUp(_) = sent(out).2 else { panic!("setup follows the make") };
    }
}
#[test]
fn signing_out_clears_the_attempt_and_address_draft() {
    let (mut root, _, _) = opened(true);
    f::form_act(
        &mut root,
        forge::Act::ElsewhereWrite { field: forge::making::Field::Api, text: b"forge:80".as_slice().into() },
    );
    sent(f::form_act(&mut root, forge::Act::MakeAndSetUpAtOnce));
    event(&mut root, Event::SignedOut);
    assert!(root.forge().making().is_none());
    assert!(root.forge().elsewhere_form().api.is_empty());
}

#[test]
fn making_parks_out_of_date_and_refuses_signed_out_or_stale_sends() {
    let mut root = crate::unlinked(&f::limits(), f::SEED);
    assert!(f::form_act(&mut root, forge::Act::MakeAndSetUpAtOnce).is_empty());
    let (mut root, _, _) = opened(true);
    sent(f::form_act(&mut root, forge::Act::MakeAndSetUpAtOnce));
    event(&mut root, Event::Forge(forge::Event::Served { served: forge::Served::OutOfDate }));
    assert_eq!(root.forge().making().unwrap().progress, Some(skein_lib::RequestProgress::Parked));
    assert!(f::form_act(&mut root, forge::Act::MakeAndSetUpAtOnce).is_empty());
    let (mut root, _, _) = opened(true);
    event(&mut root, Event::Forge(forge::Event::Served { served: forge::Served::OutOfDate }));
    assert!(f::form_act(&mut root, forge::Act::MakeAndSetUpAtOnce).is_empty());
    assert!(root.forge().making().is_none());
}
#[test]
fn the_drafted_setup_stays_with_the_make_while_it_is_pending() {
    let (mut root, _, _) = opened(true);
    let (attempt, _, _) = sent(f::form_act(&mut root, forge::Act::MakeAndSetUp));
    f::form_act(
        &mut root,
        forge::Act::SetupWrite { field: forge::forges::SetupField::Repository, text: b"changed".as_slice().into() },
    );
    let (_, _, w::Request::SetUp(request)) = sent(reply(&mut root, attempt, made(9))) else { panic!("setup") };
    assert_eq!(request.repository(), b"hello");
}
#[test]
fn elsewhere_omits_an_unwritten_web_address() {
    let (mut root, _, _) = opened(true);
    for field in [forge::making::Field::Api, forge::making::Field::Git] {
        f::form_act(&mut root, forge::Act::ElsewhereWrite { field, text: b"address".as_slice().into() });
    }
    let (_, _, w::Request::MakeForge(request)) = sent(f::form_act(&mut root, forge::Act::MakeElsewhere)) else {
        panic!("make")
    };
    let w::ForgeChoice::Forgejo(choice) = request.choice() else { panic!("forgejo") };
    assert!(choice.web().is_none());
}
#[test]
fn unknown_make_answers_end_the_attempt_without_a_follow_up() {
    let (mut root, _, _) = opened(true);
    let (attempt, _, _) = sent(f::form_act(&mut root, forge::Act::MakeAndSetUpAtOnce));
    let mut out = reply(&mut root, attempt, w::Outcome::Unknown);
    assert_eq!(f::sends(&mut out), 0);
    assert_eq!(root.forge().making().unwrap().answer, Some(MakeAnswer::Unknown));
    assert!(root.forge().making().unwrap().progress.is_none());
    assert!(root.forge().set_up().is_none());
}
#[test]
fn make_is_deployment_scoped_and_decodes_only_its_answers() {
    use forge::making::{answer, ask};
    let limits = f::limits().forge;
    let jig_client::Ask::Family { scope, .. } = ask(MakeChoice::Here, &limits).unwrap() else { panic!("family") };
    assert_eq!(scope, jig_client::contract::Scope::Deployment);
    assert!(
        ask(MakeChoice::Forgejo { api: Box::default(), git: b"git".as_slice().into(), web: None }, &limits).is_none()
    );
    let mut record = jig_client::Family {
        family: temper_client_protocol_wire::FORGE,
        version: temper_client_protocol_wire::CLIENT_VERSION,
        bytes: temper_client_protocol_wire::answer(
            &w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome: made(8) }).unwrap(),
        ),
    };
    assert_eq!(answer(&jig_client::Answered::Family(record.clone()), &limits), Some(MakeAnswer::Made { forge: 8 }));
    record.version = record.version.checked_add(1).unwrap();
    assert_eq!(answer(&jig_client::Answered::Family(record.clone()), &limits), None);
    record.version = temper_client_protocol_wire::CLIENT_VERSION;
    record.family = 2;
    assert_eq!(answer(&jig_client::Answered::Family(record.clone()), &limits), None);
    record.family = temper_client_protocol_wire::FORGE;
    record.bytes = temper_client_protocol_wire::answer(
        &w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome: w::Outcome::SettingUp }).unwrap(),
    );
    assert_eq!(answer(&jig_client::Answered::Family(record), &limits), None);
}

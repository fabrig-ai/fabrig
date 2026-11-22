//! Service reissue checks in people (domain/people/acts.md, 1).
use super::*;

const OWNER: u64 = 1;
const OTHER_PERSON: u64 = 2;
const SERVICE: u64 = 9;
const UNKNOWN_PARTY: u64 = 99;
const OWNER_SIGN_IN: u64 = 10;
const OTHER_SIGN_IN: u64 = 11;
const SERVICE_SIGN_IN: u64 = 20;
const PROJECT: u32 = 1;
const OTHER_PROJECT: u32 = 2;
const DEPLOYMENT_PROVIDER: u16 = 2; // Test::new
const MAKE: u8 = 1;
const REISSUE: u8 = 2;

fn with_service(limits: Limits) -> Test {
    let mut test = Test::new(limits);
    test.signin(OWNER, OWNER_SIGN_IN, identity(0, OWNER));
    set_roles(&mut test, PROJECT, &[(OWNER, Role::Owner)]);
    let make = Ask::MakeService { project: PROJECT, name: b"build".as_slice().into(), role: Role::Member };
    let request = route(&test.request(OWNER_SIGN_IN, MAKE, make));
    test.send(Event::MakeService { request, person: SERVICE });
    test.send(Event::Decided { request, outcome: Outcome::ServiceMade { person: SERVICE } });
    sign_in_service(&mut test, SERVICE_SIGN_IN);
    test
}

fn sign_in_service(test: &mut Test, sign_in: u64) {
    let reply_to = test.to();
    test.send(Event::SignedIn {
        until: None,
        held: None,
        reply_to,
        person: SERVICE,
        sign_in,
        identity: identity(DEPLOYMENT_PROVIDER, SERVICE),
        kind: Kind::Service,
    });
}

fn set_roles(test: &mut Test, project: u32, roles: &[(u64, Role)]) {
    let holdings = roles.iter().map(|&(person, role)| Holding { person, role }).collect();
    test.send(Event::Roles { project, holdings });
}

fn check_refusal(test: &mut Test, sign_in: u64, party: u64, why: Refusal) {
    let request = route(&test.request(sign_in, REISSUE, Ask::ReissueService { party }));
    let expected =
        Request::Reply { to: ReplyTo::new(Token::new(test.serial)), reply: Reply::Outcome(Outcome::Refused(why)) };
    assert_eq!(test.send(Event::ReissueService { request }), [expected]);
}

#[test]
fn reissue_erases_every_service_sign_in_before_answering_all_waiters() {
    // One sign-in for the owner, the rest for the service, and MakeService fills the answer table.
    let limits = Limits { sign_ins: 8, waiters: 3, requests: 1, ..LIMITS };
    let mut test = with_service(limits);
    set_roles(&mut test, DEPLOYMENT, &[(OWNER, Role::Owner)]);
    let sign_ins = SERVICE_SIGN_IN..SERVICE_SIGN_IN + u64::from(limits.sign_ins - 1);
    for sign_in in sign_ins.clone().skip(1) {
        sign_in_service(&mut test, sign_in);
    }

    let ask = Ask::ReissueService { party: SERVICE };
    let first = test.request(OWNER_SIGN_IN, REISSUE, ask.clone());
    let request = route(&first);
    let mut expected: Vec<_> = sign_ins.map(|number| Request::Erase { key: Key::SignIn(number) }).collect();
    let outcome = Reply::Outcome(Outcome::ServiceReissued { party: SERVICE });
    expected.push(Request::Reply { to: ReplyTo::new(Token::new(test.serial)), reply: outcome.clone() });
    for _ in 1..limits.waiters {
        test.request(OWNER_SIGN_IN, REISSUE, ask.clone());
        expected.push(Request::Reply { to: ReplyTo::new(Token::new(test.serial)), reply: outcome.clone() });
    }

    let decision = test.send(Event::ReissueService { request });
    assert_eq!(decision, expected);
    assert!(decision.len() <= usize::try_from(outputs_bound(&limits)).expect("output count fits usize"));
}

#[test]
fn reissue_checks_existence_before_the_askers_role() {
    let mut test = with_service(LIMITS);
    check_refusal(&mut test, OWNER_SIGN_IN, UNKNOWN_PARTY, Refusal::Unknown);
}

#[test]
fn reissue_is_not_offered_for_people_or_self() {
    for (sign_in, party) in [(OWNER_SIGN_IN, OTHER_PERSON), (SERVICE_SIGN_IN, SERVICE)] {
        let mut test = with_service(LIMITS);
        test.signin(OTHER_PERSON, OTHER_SIGN_IN, identity(0, OTHER_PERSON));
        set_roles(&mut test, DEPLOYMENT, &[(OWNER, Role::Owner), (SERVICE, Role::Owner)]);
        check_refusal(&mut test, sign_in, party, Refusal::NotOffered);
    }
}

#[test]
fn reissue_requires_ownership_of_every_holding_project() {
    let one_holding = [(SERVICE, Role::Member)];
    let member = [(OWNER, Role::Member), (SERVICE, Role::Member)];
    let project_owner = [(OWNER, Role::Owner), (SERVICE, Role::Member)];
    for (project, other, deployment, held) in [
        (member.as_slice(), &[][..], &[][..], None),
        (&project_owner, &one_holding, &[], None),
        (&project_owner, &one_holding, &[(OWNER, Role::Member)], Some(Role::Member)),
        (&[], &[], &[], None),
    ] {
        let mut test = with_service(LIMITS);
        set_roles(&mut test, PROJECT, project);
        set_roles(&mut test, OTHER_PROJECT, other);
        set_roles(&mut test, DEPLOYMENT, deployment);
        check_refusal(&mut test, OWNER_SIGN_IN, SERVICE, Refusal::Role { held });
    }
}

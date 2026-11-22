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

fn with_service() -> Test {
    let mut test = Test::new(LIMITS);
    test.signin(OWNER, OWNER_SIGN_IN, identity(0, OWNER));
    set_roles(&mut test, PROJECT, &[(OWNER, Role::Owner)]);
    let make = Ask::MakeService { project: PROJECT, name: b"build".as_slice().into(), role: Role::Member };
    let request = route(&test.request(OWNER_SIGN_IN, MAKE, make));
    test.send(Event::MakeService { request, person: SERVICE });
    test.send(Event::Decided { request, outcome: Outcome::ServiceMade { person: SERVICE } });
    let reply_to = test.to();
    test.send(Event::SignedIn {
        until: None,
        held: None,
        reply_to,
        person: SERVICE,
        sign_in: SERVICE_SIGN_IN,
        identity: identity(DEPLOYMENT_PROVIDER, SERVICE),
        kind: Kind::Service,
    });
    test
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
fn reissue_checks_existence_before_the_askers_role() {
    let mut test = with_service();
    check_refusal(&mut test, OWNER_SIGN_IN, UNKNOWN_PARTY, Refusal::Unknown);
}

#[test]
fn reissue_is_not_offered_for_people_or_self() {
    for (sign_in, party) in [(OWNER_SIGN_IN, OTHER_PERSON), (SERVICE_SIGN_IN, SERVICE)] {
        let mut test = with_service();
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
        let mut test = with_service();
        set_roles(&mut test, PROJECT, project);
        set_roles(&mut test, OTHER_PROJECT, other);
        set_roles(&mut test, DEPLOYMENT, deployment);
        check_refusal(&mut test, OWNER_SIGN_IN, SERVICE, Refusal::Role { held });
    }
}

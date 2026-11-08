//! Team's plain facts and parts (temper/docs/design/client/README.md, 3.5, Team).
use crate::pages::team::{self, Facts, Parts, Person, Waits};
use jig_client::contract as c;
use skein_lib::List;

// Names and logins in this fixture fit within this text bound.
const NAME_BYTES: u32 = 64;
const PROJECT: u32 = 3;
const YOU: u64 = 7;
const SAM: u64 = 8;
const UNLISTED: u64 = 9;
const YOURS: Waits = Waits { questions: 2, reviews: 1, proposals: 3, held: 4, person_tasks: 5, cut: false };
const SAMS: Waits = Waits { questions: 1, reviews: 0, proposals: 0, held: 0, person_tasks: 0, cut: true };

fn text(bytes: &[u8]) -> c::DocumentText {
    c::DocumentText::complete(c::Text::new(bytes.into(), NAME_BYTES).unwrap())
}
fn member(party: u64, kind: c::PartyKind, name: &[u8], role: c::HeldRole) -> c::ProjectMember {
    c::ProjectMember { party, kind, role, login: text(b"login"), name: text(name) }
}
fn person(party: u64, you: bool, name: &[u8], role: c::HeldRole, waits: Option<Waits>) -> Person {
    Person { party, you, name: name.into(), role, waits }
}
fn project(members: Box<[c::ProjectMember]>, cut: bool) -> c::Project {
    c::Project {
        acts: List::with_capacity(0),
        number: PROJECT,
        name: text(b"Project"),
        resources: List::with_capacity(0),
        policy: c::ProjectPolicy {
            templates: List::with_capacity(0),
            ceiling: c::AuthorityDocument::complete(c::Authority {
                models: None,
                tools: 0,
                grants: List::with_capacity(0),
                delegation: c::Delegation { kinds: List::with_capacity(0), tasks: 0, depth: 0 },
                spend: 0,
                deadline: None,
                notes: 0,
                note_resources: List::with_capacity(0),
            }),
            period_spend: 0,
            escalation_role: None,
            projections: List::with_capacity(0),
            roles: List::with_capacity(0),
            requirements: List::with_capacity(0),
            permissions: None,
            permissions_cut: false,
            cut: false,
        },
        members: List::from_boxed(members),
        current_period: None,
        pools: List::with_capacity(0),
        configured_period: 0,
        configured_pools: List::with_capacity(0),
        configured_pools_cut: false,
        spend: List::with_capacity(0),
        recurring: List::with_capacity(0),
        cut,
    }
}

#[test]
fn gather_lists_each_person_once_the_reader_first() {
    use c::{
        HeldRole::{Member, Observer, Owner},
        PartyKind::{Person as Human, Service},
    };
    let you = member(YOU, Human, b"Reader", Owner);
    let sam = member(SAM, Human, b"Sam", Member);
    let reader = person(YOU, true, b"Reader", Owner, None);
    let teammate = person(SAM, false, b"Sam", Member, None);
    for (name, members, yours, others, cut, expected) in [
        (
            "reader first",
            Box::<[_]>::from([sam.clone(), you.clone()]),
            None,
            Box::<[_]>::from([]),
            false,
            Box::<[_]>::from([reader.clone(), teammate.clone()]),
        ),
        (
            "service left out",
            Box::<[_]>::from([member(SAM, Service, b"Service", Owner)]),
            None,
            Box::<[_]>::from([]),
            false,
            Box::<[_]>::from([]),
        ),
        (
            "first duplicate kept",
            Box::<[_]>::from([
                sam.clone(),
                member(SAM, Human, b"Later", Observer),
                you.clone(),
                member(YOU, Human, b"Later reader", Member),
            ]),
            None,
            Box::<[_]>::from([]),
            false,
            Box::<[_]>::from([reader.clone(), teammate.clone()]),
        ),
        (
            "empty name uses login",
            Box::<[_]>::from([member(SAM, Human, b"", Observer)]),
            None,
            Box::<[_]>::from([]),
            false,
            Box::<[_]>::from([person(SAM, false, b"login", Observer, None)]),
        ),
        (
            "reader and other waits, first pair kept",
            Box::<[_]>::from([sam.clone(), you.clone()]),
            Some(YOURS),
            Box::<[_]>::from([(SAM, SAMS), (SAM, YOURS), (YOU, SAMS)]),
            false,
            Box::<[_]>::from([
                person(YOU, true, b"Reader", Owner, Some(YOURS)),
                person(SAM, false, b"Sam", Member, Some(SAMS)),
            ]),
        ),
        (
            "unlisted pair dropped",
            Box::<[_]>::from([you.clone()]),
            None,
            Box::<[_]>::from([(UNLISTED, SAMS)]),
            false,
            Box::<[_]>::from([reader.clone()]),
        ),
        (
            "reader is no member",
            Box::<[_]>::from([sam.clone()]),
            Some(YOURS),
            Box::<[_]>::from([]),
            false,
            Box::<[_]>::from([teammate.clone()]),
        ),
        ("cut copied", Box::<[_]>::from([you]), None, Box::<[_]>::from([]), true, Box::<[_]>::from([reader])),
    ] {
        assert_eq!(
            team::gather(&project(members, cut), YOU, yours, &others),
            Facts { people: List::from_boxed(expected), cut, triager: None, standing_roles: None },
            "{name}",
        );
    }
}

#[test]
fn parts_show_what_the_facts_hold() {
    for (name, people, triager, expected) in [
        ("no people or triager", Box::<[_]>::from([]), None, Parts { people: false, triager: false }),
        (
            "people without waits",
            Box::<[_]>::from([person(YOU, true, b"Reader", c::HeldRole::Owner, None)]),
            None,
            Parts { people: true, triager: false },
        ),
        ("triager alone", Box::<[_]>::from([]), Some(b"Sam".as_slice()), Parts { people: false, triager: true }),
        (
            "both parts",
            Box::<[_]>::from([person(SAM, false, b"Sam", c::HeldRole::Member, Some(SAMS))]),
            Some(b"Reader".as_slice()),
            Parts { people: true, triager: true },
        ),
    ] {
        let facts = Facts {
            people: List::from_boxed(people),
            cut: true,
            triager: match triager {
                Some(name) => Some(name.into()),
                None => None,
            },
            standing_roles: Some(1),
        };
        assert_eq!(team::parts(&facts), expected, "{name}");
    }
}

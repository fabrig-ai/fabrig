//! The rail's rules and domain facts (temper/docs/design/client/README.md, 5.3).
use super::{
    fixtures as f,
    welcome::{deliver, event, up},
};
use crate::{
    Address, Domain, Event, Place, Request,
    address::Filter,
    pages::rail::{self, Facts, New, Parts},
};
use jig_client::{Down, Readable, Readout, Up, Watchable, contract as c};
use skein_lib::{List, Writer};
use temper_client_protocol_wire::{self as wire, v1 as w};

const OTHER: u32 = 2;
// The fixture lists the open project and OTHER.
const PROJECTS: u32 = 2;
// One delivered need; the count's cut is independent of its value.
const NEEDS: u32 = 1;
const BASE: Facts = Facts {
    projects: 0,
    needs: 0,
    needs_cut: false,
    members: None,
    standing_roles: None,
    asking: false,
    away: false,
    drafts: false,
    beginning: false,
};
const ORDINARY: Parts = Parts {
    all_projects: false,
    needs_you: true,
    count: 0,
    cut: false,
    work: true,
    team: false,
    ask: false,
    how: true,
    away: false,
    new: New::Change,
};

#[test]
fn parts_follow_each_rails_rule_and_to_begin_cut() {
    for (name, facts, expected) in [
        ("no facts", BASE, ORDINARY),
        ("one project", Facts { projects: 1, ..BASE }, ORDINARY),
        ("two projects", Facts { projects: PROJECTS, ..BASE }, Parts { all_projects: true, ..ORDINARY }),
        ("needs count", Facts { needs: NEEDS, ..BASE }, Parts { count: NEEDS, ..ORDINARY }),
        ("zero cut", Facts { needs_cut: true, ..BASE }, Parts { cut: true, ..ORDINARY }),
        ("count floor", Facts { needs: NEEDS, needs_cut: true, ..BASE }, Parts { count: NEEDS, cut: true, ..ORDINARY }),
        ("no members or roles", Facts { members: Some(0), standing_roles: Some(0), ..BASE }, ORDINARY),
        ("one member", Facts { members: Some(1), ..BASE }, ORDINARY),
        ("second member", Facts { members: Some(2), ..BASE }, Parts { team: true, ..ORDINARY }),
        ("standing role", Facts { standing_roles: Some(1), ..BASE }, Parts { team: true, ..ORDINARY }),
        (
            "both team facts",
            Facts { members: Some(2), standing_roles: Some(1), ..BASE },
            Parts { team: true, ..ORDINARY },
        ),
        ("asking", Facts { asking: true, ..BASE }, Parts { ask: true, ..ORDINARY }),
        ("away", Facts { away: true, ..BASE }, Parts { away: true, ..ORDINARY }),
        ("drafts", Facts { drafts: true, ..BASE }, Parts { new: New::DraftOrChange, ..ORDINARY }),
        (
            "to begin",
            Facts { beginning: true, ..BASE },
            Parts { needs_you: false, work: false, how: false, new: New::Hidden, ..ORDINARY },
        ),
        (
            "to begin with every fact",
            Facts {
                projects: PROJECTS,
                needs: NEEDS,
                needs_cut: true,
                members: Some(2),
                standing_roles: Some(1),
                asking: true,
                away: true,
                drafts: true,
                beginning: true,
            },
            Parts {
                all_projects: true,
                needs_you: false,
                count: NEEDS,
                cut: true,
                work: false,
                team: false,
                ask: true,
                how: false,
                away: true,
                new: New::Hidden,
            },
        ),
    ] {
        assert_eq!(rail::parts(&facts), expected, "{name}");
    }
}

fn two_projects() -> c::Me {
    let mut me = f::chats::me();
    let mut owner = me.projects.get(0).expect("fixture project").clone();
    let mut other = owner.clone();
    other.project = OTHER;
    owner.templates.get_mut(0).expect("fixture template").number = crate::pages::chats::ASKING_TEMPLATE;
    me.projects = List::with_capacity(PROJECTS);
    me.projects.push(owner).expect("owner");
    me.projects.push(other).expect("other");
    me
}

fn project_domain() -> Domain {
    let mut root = crate::unlinked(&f::limits(), f::SEED);
    let watch = f::start(&mut root, List::with_capacity(0));
    up(
        &mut root,
        Up::FamilyServed {
            family: wire::FORGE,
            versions: Some(c::VersionRange { first: wire::CLIENT_VERSION, last: wire::CLIENT_VERSION }),
        },
    );
    let out = deliver(&mut root, watch, c::Delivery::Me(Box::new(two_projects())));
    for request in &out {
        if let Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: Down::Open { watch, subject: Watchable::Jig(c::Subject::Inbox { .. }) },
                },
        } = request
        {
            let mut rows = List::with_capacity(NEEDS);
            rows.push(f::inbox_entry(f::acts::QUESTION, c::InboxGroup::Questions, f::PROJECT, 0)).expect("one need");
            let inbox = crate::tests::documents::inbox(rows, NEEDS, c::Count { value: 0, limited: false }, 0, true);
            deliver(&mut root, *watch, c::Delivery::Inbox(Box::new(inbox)));
        }
        if let Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down { down: Down::Read { read, what: Readable::Family { record, .. } } },
        } = request
            && let w::Read::Setting(setting) =
                wire::decode_reading(&record.bytes, &w::CEILINGS).expect("reading").read()
            && (setting.kind() == 7 || setting.kind() == 8)
        {
            let ways = w::ProjectWays::new(
                &w::CEILINGS,
                w::ProjectWaysParts { skein_bools: [true, false, false, true], way: 0 },
            )
            .expect("ways");
            let mut writer = Writer::new(usize::try_from(ways.measure()).expect("ways size"));
            ways.encode(&mut writer).expect("ways bytes");
            let answer = w::SettingAnswer::new(
                &w::CEILINGS,
                w::SettingAnswerParts { value: if setting.kind() == 7 { Some(writer.finish()) } else { None } },
            )
            .expect("answer");
            up(
                &mut root,
                Up::ReadDone {
                    read: *read,
                    result: Readout::Family { family: wire::FORGE, record: Some(wire::setting_answer(&answer)) },
                },
            );
        }
    }
    event(&mut root, Event::Went { place: Place::Address(Address::Settings { project: f::PROJECT }) });
    root
}

#[test]
fn facts_gather_the_open_projects_domain_values() {
    let project_root = project_domain();
    let unread = crate::unlinked(&f::limits(), f::SEED);
    for (name, root, project, expected) in [
        (
            "project",
            &project_root,
            f::PROJECT,
            Facts { projects: PROJECTS, needs: NEEDS, needs_cut: true, asking: true, away: true, drafts: true, ..BASE },
        ),
        ("other project", &project_root, OTHER, Facts { projects: PROJECTS, needs_cut: true, ..BASE }),
        ("unread", &unread, f::PROJECT, BASE),
    ] {
        assert_eq!(rail::facts(root, project, f::NOW), expected, "{name}");
    }
}

#[test]
fn across_means_all_home_or_inbox_without_a_project_filter() {
    let mut root = project_domain();
    for (address, expected) in [
        (Address::AllHome, true),
        (Address::Inbox { filter: Filter::ALL }, true),
        (Address::Inbox { filter: Filter { project: Some(f::PROJECT), ..Filter::ALL } }, false),
        (Address::Home { project: f::PROJECT }, false),
    ] {
        event(&mut root, Event::Went { place: Place::Address(address.clone()) });
        assert_eq!(rail::across(&root), expected, "{address:?}");
    }
}

//! Owner's words for the kit's refusal locations (protocol/setup.md, 5).
use crate::problem::{Problems, What};
use alloc::boxed::Box;
use jig_protocol_config::{Path, Segment};
use skein_forge_setup::given::{Place, Refused};

/// Names a refused kit input using the setup entry's position.
#[must_use]
pub fn refusal(index: u32, why: Refused) -> Problems {
    let root = Path::root();
    let entry = root.member(b"application").member(b"setup").child(Segment::Index(index));
    let limits = root.member(b"application").member(b"limits").member(b"overrides");
    let mut what = What::Reason(b"invalid form");
    let path = match why.place {
        Place::FilesLimit
        | Place::FileBytesLimit
        | Place::CreateOrg
        | Place::CreateRepository
        | Place::Files
        | Place::FilePath(_)
        | Place::FileContent(_)
        | Place::Plan => unreachable!("Create fields and file limits are temper constants or admitted SetUp names"),
        Place::BotLogin => entry.member(b"bot").member(b"login"),
        Place::BotEmail => entry.member(b"bot").member(b"email"),
        Place::Orgs => {
            what = What::Reason(b"past its bound");
            entry.member(b"orgs")
        }
        Place::Org(j) => entry.member(b"orgs").child(Segment::Index(j)),
        Place::OrgOwner => entry.member(b"mode").member(b"owned").member(b"person").member(b"login"),
        Place::OrgsLimit => {
            what = What::Reason(b"outside its type");
            limits.member(b"setup.orgs")
        }
        Place::Deadline => {
            what = What::Reason(b"outside its type");
            limits.member(b"setup.deadline_ms")
        }
        Place::Backoff => {
            what = What::Reason(b"outside its type");
            limits.member(b"setup.backoff_ms")
        }
        Place::BackoffMax => {
            what = What::Reason(b"outside its type");
            limits.member(b"setup.backoff_max_ms")
        }
        Place::ReachableLimit | Place::Page => limits.member(b"picker.page"),
        Place::Repositories | Place::Repository(_) | Place::RepositoriesLimit => {
            what = What::Reason(b"outside its type");
            limits.member(b"forge.client.repositories")
        }
        Place::Word => {
            what = What::Own(b"temper's word");
            root
        }
        Place::AdminName => {
            what = What::Own(b"temper's admin name");
            root
        }
        Place::ApiName => {
            what = What::Own(b"temper's API token name");
            root
        }
        Place::GitName => {
            what = What::Own(b"temper's git token name");
            root
        }
    };
    if let Some(other) = why.other {
        what = What::Against(match other {
            Place::BackoffMax => b"setup.backoff_max_ms",
            Place::Deadline => b"setup.deadline_ms",
            Place::ReachableLimit | Place::Page => b"picker.page",
            Place::Repositories | Place::Repository(_) | Place::RepositoriesLimit => b"forge.client.repositories",
            Place::FilesLimit
            | Place::FileBytesLimit
            | Place::CreateOrg
            | Place::CreateRepository
            | Place::Files
            | Place::FilePath(_)
            | Place::FileContent(_)
            | Place::Plan => unreachable!("Create has no paired limits"),
            Place::Word
            | Place::BotLogin
            | Place::BotEmail
            | Place::AdminName
            | Place::ApiName
            | Place::GitName
            | Place::Orgs
            | Place::Org(_)
            | Place::OrgOwner
            | Place::OrgsLimit
            | Place::Backoff => unreachable!("kit limit pairs name maximum backoff or deadline"),
        });
    }
    let mut problems = Problems::empty();
    problems.push(path, what);
    problems
}

/// Copies temper's configured setup and owner constants into the kit's values.
#[must_use]
pub fn values(
    entry: &crate::setup::Entry,
    listener: Option<core::net::SocketAddr>,
    limits: &skein_forge_setup::limits::Limits,
) -> skein_forge_setup::given::Given {
    skein_forge_setup::given::Given {
        forge: entry.forge,
        bot_login: entry.bot.login.clone(),
        bot_email: entry.bot.email.clone(),
        orgs: Box::new([]),
        // The runtime supplies the restored adopted ids in its Reconcile plan.
        adopted: Box::new([]),
        org_owner: match &entry.mode {
            crate::setup::Mode::Managed { .. } | crate::setup::Mode::Fake(_) => None,
            crate::setup::Mode::Owned(owned) => Some(owned.person.login.clone()),
        },
        listener,
        word: skein_lib::bytes::copy_of(b"temper"),
        admin_name: skein_lib::bytes::copy_of(match entry.mode {
            crate::setup::Mode::Managed { admin_token: None } => b"admin",
            crate::setup::Mode::Managed { admin_token: Some(_) }
            | crate::setup::Mode::Owned(_)
            | crate::setup::Mode::Fake(_) => b"temper-admin",
        }),
        api_name: skein_lib::bytes::copy_of(b"temper-api"),
        git_name: skein_lib::bytes::copy_of(b"temper-git"),
        limits: limits.clone(),
    }
}

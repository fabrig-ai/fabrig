//! Members refused at start and their destinations (protocol/configuration/moved.md, 3).
use jig_protocol_config::Path;
use skein_lib::Writer;

#[derive(Debug)]
pub struct Member {
    pub path: &'static str,
    pub words: &'static str,
}
pub const MEMBERS: &[Member] = &[
    Member { path: "engine.deployment", words: "moved: temper keeps the store in the data directory's state" },
    Member {
        path: "engine.providers",
        words: "moved: temper fixes its identity providers: 1 link, 2 deployment, 3 Google's",
    },
    Member { path: "engine.connectors", words: "moved: temper fixes its connector, the forge's, 1" },
    Member { path: "engine.acts", words: "moved: temper fixes the forge connector's acts" },
    Member {
        path: "engine.projects",
        words: "moved: projects are made at runtime; make one from the client or temper ctl",
    },
    Member {
        path: "engine.clients.heartbeat_ms",
        words: "moved: temper fixes the clients' heartbeat, printed link and approval",
    },
    Member {
        path: "engine.clients.link",
        words: "moved: temper fixes the clients' heartbeat, printed link and approval",
    },
    Member {
        path: "engine.clients.approval",
        words: "moved: temper fixes the clients' heartbeat, printed link and approval",
    },
];

#[must_use]
#[expect(clippy::manual_find, reason = "step code uses no closures")]
pub fn at(path: &[u8]) -> Option<&'static Member> {
    for member in MEMBERS {
        if member.path.as_bytes() == path {
            return Some(member);
        }
    }
    None
}
/// A reference row at its containing shape's path.
#[must_use]
pub fn child(path: &str, name: &str) -> Option<&'static Member> {
    for member in MEMBERS {
        if let Some((parent, child)) = member.path.rsplit_once('.')
            && parent == path
            && child == name
        {
            return Some(member);
        }
    }
    None
}
#[must_use]
pub fn words(path: &Path) -> &'static [u8] {
    let mut length = 0;
    super::problems::render_path(path, &mut Writer::new(0), &mut length, true);
    let mut writer = Writer::new(length);
    super::problems::render_path(path, &mut writer, &mut 0, false);
    let rendered = writer.finish();
    at(&rendered).expect("every fixed member has its moved words").words.as_bytes()
}

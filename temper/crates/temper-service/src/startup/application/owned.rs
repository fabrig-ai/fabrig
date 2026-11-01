//! The machine's owned Forgejo (temper/docs/plan/playground-reset-r4/notes/contracts.md, 6).
use super::Problems;
use jig_protocol_config::{self as config, Path};
use skein_json::{Document, document::Kind};
use temper_forge_setup::{
    limits::Host,
    setup::{Template, Want},
};

pub(super) fn parse(
    document: &Document,
    at: Option<u32>,
    limits: &Host,
    directories: &config::Directories,
    problems: &mut Problems,
) -> Option<Template> {
    let at = at?;
    if document.token(at)?.kind == Kind::Null {
        return None;
    }
    let path = Path::root().member(b"application").member(b"owned");
    let mut found = config::Problems::empty();
    let template = Template::read(document, at, Want::Owned, &limits.plan(), directories, &path, &mut found);
    problems.append(found, &Path::root());
    template
}

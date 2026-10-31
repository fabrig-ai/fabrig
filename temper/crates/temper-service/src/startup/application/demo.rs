//! The scripted provider (temper/docs/design/draft/playground-reset.md, 6.1).
use super::{Problems, What, read, settings};
use alloc::boxed::Box;
use jig_protocol_config::{self as config, Path};
use skein_json::{Document, document::Kind};
use skein_lib::List;
use temper_client_protocol_wire::v1 as w;

fn text(
    document: &Document,
    object: &mut config::read::Object,
    name: &[u8],
    max: u32,
    path: &Path,
    problems: &mut Problems,
) -> Option<Box<[u8]>> {
    let at = read::required(object, document, name, path, problems)?;
    match config::read::text(document, at, max) {
        Ok(value) => Some(value),
        Err(what) => {
            problems.push(path.member(name), What::Jig(what));
            None
        }
    }
}
pub(super) fn parse(
    document: &Document,
    at: Option<u32>,
    directories: &config::Directories,
    problems: &mut Problems,
) -> Option<settings::Demo> {
    let at = at?;
    if document.token(at)?.kind == Kind::Null {
        return None;
    }
    let path = Path::root().member(b"application").member(b"demo");
    let mut object = read::object(document, at, &path, problems)?;
    let at = read::required(&mut object, document, b"llm", &path, problems)?;
    let llm_path = path.member(b"llm");
    let mut llm = read::object(document, at, &llm_path, problems)?;
    let provider = read::number(&mut llm, document, b"provider", &llm_path, problems)?;
    let key_file = text(document, &mut llm, b"key", 4096, &llm_path, problems)?;
    let key_file = super::joined(directories, config::Directory::Data, &key_file, &llm_path.member(b"key"), problems)?;
    // This public view reads asked alone; optional cued is checked by the fake.
    // These objects are not finished here, so cued needs no consuming read.
    let asked = read::list(&mut llm, document, b"asked", w::CEILINGS.demo_answer_cues, &llm_path, problems)?;
    let mut cues = List::with_capacity(asked.len());
    for (index, at) in asked.iter().enumerate() {
        let path = read::index(&llm_path.member(b"asked"), index);
        let mut asked = read::object(document, *at, &path, problems)?;
        let cue = text(document, &mut asked, b"cue", w::CEILINGS.demo_answer_cues_item, &path, problems)?;
        cues.push(cue).expect("bounded demo cues");
    }
    Some(settings::Demo { provider, key_file })
}

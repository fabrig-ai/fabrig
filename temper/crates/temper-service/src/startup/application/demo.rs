//! The scripted provider (temper/docs/design/protocol/setup/playground.md).
use super::{Problems, What, read, settings};
use alloc::boxed::Box;
use jig_protocol_config::{self as config, Path};
use skein_json::{Document, document::Kind};
use skein_lib::List;
use temper_client_protocol_wire::v1 as w;

fn text(
    document: &Document,
    object: &mut crate::startup::members::Object,
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
    let mut object = read::object(document, at, "demo", &path, problems)?;
    let at = read::required(&mut object, document, b"llm", &path, problems);
    read::finish(object, document, &path, problems);
    let at = at?;
    let llm_path = path.member(b"llm");
    let mut llm = read::object(document, at, "demo-llm", &llm_path, problems)?;
    let provider = read::number(&mut llm, document, b"provider", &llm_path, problems)?;
    let key_file = text(document, &mut llm, b"key", 4096, &llm_path, problems)?;
    let key_file = super::joined(directories, config::Directory::Data, &key_file, &llm_path.member(b"key"), problems)?;
    // The fake checks script values; startup takes their shape through the same table.
    let _listen = llm.member(document, b"listen");
    let unoffered = llm.member(document, b"cued");
    let otherwise = llm.member(document, b"otherwise");
    let asked = read::list(&mut llm, document, b"asked", w::CEILINGS.playground_answer_cues, &llm_path, problems)?;
    let mut cues = List::with_capacity(asked.len());
    for (index, at) in asked.iter().enumerate() {
        let path = read::index(&llm_path.member(b"asked"), index);
        let mut asked = read::object(document, *at, "demo-script", &path, problems)?;
        let cue = text(document, &mut asked, b"cue", w::CEILINGS.playground_answer_cues_item, &path, problems)?;
        let turns = asked.member(document, b"turns");
        take_turns(document, turns, &path.member(b"turns"), problems);
        read::finish(asked, document, &path, problems);
        cues.push(cue).expect("bounded demo cues");
    }
    take_scripts(document, unoffered, &llm_path.member(b"cued"), problems);
    take_turns(document, otherwise, &llm_path.member(b"otherwise"), problems);
    read::finish(llm, document, &llm_path, problems);
    Some(settings::Demo {
        provider,
        key_file,
        owner: b"playground".as_slice().into(),
        name: b"widget".as_slice().into(),
        landing: b"main".as_slice().into(),
        cues: cues.into_boxed(),
    })
}

fn take_scripts(document: &Document, at: Option<u32>, path: &Path, problems: &mut Problems) {
    let Some(at) = at else { return };
    let Ok(items) = config::read::list(document, at, document.len()) else { return };
    for (index, at) in items.iter().enumerate() {
        let path = read::index(path, index);
        let Some(mut object) = read::object(document, *at, "demo-script", &path, problems) else { continue };
        let _cue = object.member(document, b"cue");
        let turns = object.member(document, b"turns");
        take_turns(document, turns, &path.member(b"turns"), problems);
        read::finish(object, document, &path, problems);
    }
}

fn take_turns(document: &Document, at: Option<u32>, path: &Path, problems: &mut Problems) {
    let Some(at) = at else { return };
    let Ok(items) = config::read::list(document, at, document.len()) else { return };
    for (index, at) in items.iter().enumerate() {
        if document.token(*at).expect("turn").kind != Kind::ObjectStart {
            continue;
        }
        let path = read::index(path, index);
        let Some(mut object) = read::object(document, *at, "demo-turn", &path, problems) else { continue };
        let lines = object.member(document, b"lines");
        let _finish = object.member(document, b"finish");
        let _tokens = object.member(document, b"tokens");
        read::finish(object, document, &path, problems);
        let Some(at) = lines else { continue };
        let Ok(lines) = config::read::list(document, at, document.len()) else { continue };
        for (index, at) in lines.iter().enumerate() {
            let path = read::index(&path.member(b"lines"), index);
            let Some(mut object) = read::object(document, *at, "demo-line", &path, problems) else { continue };
            let _text = object.member(document, b"text");
            let _call = object.member(document, b"call");
            let _arguments = object.member(document, b"arguments");
            read::finish(object, document, &path, problems);
        }
    }
}

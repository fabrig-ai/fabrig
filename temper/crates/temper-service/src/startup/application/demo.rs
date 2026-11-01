//! The demo's fake LLM, forge and project (protocol/setup.md, 8.4).
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
    limits: &skein_forge_setup::limits::Limits,
    machine: &mut temper_engine_domain_forge::Machine,
    directories: &config::Directories,
    problems: &mut Problems,
) -> Option<settings::Demo> {
    let at = at?;
    if document.token(at)?.kind == Kind::Null {
        return None;
    }
    let path = Path::root().member(b"application").member(b"demo");
    let mut object = read::object(document, at, &path, problems)?;
    let forge_at = read::required(&mut object, document, b"forge", &path, problems)?;
    if document.token(forge_at)?.kind != Kind::ObjectStart {
        problems.push(path.member(b"forge"), What::Jig(config::What::Form));
        return None;
    }
    t4(document, at, forge_at, limits, machine, directories, problems)
}

struct Llm {
    provider: u32,
    key_file: Box<[u8]>,
    cues: Box<[Box<[u8]>]>,
}
fn llm(
    document: &Document,
    object: &mut config::read::Object,
    path: &Path,
    directories: &config::Directories,
    problems: &mut Problems,
) -> Option<Llm> {
    let at = read::required(object, document, b"llm", path, problems)?;
    let llm_path = path.member(b"llm");
    let mut llm = read::object(document, at, &llm_path, problems)?;
    let provider = read::number(&mut llm, document, b"provider", &llm_path, problems)?;
    let key_file = text(document, &mut llm, b"key", 4096, &llm_path, problems)?;
    let key_file = super::joined(directories, config::Directory::Data, &key_file, &llm_path.member(b"key"), problems)?;
    // This public view reads asked alone; optional cued is checked by the fake.
    // These objects are not finished here, so cued needs no consuming read.
    let asked = read::list(&mut llm, document, b"asked", w::CEILINGS.playground_answer_cues, &llm_path, problems)?;
    let mut cues = List::with_capacity(asked.len());
    for (index, at) in asked.iter().enumerate() {
        let path = read::index(&llm_path.member(b"asked"), index);
        let mut asked = read::object(document, *at, &path, problems)?;
        let cue = text(document, &mut asked, b"cue", w::CEILINGS.playground_answer_cues_item, &path, problems)?;
        cues.push(cue).expect("bounded demo cues");
    }
    Some(Llm { provider, key_file, cues: cues.into_boxed() })
}

#[expect(
    clippy::disallowed_methods,
    reason = "the configuration protocol converts validated JSON text to a project name"
)]
fn t4(
    document: &Document,
    at: u32,
    forge_at: u32,
    limits: &skein_forge_setup::limits::Limits,
    machine: &mut temper_engine_domain_forge::Machine,
    directories: &config::Directories,
    problems: &mut Problems,
) -> Option<settings::Demo> {
    let path = Path::root().member(b"application").member(b"demo");
    let mut object = read::object(document, at, &path, problems)?;
    object.member(document, b"forge");
    let forge_path = path.member(b"forge");
    let (entry, body) = super::machine::body(document, forge_at, &forge_path, limits, directories, problems)?;
    let temper_forge_setup::setup::Mode::Fake(fake) = &entry.mode else {
        problems.push(forge_path.member(b"mode"), What::Jig(config::What::Form));
        return None;
    };
    let Some(repository) = fake.repositories.first() else {
        problems
            .push(forge_path.member(b"mode").member(b"fake").member(b"repositories"), What::Jig(config::What::Form));
        return None;
    };
    let project_at = read::required(&mut object, document, b"project", &path, problems)?;
    let project_path = path.member(b"project");
    if document.token(project_at)?.kind != Kind::ObjectStart {
        problems.push(project_path, What::Jig(config::What::Form));
        return None;
    }
    let mut project = read::object(document, project_at, &project_path, problems)?;
    let name = text(document, &mut project, b"name", 64, &project_path, problems)?;
    read::form(!name.is_empty(), project_path.member(b"name"), problems);
    let policy = read::number(&mut project, document, b"policy", &project_path, problems)?;
    read::form(policy > 0, project_path.member(b"policy"), problems);
    read::finish(project, document, &project_path, problems);
    let Llm { provider, key_file, cues } = llm(document, &mut object, &path, directories, problems)?;
    read::finish(object, document, &path, problems);
    let demo = settings::Demo {
        project: 0,
        forge: 0,
        project_name: core::str::from_utf8(&name).expect("JSON text is UTF-8").into(),
        project_policy: policy,
        provider,
        owner: fake.owner.clone(),
        name: repository.name.clone(),
        landing: repository.landing.clone(),
        cues,
        key_file,
        state: None,
        playground: None,
    };
    machine.playground = Some(super::machine::playground(&entry, body));
    Some(demo)
}

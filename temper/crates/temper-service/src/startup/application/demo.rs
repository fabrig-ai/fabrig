//! The demo's fake LLM, forge and project (protocol/setup.md, 8.4).
use super::{Forge, Problems, What, read, settings};
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
#[expect(clippy::too_many_arguments, reason = "both demo shapes share the document and startup context")]
pub(super) fn parse(
    document: &Document,
    at: Option<u32>,
    setup: Option<u32>,
    forges: &List<Forge>,
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
    if document.token(forge_at)?.kind == Kind::ObjectStart {
        return t4(document, at, forge_at, limits, machine, directories, problems);
    }
    let project = read::number(&mut object, document, b"project", &path, problems)?;
    let forge = read::short(&mut object, document, b"forge", &path, problems)?;
    let Llm { provider, key_file, cues } = llm(document, &mut object, &path, directories, problems)?;
    let mut configured = false;
    for value in forges {
        configured |= value.number == forge;
    }
    if !configured {
        problems.push(path.member(b"forge"), What::Jig(config::What::Refers));
        return None;
    }
    let setup_path = Path::root().member(b"application").member(b"setup");
    let Some(setup) = setup else {
        problems.push(path.member(b"forge"), What::Jig(config::What::Refers));
        return None;
    };
    let entries = match config::read::list(document, setup, document.len()) {
        Ok(entries) => entries,
        Err(what) => {
            problems.push(setup_path, What::Jig(what));
            return None;
        }
    };
    for (index, at) in entries.iter().enumerate() {
        let setup_path = read::index(&setup_path, index);
        let mut entry = read::object(document, *at, &setup_path, problems)?;
        if read::short(&mut entry, document, b"forge", &setup_path, problems)? != forge {
            continue;
        }
        let at = read::required(&mut entry, document, b"mode", &setup_path, problems)?;
        let mode_path = setup_path.member(b"mode");
        let mut mode = read::object(document, at, &mode_path, problems)?;
        let Some(at) = mode.member(document, b"fake") else {
            problems.push(path.member(b"forge"), What::Jig(config::What::Form));
            return None;
        };
        let fake_path = mode_path.member(b"fake");
        let mut fake = read::object(document, at, &fake_path, problems)?;
        let owner = text(document, &mut fake, b"owner", w::CEILINGS.playground_answer_owner, &fake_path, problems)?;
        let repositories = read::list(&mut fake, document, b"repositories", document.len(), &fake_path, problems)?;
        let Some(at) = repositories.get(0) else {
            problems.push(fake_path.member(b"repositories"), What::Jig(config::What::Form));
            return None;
        };
        let repository_path = read::index(&fake_path.member(b"repositories"), 0);
        let mut repository = read::object(document, *at, &repository_path, problems)?;
        let name =
            text(document, &mut repository, b"name", w::CEILINGS.playground_answer_name, &repository_path, problems)?;
        let landing =
            text(document, &mut repository, b"landing", w::CEILINGS.adopt_landing, &repository_path, problems)?;
        return Some(settings::Demo {
            project,
            project_name: "".into(),
            project_policy: 0,
            forge,
            provider,
            owner,
            name,
            cues,
            landing,
            key_file,
            state: None,
        });
    }
    problems.push(path.member(b"forge"), What::Jig(config::What::Refers));
    None
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
    };
    machine.playground = Some(super::machine::playground(&entry, body));
    Some(demo)
}

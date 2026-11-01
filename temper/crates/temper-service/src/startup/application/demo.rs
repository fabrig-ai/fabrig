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
#[expect(clippy::too_many_lines, reason = "the demo reads its optional forge reference and fake template together")]
pub(super) fn parse(
    document: &Document,
    at: Option<u32>,
    setup: Option<u32>,
    forges: &List<Forge>,
    directories: &config::Directories,
    problems: &mut Problems,
) -> Option<settings::Demo> {
    let at = at?;
    if document.token(at)?.kind == Kind::Null {
        return None;
    }
    let path = Path::root().member(b"application").member(b"demo");
    let mut object = read::object(document, at, &path, problems)?;
    let project = read::number(&mut object, document, b"project", &path, problems)?;
    let forge = match object.member(document, b"forge") {
        Some(at) => match config::read::integer(document, at, u64::from(u16::MAX)) {
            Ok(value) => Some(u16::try_from(value).expect("u16 bound")),
            Err(what) => {
                problems.push(path.member(b"forge"), What::Jig(what));
                return None;
            }
        },
        None => None,
    };
    let fake_at = object.member(document, b"fake");
    let mut found = config::Problems::empty();
    let fake_template = match fake_at {
        Some(at) if document.token(at)?.kind != Kind::Null => temper_forge_setup::setup::Template::read(
            document,
            at,
            temper_forge_setup::setup::Want::Fake,
            &temper_forge_setup::limits::small(),
            directories,
            &path.member(b"fake"),
            &mut found,
        ),
        Some(_) | None => None,
    };
    problems.append(found, &Path::root());
    let at = read::required(&mut object, document, b"llm", &path, problems)?;
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
    let Some(number) = forge else {
        let Some(template) = &fake_template else {
            problems.push(path.member(b"fake"), What::Jig(config::What::Missing));
            return None;
        };
        let temper_forge_setup::setup::Mode::Fake(fake) = &template.mode else { return None };
        let repository = fake.repositories.first()?;
        return Some(settings::Demo {
            project,
            forge,
            provider,
            owner: fake.owner.clone(),
            name: repository.name.clone(),
            cues: cues.into_boxed(),
            landing: repository.landing.clone(),
            key_file,
            fake: fake_template,
            playground: None,
        });
    };
    let mut configured = false;
    for value in forges {
        configured |= value.number == number;
    }
    if !configured {
        problems.push(path.member(b"forge"), What::Jig(config::What::Refers));
        return None;
    }
    let setup_path = Path::root().member(b"application").member(b"setup");
    let entries = match config::read::list(document, setup?, document.len()) {
        Ok(entries) => entries,
        Err(what) => {
            problems.push(setup_path, What::Jig(what));
            return None;
        }
    };
    for (index, at) in entries.iter().enumerate() {
        let setup_path = read::index(&setup_path, index);
        let mut entry = read::object(document, *at, &setup_path, problems)?;
        if read::short(&mut entry, document, b"forge", &setup_path, problems)? != number {
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
            forge,
            fake: fake_template,
            provider,
            owner,
            name,
            cues: cues.into_boxed(),
            landing,
            key_file,
            playground: None,
        });
    }
    problems.push(path.member(b"forge"), What::Jig(config::What::Refers));
    None
}

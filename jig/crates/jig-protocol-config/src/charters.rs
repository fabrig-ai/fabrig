//! Offered run policies and their reserve admission (protocol/configuration.md, 3 and protocol/limits.md).
use crate::{Directories, Directory, Limits, Llm, Path, Problems, Segment, What, check, read};
use alloc::boxed::Box;
use jig_domain_core::{Charter, CharterPart, Model, RunPolicy};
use jig_domain_fleet::Kinds;
use jig_domain_tasks::{Reasons, ResultsWake, WakePolicy, WakeRule};
use skein_json::Document;
use skein_lib::{Duration, List, length::len_u64};
pub(crate) fn parse(
    document: &Document,
    at: u32,
    path: &Path,
    llm: &Llm,
    limits: &Limits,
    directories: &Directories,
    problems: &mut Problems,
) -> Option<Box<[Charter]>> {
    let items = match read::list(document, at, limits.core.tasks.charters) {
        Ok(items) => items,
        Err(what) => {
            problems.push(path.clone(), what);
            return None;
        }
    };
    let mut charters = List::<Charter>::with_capacity(limits.core.tasks.charters);
    for (index, at) in items.iter().enumerate() {
        let path = path.child(Segment::Index(u32::try_from(index).expect("bounded charter count")));
        if let Some(charter) = charter(document, *at, &path, llm, limits, directories, problems) {
            for previous in &charters {
                if previous.number == charter.number {
                    problems.push(path.member(b"number"), What::Unique);
                    break;
                }
            }
            if charter.number == 0 {
                problems.push(path.member(b"number"), What::Bound);
            }
            match owned_bytes(&charter) {
                Some(bytes) if bytes <= u64::from(limits.core.run_bytes) => {}
                Some(_) | None => problems.push(path.clone(), What::Bound),
            }
            charters.push(charter).expect("input charter count bounded");
        }
    }
    Some(charters.into_boxed())
}
pub(crate) fn files(engine: &crate::Engine, files: &mut List<crate::File>) {
    if engine.defaults == crate::Defaults::Passed {
        return;
    }
    for (index, charter) in engine.charters.iter().enumerate() {
        let owned = owned_bytes(charter).expect("validated charter policy and parts size");
        let room =
            u64::from(engine.limits.core.run_bytes).checked_sub(owned).expect("validated charter fits run bound");
        let at = check::index(&Path::root().member(b"charters"), index);
        instruction_file(files, at.member(b"instructions"), &charter.policy.instructions, room);
        for (index, part) in charter.parts.iter().enumerate() {
            let at = check::index(&at.member(b"parts"), index).member(b"instructions");
            instruction_file(files, at, &part.instructions, room);
        }
    }
}

fn owned_bytes(charter: &Charter) -> Option<u64> {
    jig_domain_core::policy_owned_bytes(&charter.policy)?
        .checked_add(jig_domain_core::parts_owned_bytes(&charter.parts)?)
}

fn instruction_file(files: &mut List<crate::File>, at: Path, path: &[u8], room: u64) {
    let max = room.checked_add(len_u64(path.len())).expect("room plus this file's path fits run bound");
    crate::files::push(files, at, path, u32::try_from(max).expect("run bytes bound is u32"), false);
}

fn charter(
    document: &Document,
    at: u32,
    path: &Path,
    llm: &Llm,
    limits: &Limits,
    directories: &Directories,
    problems: &mut Problems,
) -> Option<Charter> {
    let mut object = check::object(document, at, path, problems, "charter")?;
    let models_at = object.member(document, b"models");
    let rest = match object.member(document, b"rest") {
        Some(at) => match read::boolean(document, at) {
            Ok(value) => value,
            Err(what) => {
                problems.push(path.member(b"rest"), what);
                false
            }
        },
        None => models_at.is_none(),
    };
    let number = check::count(&mut object, document, b"number", path, problems);
    let instructions =
        check::path(&mut object, document, b"instructions", path, Directory::Config, directories, problems);
    let waiting = check::duration(&mut object, document, b"waiting_ms", path, problems);
    let resume = check::boolean(&mut object, document, b"resume", path, problems);
    let turns = check::count(&mut object, document, b"turns", path, problems);
    let time = check::duration(&mut object, document, b"time_ms", path, problems);
    let reserve_turns = check::count(&mut object, document, b"reserve_turns", path, problems);
    let reserve_time = check::duration(&mut object, document, b"reserve_time_ms", path, problems);
    let spend = check::integer(&mut object, document, b"spend", path, problems);
    let (model, alternatives) = match models_at {
        Some(at) => {
            let _model_at = object.member(document, b"model");
            let _alternatives_at = object.member(document, b"alternatives");
            named_models(document, at, path, llm, problems)
        }
        None => {
            let model = match object.member(document, b"model") {
                Some(at) => match read::text(document, at, jig_domain_core::MODEL_NAME_BYTES) {
                    Ok(name) => model(&name, llm, &path.member(b"model"), problems),
                    Err(what) => {
                        problems.push(path.member(b"model"), what);
                        None
                    }
                },
                None => match llm.models.first() {
                    Some(model) => Some(model.model.clone()),
                    None => {
                        problems.push(path.member(b"models"), What::Refers);
                        None
                    }
                },
            };
            (model, alternatives(&mut object, document, path, llm, problems))
        }
    };
    let hosts = hosts(&mut object, document, path, problems);
    let inspect = check::boolean(&mut object, document, b"inspect", path, problems);
    let modify = check::boolean(&mut object, document, b"modify", path, problems);
    let shell = check::boolean(&mut object, document, b"shell", path, problems);
    let agents = check::boolean(&mut object, document, b"agents", path, problems);
    let call_timeout = check::duration(&mut object, document, b"call_timeout_ms", path, problems);
    let wake = match object.member(document, b"wake") {
        Some(at) => wake(document, at, &path.member(b"wake"), problems),
        None => Some(WakePolicy::DEFAULT),
    };
    let documents = documents(&mut object, document, path, problems);
    let parts = parts(&mut object, document, path, limits.core.tasks.charter_parts, directories, problems);
    object.finish(document, path, problems);
    let charter = Charter {
        parts: parts?,
        rest,
        documents: documents?,
        number: number?,
        hosts: hosts?,
        spend: spend?,
        wake: wake?,
        policy: RunPolicy {
            instructions: instructions?,
            waiting: waiting?,
            resume: resume?,
            turns: turns?,
            time: time?,
            reserve_turns: reserve_turns?,
            reserve_time: reserve_time?,
            model: model?,
            alternatives: alternatives?,
            inspect: inspect?,
            modify: modify?,
            shell: shell?,
            agents: agents?,
            call_timeout: call_timeout?,
            // Its format carries no mask yet: every family (jig/docs/design/domain/engine/executors.md, 1).
            tools: RunPolicy::EVERY_FAMILY,
        },
    };
    validate(&charter, path, problems);
    Some(charter)
}

fn parts(
    object: &mut read::Object,
    document: &Document,
    path: &Path,
    max: u32,
    directories: &Directories,
    problems: &mut Problems,
) -> Option<Box<[CharterPart]>> {
    let Some(at) = object.member(document, b"parts") else { return Some(Box::new([])) };
    let path = path.member(b"parts");
    let items = crate::domain::items(document, at, document.len(), &path, problems)?;
    let mut parts = List::with_capacity(items.len().min(max));
    for (index, at) in items.iter().enumerate() {
        let path = check::index(&path, index);
        if u32::try_from(index).expect("bounded parts") >= max {
            problems.push(path, What::Bound);
            break;
        }
        let Some(mut object) = check::object(document, *at, &path, problems, "charter-part") else { continue };
        let on = reasons(&mut object, document, &path, problems);
        let instructions =
            check::path(&mut object, document, b"instructions", &path, Directory::Config, directories, problems);
        object.finish(document, &path, problems);
        if let Some(on) = on
            && let Some(instructions) = instructions
        {
            parts.push(CharterPart { on, instructions }).expect("bounded parts");
        }
    }
    Some(parts.into_boxed())
}

fn reasons(object: &mut read::Object, document: &Document, path: &Path, problems: &mut Problems) -> Option<Reasons> {
    let items = check::list(object, document, b"on", document.len(), path, problems)?;
    let path = path.member(b"on");
    if items.is_empty() {
        problems.push(path, What::Bound);
        return None;
    }
    let mut reasons = Reasons::EMPTY;
    for (index, at) in items.iter().enumerate() {
        let path = check::index(&path, index);
        match read::text(document, *at, 4_096) {
            Ok(name) => match Reasons::named(&name) {
                Some(reason) if reasons.meets(reason) => problems.push(path, What::Unique),
                Some(reason) => reasons = reasons.union(reason),
                None => problems.push(path, What::Unknown),
            },
            Err(what) => problems.push(path, what),
        }
    }
    Some(reasons)
}

fn documents(
    object: &mut read::Object,
    document: &Document,
    path: &Path,
    problems: &mut Problems,
) -> Option<Box<[jig_domain_core::Document]>> {
    let Some(at) = object.member(document, b"documents") else { return Some(Box::new([])) };
    let path = path.member(b"documents");
    let items = crate::domain::items(document, at, document.len(), &path, problems)?;
    let mut documents = List::with_capacity(items.len());
    for (index, at) in items.iter().enumerate() {
        let path = check::index(&path, index);
        let Some(mut object) = check::object(document, *at, &path, problems, "charter-document") else { continue };
        let connector = check::short(&mut object, document, b"connector", &path, problems);
        let name = document_name(&mut object, document, &path, problems);
        let file = check::text(&mut object, document, b"path", &path, problems);
        let mut heading = None;
        if let Some(at) = object.member(document, b"heading") {
            match read::text(document, at, u32::MAX) {
                Ok(value) => heading = Some(value),
                Err(what) => problems.push(path.member(b"heading"), what),
            }
        }
        object.finish(document, &path, problems);
        if let Some(connector) = connector
            && let Some(name) = name
            && let Some(path) = file
        {
            documents
                .push(jig_domain_core::Document { connector, name, path, heading })
                .expect("bounded document count");
        }
    }
    Some(documents.into_boxed())
}

fn document_name(
    object: &mut read::Object,
    document: &Document,
    path: &Path,
    problems: &mut Problems,
) -> Option<jig_domain_authority::Name> {
    let items = check::list(object, document, b"name", document.len(), path, problems)?;
    let mut segments = List::with_capacity(items.len());
    for (index, at) in items.iter().enumerate() {
        match read::text(document, *at, u32::MAX) {
            Ok(value) => segments.push(value).expect("bounded name segments"),
            Err(what) => problems.push(check::index(&path.member(b"name"), index), what),
        }
    }
    Some(jig_domain_authority::Name { segments: segments.into_boxed() })
}

fn alternatives(
    object: &mut read::Object,
    document: &Document,
    path: &Path,
    llm: &Llm,
    problems: &mut Problems,
) -> Option<Box<[Model]>> {
    let Some(at) = object.member(document, b"alternatives") else {
        return Some(Box::default());
    };
    let items = match read::list(document, at, document.len()) {
        Ok(items) => items,
        Err(what) => {
            problems.push(path.member(b"alternatives"), what);
            return None;
        }
    };
    let mut models = List::with_capacity(items.len());
    for (index, at) in items.iter().enumerate() {
        let path = path.member(b"alternatives").child(Segment::Index(u32::try_from(index).expect("bounded models")));
        match read::text(document, *at, 4_096) {
            Ok(name) => {
                if let Some(model) = model(&name, llm, &path, problems) {
                    for previous in &models {
                        if previous == &model {
                            problems.push(path.clone(), What::Unique);
                            break;
                        }
                    }
                    models.push(model).expect("bounded alternative count");
                }
            }
            Err(what) => problems.push(path, what),
        }
    }
    Some(models.into_boxed())
}
fn model(name: &[u8], llm: &Llm, path: &Path, problems: &mut Problems) -> Option<Model> {
    for model in &llm.models {
        if model.model.name.as_ref() == name {
            return Some(model.model.clone());
        }
    }
    problems.push(path.clone(), What::Refers);
    None
}
fn hosts(object: &mut read::Object, document: &Document, path: &Path, problems: &mut Problems) -> Option<Kinds> {
    let name = check::text(object, document, b"hosts", path, problems)?;
    match name.as_ref() {
        b"workers" => Some(Kinds::Workers),
        b"engine" => Some(Kinds::Engine),
        b"both" => Some(Kinds::Both),
        _ => {
            problems.push(path.member(b"hosts"), What::Form);
            None
        }
    }
}
fn validate(charter: &Charter, path: &Path, problems: &mut Problems) {
    let policy = &charter.policy;
    if policy.turns <= policy.reserve_turns || policy.time <= policy.reserve_time {
        problems.push(path.clone(), What::Reserve { charter: charter.number });
    }
    let enough = match jig_domain_core::reserve_floor(policy) {
        Some(floor) => charter.spend >= floor,
        None => false,
    };
    if !enough {
        problems.push(path.clone(), What::Budget { charter: charter.number });
    }
}

fn wake(document: &Document, at: u32, path: &Path, problems: &mut Problems) -> Option<WakePolicy> {
    let mut object = check::object(document, at, path, problems, "wake")?;
    let words = wake_member(&mut object, document, b"words", path, problems);
    let notices = wake_member(&mut object, document, b"notices", path, problems);
    let news = wake_member(&mut object, document, b"news", path, problems);
    let results = check::text(&mut object, document, b"results", path, problems);
    let results = match results.as_deref() {
        Some(b"never") => Some(ResultsWake::Never),
        Some(b"each") => Some(ResultsWake::Each),
        Some(b"last_or_failure") => Some(ResultsWake::LastOrFailure),
        Some(_) => {
            problems.push(path.member(b"results"), What::Form);
            None
        }
        None => None,
    };
    let questions = check::boolean(&mut object, document, b"questions", path, problems);
    let answers = check::boolean(&mut object, document, b"answers", path, problems);
    let timers = check::boolean(&mut object, document, b"timers", path, problems);
    object.finish(document, path, problems);
    Some(WakePolicy {
        words: words?,
        notices: notices?,
        news: news?,
        results: results?,
        questions: questions?,
        answers: answers?,
        timers: timers?,
    })
}

fn wake_member(
    object: &mut read::Object,
    document: &Document,
    name: &[u8],
    path: &Path,
    problems: &mut Problems,
) -> Option<WakeRule> {
    let at = object.required(document, name, path, problems)?;
    let path = path.member(name);
    match wake_rule(document, at, &path, problems) {
        Ok(rule) => Some(rule),
        Err(what) => {
            problems.push(path, what);
            None
        }
    }
}

fn wake_rule(document: &Document, at: u32, path: &Path, problems: &mut Problems) -> Result<WakeRule, What> {
    if document.token(at).ok_or(What::Kind)?.kind == skein_json::document::Kind::String {
        return match read::text(document, at, 32)?.as_ref() {
            b"never" => Ok(WakeRule::Never),
            b"immediate" => Ok(WakeRule::Immediate),
            _ => Err(What::Form),
        };
    }
    let mut object = check::object(document, at, path, problems, "wake-rule").ok_or(What::Kind)?;
    let batch = object.member(document, b"batch");
    let paced = object.member(document, b"paced");
    let rule = match (batch, paced) {
        (Some(at), None) => {
            let path = path.member(b"batch");
            let mut fields = check::object(document, at, &path, problems, "wake-batch").ok_or(What::Kind)?;
            let count = check::count(&mut fields, document, b"count", &path, problems);
            let age = check::duration(&mut fields, document, b"age_ms", &path, problems);
            fields.finish(document, &path, problems);
            match (count, age) {
                (Some(count), Some(age)) if count > 0 && age > Duration::ZERO => Ok(WakeRule::Batch { count, age }),
                _ => Err(What::Bound),
            }
        }
        (None, Some(at)) => {
            let path = path.member(b"paced");
            let mut fields = check::object(document, at, &path, problems, "wake-paced").ok_or(What::Kind)?;
            let span = check::duration(&mut fields, document, b"span_ms", &path, problems);
            fields.finish(document, &path, problems);
            match span {
                Some(span) if span > Duration::ZERO => Ok(WakeRule::Paced { span }),
                _ => Err(What::Bound),
            }
        }
        _ => Err(What::Form),
    };
    object.finish(document, path, problems);
    rule
}

fn named_models(
    document: &Document,
    at: u32,
    path: &Path,
    llm: &Llm,
    problems: &mut Problems,
) -> (Option<Model>, Option<Box<[Model]>>) {
    let items = match read::list(document, at, jig_domain_tasks::MODELS) {
        Ok(items) => items,
        Err(what) => {
            problems.push(path.member(b"models"), what);
            return (None, None);
        }
    };
    let mut main = None;
    let mut alternatives = List::with_capacity(items.len());
    for (index, at) in items.iter().enumerate() {
        let path = check::index(&path.member(b"models"), index);
        let name = match read::text(document, *at, jig_domain_core::MODEL_NAME_BYTES) {
            Ok(name) => name,
            Err(what) => {
                problems.push(path, what);
                continue;
            }
        };
        if let Some(value) = model(&name, llm, &path, problems) {
            if main.is_none() {
                main = Some(value);
            } else {
                alternatives.push(value).expect("bounded model count");
            }
        }
    }
    if main.is_none() {
        problems.push(path.member(b"models"), What::Refers);
    }
    (main, Some(alternatives.into_boxed()))
}

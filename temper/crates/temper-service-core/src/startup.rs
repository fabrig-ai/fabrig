//! The document's shape (protocol/services.md, 3).
use jig_protocol_config::{self as config, read};

/// The services selected by the document's members (protocol/cluster.md, 1).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Shape {
    Standalone,
    EngineAlone,
    WorkerAlone,
}
#[must_use]
pub fn member(document: &skein_json::Document, at: u32, name: &[u8]) -> Option<u32> {
    read::Object::new(document, at).ok()?.member(document, name)
}
/// The nearest shape of a parsed document (protocol/cluster.md, 1).
#[must_use]
pub fn nearest_document(document: &skein_json::Document) -> Shape {
    let Some(engine) = member(document, 0, b"engine") else { return Shape::WorkerAlone };
    let Some(hosts) = member(document, engine, b"hosts") else { return Shape::Standalone };
    let Some(at) = member(document, hosts, b"listener") else { return Shape::Standalone };
    let Ok(listeners) = read::list(document, at, document.len()) else { return Shape::Standalone };
    let Some(first) = listeners.get(0) else { return Shape::Standalone };
    if member(document, *first, b"socket").is_some() { Shape::EngineAlone } else { Shape::Standalone }
}
/// The document's nearest shape (protocol/cluster.md, 1), read from its
/// members alone; `None` when the bytes are no JSON document whose root is an object.
#[must_use]
pub fn nearest(input: &[u8]) -> Option<Shape> {
    let length = u32::try_from(input.len()).ok()?;
    let document =
        config::document(input, &config::DocumentLimits { bytes: 1_048_576, tokens: length, depth: 32 }).ok()?;
    read::Object::new(&document, 0).ok()?;
    Some(nearest_document(&document))
}

pub mod problems;
/// Paths shared by the halves' starts (protocol/services.md, 3).
#[must_use]
pub fn at(index: usize, path: config::Path) -> config::Path {
    path.child(config::Segment::Index(u32::try_from(index).expect("bounded configuration")))
}
#[must_use]
pub fn indexed(index: usize) -> config::Path {
    at(index, config::Path::root().member(b"workers"))
}
#[must_use]
pub fn own(name: &[u8]) -> config::Path {
    config::Path::root().member(b"application").member(b"limits").member(b"overrides").member(name)
}
#[must_use]
pub fn worker_limit(name: &[u8]) -> config::Path {
    indexed(0).member(b"limits").member(b"overrides").member(name)
}
#[must_use]
pub fn call_input_path(path: config::Path) -> config::Path {
    path.member(b"limits").member(b"overrides").member(b"core.call_answer_bytes")
}
pub fn short<E>(
    problems: &mut problems::Problems<problems::What<E>>,
    path: config::Path,
    bytes: u64,
    of: &'static [u8],
    needs: Option<u64>,
) {
    match needs {
        Some(needs) if bytes < needs => problems.push(path, problems::What::Short { bytes, of, needs }),
        Some(_) => {}
        None => problems.push(path, problems::What::Jig(config::What::Bound)),
    }
}
pub fn against<E>(problems: &mut problems::Problems<problems::What<E>>, name: &[u8], other: &'static [u8]) {
    problems.push(own(name), problems::What::Against { other });
}
pub fn call_answer<E>(path: config::Path, bytes: u32, problems: &mut problems::Problems<problems::What<E>>) {
    short(
        problems,
        call_input_path(path),
        u64::from(bytes),
        b"temper's documentation answer (read_docs)",
        Some(u64::from(crate::DOC_ANSWER)),
    );
}

/// Raw membership preserves an explicit profile-default override (protocol/configuration/limits.md).
#[derive(Clone, Copy, Debug)]
pub enum CallAnswer {
    Profile,
    Explicit,
}
impl CallAnswer {
    #[must_use]
    pub fn read(document: &skein_json::Document, at: u32) -> Self {
        let Some(limits) = member(document, at, b"limits") else { return Self::Profile };
        let Some(overrides) = member(document, limits, b"overrides") else { return Self::Profile };
        if member(document, overrides, b"core.call_answer_bytes").is_some() { Self::Explicit } else { Self::Profile }
    }
    pub fn compose(
        self,
        limits: &mut config::Limits,
        tool_payload: u32,
        host_ask_input: u32,
        host_reply_text: u32,
    ) -> Result<(), problems::SharedWhat> {
        match self {
            Self::Profile => limits.core.call_answer_bytes = limits.core.call_answer_bytes.max(tool_payload),
            Self::Explicit => {
                if limits.core.call_answer_bytes < tool_payload {
                    return Err(problems::What::CallInput {
                        bytes: u64::from(limits.core.call_answer_bytes),
                        input: u64::from(tool_payload),
                        ask: u64::from(host_ask_input),
                        reply: u64::from(host_reply_text),
                    });
                }
            }
        }
        Ok(())
    }
}

/// The shared limits grammar (protocol/services.md, 3).
pub fn configured_limits<E>(
    document: &skein_json::Document,
    at: u32,
    path: &config::Path,
    problems: &mut problems::Problems<problems::What<E>>,
) -> Option<crate::limits::Shared> {
    use crate::limits::{self, Profile, Refused};
    use problems::What;
    let mut object = match read::Object::new(document, at) {
        Ok(object) => object,
        Err(what) => {
            problems.push(path.clone(), What::Jig(what));
            return None;
        }
    };
    let profile = match object.member(document, b"profile") {
        Some(at) => match read::text(document, at, 1_048_576) {
            Ok(value) => Some(value),
            Err(what) => {
                problems.push(path.member(b"profile"), What::Jig(what));
                None
            }
        },
        None => {
            problems.push(path.member(b"profile"), What::Jig(config::What::Missing));
            None
        }
    };
    let overrides = object.member(document, b"overrides");
    if overrides.is_none() {
        problems.push(path.member(b"overrides"), What::Jig(config::What::Missing));
    }
    let mut found = config::Problems::empty();
    object.finish(document, path, &mut found);
    problems.append(found, &config::Path::root());
    let profile = match profile.as_deref() {
        Some(b"standard") => Profile::Standard,
        Some(b"small") | None => Profile::Small,
        Some(_) => {
            problems.push(path.member(b"profile"), What::Profile);
            Profile::Small
        }
    };
    let mut limits = limits::profile(profile);
    let path = path.member(b"overrides");
    if let Some(at) = overrides {
        let mut object = match read::Object::new(document, at) {
            Ok(object) => object,
            Err(what) => {
                problems.push(path, What::Jig(what));
                return Some(limits);
            }
        };
        let end = read::after(document, at).ok()?;
        for index in at..end {
            let Some(token) = document.token(index) else { continue };
            if token.kind != skein_json::document::Kind::Key {
                continue;
            }
            let name = document.text(token).expect("object key text");
            let Some(value_at) = object.member(document, name) else { continue };
            match read::integer(document, value_at, u64::MAX) {
                Ok(value) => {
                    if name == b"checkout.preparations" && value == 0 {
                        problems.push(path.member(name), What::Jig(config::What::Because(b"0; requires at least 1")));
                    }
                    if let Err(what) = limits::apply(&mut limits, name, value) {
                        problems.push(path.member(name), What::Override(what));
                    }
                }
                Err(config::What::Bound) => problems.push(path.member(name), What::Override(Refused::Range)),
                Err(what) => problems.push(path.member(name), What::Jig(what)),
            }
        }
        let mut found = config::Problems::empty();
        object.finish(document, &path, &mut found);
        problems.append(found, &config::Path::root());
    }
    Some(limits)
}

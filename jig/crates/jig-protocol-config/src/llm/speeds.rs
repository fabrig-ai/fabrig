//! Model speed declarations (jig/docs/design/protocol/configuration-engine.md, 1).
use crate::{Path, Problems, What, check, read};
use alloc::boxed::Box;
use jig_domain_core_vocabulary::settings::catalogue::{SPEED_ALIASES, SPEED_NAME_BYTES, SPEEDS, speeds_fit};
use jig_domain_core_vocabulary::{Model, Speed};
use skein_json::Document;
use skein_lib::List;

pub(super) struct Speeds {
    declared: Box<[Speed]>,
    default: Option<Box<[u8]>>,
}

impl Speeds {
    pub(super) fn apply(self, model: &mut Model, path: &Path, problems: &mut Problems) {
        model.speeds = self.declared;
        model.speed = self.default;
        if model.speed.is_some() && model.run_speed().is_none() {
            problems.push(path.member(b"speed"), What::Form);
        } else if !speeds_fit(model) {
            problems.push(path.member(b"speeds"), What::Form);
        }
    }
}

pub(super) fn read(
    object: &mut read::Object,
    document: &Document,
    path: &Path,
    problems: &mut Problems,
) -> Option<Speeds> {
    let declared = match object.member(document, b"speeds") {
        Some(at) => declarations(document, at, &path.member(b"speeds"), problems),
        None => Some(Box::default()),
    };
    let default = match object.member(document, b"speed") {
        Some(at) => Some(name(document, at, &path.member(b"speed"), problems)?),
        None => None,
    };
    Some(Speeds { declared: declared?, default })
}

fn declarations(document: &Document, at: u32, path: &Path, problems: &mut Problems) -> Option<Box<[Speed]>> {
    let items = list(document, at, SPEEDS, path, problems)?;
    let mut speeds = List::with_capacity(items.len());
    for (index, at) in items.iter().enumerate() {
        if let Some(speed) = declaration(document, *at, &check::index(path, index), problems) {
            speeds.push(speed).expect("speed list bounds the declarations");
        }
    }
    if speeds.len() != items.len() {
        return None;
    }
    Some(speeds.into_boxed())
}

fn declaration(document: &Document, at: u32, path: &Path, problems: &mut Problems) -> Option<Speed> {
    let mut object = check::object(document, at, path, problems, "speed")?;
    let name = match object.required(document, b"name", path, problems) {
        Some(at) => name(document, at, &path.member(b"name"), problems),
        None => None,
    };
    let also = match object.member(document, b"also") {
        Some(at) => aliases(document, at, &path.member(b"also"), problems),
        None => Some(Box::default()),
    };
    let prices = match object.required(document, b"prices", path, problems) {
        Some(at) => prices(document, at, &path.member(b"prices"), problems),
        None => None,
    };
    object.finish(document, path, problems);
    let prices = prices?;
    Some(Speed {
        name: name?,
        also: also?,
        input_price: prices.input,
        cached_price: prices.cached,
        cache_write_price: prices.cache_write,
        output_price: prices.output,
    })
}

fn prices(document: &Document, at: u32, path: &Path, problems: &mut Problems) -> Option<super::Rates> {
    let mut object = check::object(document, at, path, problems, "speed-prices")?;
    let prices = super::rates(&mut object, document, path, problems);
    object.finish(document, path, problems);
    prices
}

fn aliases(document: &Document, at: u32, path: &Path, problems: &mut Problems) -> Option<Box<[Box<[u8]>]>> {
    let items = list(document, at, SPEED_ALIASES, path, problems)?;
    let mut names = List::with_capacity(items.len());
    for (index, at) in items.iter().enumerate() {
        if let Some(name) = name(document, *at, &check::index(path, index), problems) {
            names.push(name).expect("alias list bounds the names");
        }
    }
    if names.len() != items.len() {
        return None;
    }
    Some(names.into_boxed())
}

fn name(document: &Document, at: u32, path: &Path, problems: &mut Problems) -> Option<Box<[u8]>> {
    match read::text(document, at, SPEED_NAME_BYTES) {
        Ok(name) => Some(name),
        Err(what) => {
            problems.push(path.clone(), what);
            None
        }
    }
}

fn list(document: &Document, at: u32, limit: u32, path: &Path, problems: &mut Problems) -> Option<List<u32>> {
    match read::list(document, at, limit) {
        Ok(items) => Some(items),
        Err(what) => {
            problems.push(path.clone(), what);
            None
        }
    }
}

//! Engine endpoint, model values (llm.md, 4 and 5).
use crate::{Address, Directories, Directory, Path, Problems, Segment, What, check, read};
use alloc::boxed::Box;
use jig_domain_core::{
    Model,
    settings::{CatalogueModel, ModelSource},
};
use skein_json::Document;
use skein_lib::{Duration, List};
/// The most models configuration names: authority keeps a party's models one bit each
/// (protocol/limits.md).
pub(crate) const MODELS: u32 = 64;
// smith/docs/design/protocol/agent.md, section 4: the dialect numbers are fixed for good.
const ANTHROPIC: u32 = 2;
const OPENAI: u32 = 3;
/// The engine's endpoints, providers and default catalogue
/// (jig/docs/design/protocol/configuration.md, 3).
#[derive(Clone, Debug)]
pub struct Llm {
    pub endpoints: Box<[Endpoint]>,
    pub models: Box<[CatalogueModel]>,
    /// Every model belongs to one numbered provider (configuration.md, 3).
    pub providers: Box<[Provider]>,
}
impl Llm {
    /// The catalogue's run models, with a row at each number's position
    /// (jig/docs/design/domain/engine/settings.md, 11).
    #[must_use]
    pub fn models(&self) -> Box<[Model]> {
        let mut highest = 0;
        for entry in &self.models {
            highest = highest.max(entry.number);
        }
        assert!(highest <= MODELS, "catalogue numbers bounded by MODELS");
        let mut models = List::with_capacity(highest);
        for number in 1..=highest {
            let mut model = Model::hole();
            for entry in &self.models {
                if entry.number == number {
                    model = entry.model.clone();
                    break;
                }
            }
            models.push(model).expect("one row per model number");
        }
        models.into_boxed()
    }
}
pub(crate) mod providers;
pub use providers::{Provider, Registration};
#[derive(Clone, Debug)]
pub struct Endpoint {
    pub name: Box<[u8]>,
    pub number: u32,
    pub dialect: u32,
    pub provider: Box<[u8]>,
    pub program: Option<Box<[u8]>>,
    pub address: Address,
    pub transport: Transport,
    pub headers: Box<[Header]>,
    pub connect: Duration,
    pub handshake: Duration,
}
#[derive(Clone, Debug)]
pub struct Header {
    pub name: Box<[u8]>,
    pub value: Box<[u8]>,
}
#[derive(Clone, Debug)]
pub enum Transport {
    Tls { server_name: Box<[u8]>, trust: Option<Box<[u8]>> },
    Plaintext,
}
pub(crate) fn parse(
    document: &Document,
    at: u32,
    path: &Path,
    directories: &Directories,
    defaults: crate::Defaults,
    problems: &mut Problems,
) -> Option<Llm> {
    let mut object = check::object(document, at, path, problems, "llm")?;
    let mut endpoints = List::<Endpoint>::with_capacity(document.len());
    if let Some(items) = check::list(&mut object, document, b"endpoints", document.len(), path, problems) {
        for (index, at) in items.iter().enumerate() {
            let path =
                path.member(b"endpoints").child(Segment::Index(u32::try_from(index).expect("list bounded by u32")));
            if let Some(value) = endpoint(document, *at, &path, directories, problems) {
                for previous in &endpoints {
                    if previous.name == value.name {
                        problems.push(path.member(b"name"), What::Unique);
                    }
                    if previous.number == value.number {
                        problems.push(path.member(b"number"), What::Unique);
                    }
                }
                endpoints.push(value).expect("input count bounded");
            }
        }
    }
    let mut models = List::<CatalogueModel>::with_capacity(document.len());
    if defaults == crate::Defaults::Passed {
        if object.member(document, b"models").is_some() {
            problems.push(path.member(b"models"), What::Stored);
        }
    } else if let Some(items) = check::list(&mut object, document, b"models", MODELS, path, problems) {
        for (index, at) in items.iter().enumerate() {
            let path = path.member(b"models").child(Segment::Index(u32::try_from(index).expect("list bounded by u32")));
            if let Some(value) = model(document, *at, &path, endpoints.as_slice(), problems) {
                for previous in &models {
                    if previous.model.name == value.model.name {
                        problems.push(path.member(b"name"), What::Unique);
                        break;
                    }
                }
                models
                    .push(CatalogueModel {
                        first_read: None,
                        number: u32::try_from(index)
                            .expect("bounded model position")
                            .checked_add(1)
                            .expect("model number"),
                        model: value.model,
                        shown: value.shown,
                        description: value.description,
                        picker: value.picker,
                        withdrawn: false,
                        source: ModelSource::Stock,
                        estimated: false,
                    })
                    .expect("input count bounded");
            }
        }
    }
    let providers = providers::parse(&mut object, document, path, models.as_slice(), directories, defaults, problems);
    for (index, provider) in providers.iter().enumerate() {
        if let Some(name) = &provider.endpoint {
            let mut found = false;
            for endpoint in &endpoints {
                found |= endpoint.name == *name;
            }
            if !found {
                problems.push(check::index(&path.member(b"providers"), index).member(b"endpoint"), What::Refers);
            }
        }
    }
    program_bindings(endpoints.as_slice(), models.as_slice(), providers.as_slice(), path, problems);
    for model in models.iter_mut() {
        for provider in &providers {
            if provider.models.contains(&model.model.name) {
                model.model.provider = provider.number;
            }
        }
    }
    object.finish(document, path, problems);
    Some(Llm { endpoints: endpoints.into_boxed(), models: models.into_boxed(), providers: providers.into_boxed() })
}
/// A shared endpoint must name a program offered by every provider whose models use it.
fn program_bindings(
    endpoints: &[Endpoint],
    models: &[CatalogueModel],
    providers: &[Provider],
    path: &Path,
    problems: &mut Problems,
) {
    for (index, endpoint) in endpoints.iter().enumerate() {
        let Some(program) = &endpoint.program else { continue };
        let at = check::index(&path.member(b"endpoints"), index).member(b"program");
        let mut valid = false;
        let mut refused = program.is_empty() || program.len() > skein_lib::length::usize_of(jig_domain_accounts::NAME);
        for provider in providers {
            if provider.endpoint.as_deref() == Some(endpoint.name.as_ref()) {
                valid = true;
                if !provider.programs.contains(program) {
                    refused = true;
                }
            }
        }
        for model in models {
            if model.model.endpoint != endpoint.number {
                continue;
            }
            for provider in providers {
                if provider.models.contains(&model.model.name) {
                    valid = true;
                    if !provider.programs.contains(program) {
                        refused = true;
                    }
                }
            }
        }
        if !valid || refused {
            problems.push(at, What::Refers);
        }
    }
}

fn endpoint(
    document: &Document,
    at: u32,
    path: &Path,
    directories: &Directories,
    problems: &mut Problems,
) -> Option<Endpoint> {
    let mut object = check::object(document, at, path, problems, "endpoint")?;
    let name = check::text(&mut object, document, b"name", path, problems);
    let number = check::count(&mut object, document, b"number", path, problems);
    let dialect = check::count(&mut object, document, b"dialect", path, problems);
    if let Some(dialect) = dialect {
        match dialect {
            ANTHROPIC | OPENAI => {}
            _ => problems.push(path.member(b"dialect"), What::Form),
        }
    }
    let provider = check::text(&mut object, document, b"provider", path, problems);
    let program = optional_text(&mut object, document, b"program", path, problems);
    let address = check::address(&mut object, document, b"address", path, problems);
    if let Some(address) = &address {
        network_address(address, &path.member(b"address"), problems);
    }
    let transport = transport(&mut object, document, path, b"address", address.as_ref(), directories, problems);
    let headers = headers(&mut object, document, path, problems);
    let connect = check::duration(&mut object, document, b"connect_ms", path, problems);
    let handshake = check::duration(&mut object, document, b"handshake_ms", path, problems);
    object.finish(document, path, problems);
    Some(Endpoint {
        name: name?,
        number: number?,
        dialect: dialect?,
        provider: provider?,
        program,
        address: address?,
        transport: transport?,
        headers: headers.into_boxed(),
        connect: connect?,
        handshake: handshake?,
    })
}
fn transport(
    object: &mut read::Object,
    document: &Document,
    path: &Path,
    address_name: &[u8],
    address: Option<&Address>,
    directories: &Directories,
    problems: &mut Problems,
) -> Option<Transport> {
    let kind = check::text(object, document, b"transport", path, problems)?;
    match kind.as_ref() {
        b"plaintext" => {
            if let Some(address) = address {
                check::local(address, &path.member(address_name), problems);
            }
            Some(Transport::Plaintext)
        }
        b"tls" => {
            let server_name = check::text(object, document, b"server_name", path, problems);
            let trust =
                check::optional_path(object, document, b"trust", path, Directory::Config, directories, problems);
            Some(Transport::Tls { server_name: server_name?, trust })
        }
        _ => {
            problems.push(path.member(b"transport"), What::Form);
            None
        }
    }
}
fn optional_text(
    object: &mut read::Object,
    document: &Document,
    name: &[u8],
    path: &Path,
    problems: &mut Problems,
) -> Option<Box<[u8]>> {
    let at = object.member(document, name)?;
    match read::text(document, at, 4_096) {
        Ok(value) => Some(value),
        Err(what) => {
            problems.push(path.member(name), what);
            None
        }
    }
}
fn headers(object: &mut read::Object, document: &Document, path: &Path, problems: &mut Problems) -> List<Header> {
    let mut result = List::with_capacity(document.len());
    if let Some(items) = check::list(object, document, b"headers", document.len(), path, problems) {
        for (index, at) in items.iter().enumerate() {
            let path = path.member(b"headers").child(Segment::Index(u32::try_from(index).expect("bounded list")));
            if let Some(mut object) = check::object(document, *at, &path, problems, "header") {
                let name = check::text(&mut object, document, b"name", &path, problems);
                let value = check::text(&mut object, document, b"value", &path, problems);
                object.finish(document, &path, problems);
                if let Some(name) = name
                    && let Some(value) = value
                {
                    result.push(Header { name, value }).expect("input count bounded");
                }
            }
        }
    }
    result
}
struct ParsedModel {
    model: Model,
    description: Option<Box<[u8]>>,
    shown: Option<Box<[u8]>>,
    picker: bool,
}
fn model(
    document: &Document,
    at: u32,
    path: &Path,
    endpoints: &[Endpoint],
    problems: &mut Problems,
) -> Option<ParsedModel> {
    let mut object = check::object(document, at, path, problems, "model")?;
    let mut name = None;
    if let Some(at) = object.required(document, b"name", path, problems) {
        match read::text(document, at, jig_domain_core::MODEL_NAME_BYTES) {
            Ok(value) => name = Some(value),
            Err(what) => problems.push(path.member(b"name"), what),
        }
    }
    let endpoint = check::text(&mut object, document, b"endpoint", path, problems);
    let window = check::count(&mut object, document, b"window", path, problems);
    let cache_life_ms = match object.member(document, b"cache_life_ms") {
        Some(at) => match read::integer(document, at, u64::from(u32::MAX)) {
            Ok(value) => u32::try_from(value).expect("cache life bounded by u32"),
            Err(what) => {
                problems.push(path.member(b"cache_life_ms"), what);
                0
            }
        },
        None => 0,
    };
    let output = check::count(&mut object, document, b"output", path, problems);
    let mut efforts = Some(Box::default());
    if let Some(at) = object.member(document, b"efforts") {
        efforts = model_efforts(document, at, &path.member(b"efforts"), problems);
    }
    let mut effort = None;
    if let Some(at) = object.member(document, b"effort") {
        match read::text(document, at, jig_domain_core::EFFORT_BYTES) {
            Ok(value) if value.is_empty() => problems.push(path.member(b"effort"), What::Form),
            Ok(value) => {
                if let Some(efforts) = &efforts
                    && !efforts.contains(&value)
                {
                    problems.push(path.member(b"effort"), What::Refers);
                }
                effort = Some(value);
            }
            Err(what) => problems.push(path.member(b"effort"), what),
        }
    }
    let mut prices = None;
    if let Some(at) = object.required(document, b"prices", path, problems) {
        prices = price(document, at, &path.member(b"prices"), problems);
    }
    let mut description = None;
    if let Some(at) = object.member(document, b"description") {
        match read::text(document, at, jig_domain_core::MODEL_DESCRIPTION_BYTES) {
            Ok(value) if value.is_empty() => problems.push(path.member(b"description"), What::Form),
            Ok(value) => description = Some(value),
            Err(what) => problems.push(path.member(b"description"), what),
        }
    }
    let mut shown = None;
    if let Some(at) = object.member(document, b"shown") {
        match read::text(document, at, 256) {
            Ok(value) if value.is_empty() => problems.push(path.member(b"shown"), What::Form),
            Ok(value) => shown = Some(value),
            Err(what) => problems.push(path.member(b"shown"), what),
        }
    }
    let mut picker = true;
    if let Some(at) = object.member(document, b"picker") {
        match read::boolean(document, at) {
            Ok(value) => picker = value,
            Err(what) => problems.push(path.member(b"picker"), what),
        }
    }
    object.finish(document, path, problems);
    let mut found = None;
    if let Some(name) = endpoint {
        for endpoint in endpoints {
            if endpoint.name == name {
                found = Some(endpoint);
                break;
            }
        }
        if found.is_none() {
            problems.push(path.member(b"endpoint"), What::Refers);
        }
    }
    let endpoint = found?;
    let mut model = prices?;
    model.name = name?;
    model.dialect = endpoint.dialect;
    model.endpoint = endpoint.number;
    model.program.clone_from(&endpoint.program);
    model.window = window?;
    model.cache_life_ms = cache_life_ms;
    model.output = output?;
    model.efforts = efforts?;
    model.effort = effort;
    Some(ParsedModel { model, description, shown, picker })
}
/// The efforts a model takes, by name in order, none when absent (configuration.md, 3).
fn model_efforts(document: &Document, at: u32, path: &Path, problems: &mut Problems) -> Option<Box<[Box<[u8]>]>> {
    let items = match read::list(document, at, jig_domain_core::EFFORTS) {
        Ok(items) => items,
        Err(what) => {
            problems.push(path.clone(), what);
            return None;
        }
    };
    let mut efforts = List::<Box<[u8]>>::with_capacity(items.len());
    let mut whole = true;
    for (index, at) in items.iter().enumerate() {
        let path = check::index(path, index);
        match read::text(document, *at, jig_domain_core::EFFORT_BYTES) {
            Ok(effort) if effort.is_empty() => {
                problems.push(path, What::Form);
                whole = false;
            }
            Ok(effort) if efforts.as_slice().contains(&effort) => {
                problems.push(path, What::Unique);
                whole = false;
            }
            Ok(effort) => efforts.push(effort).expect("efforts bounded by the list"),
            Err(what) => {
                problems.push(path, what);
                whole = false;
            }
        }
    }
    if !whole {
        return None;
    }
    Some(efforts.into_boxed())
}
fn price(document: &Document, at: u32, path: &Path, problems: &mut Problems) -> Option<Model> {
    let mut object = check::object(document, at, path, problems, "prices")?;
    let input_price = check::integer(&mut object, document, b"input", path, problems);
    let cached_price = check::integer(&mut object, document, b"cached", path, problems);
    let output_price = check::integer(&mut object, document, b"output", path, problems);
    let price_unit = check::count(&mut object, document, b"unit", path, problems);
    let mut cache_write_price = None;
    if let Some(at) = object.member(document, b"cache_write") {
        match read::integer(document, at, u64::MAX) {
            Ok(value) => cache_write_price = Some(value),
            Err(what) => problems.push(path.member(b"cache_write"), what),
        }
    }
    for (member, zero) in [
        (b"input".as_slice(), input_price == Some(0)),
        (b"cached".as_slice(), cached_price == Some(0)),
        (b"output".as_slice(), output_price == Some(0)),
        (b"unit".as_slice(), price_unit == Some(0)),
        (b"cache_write".as_slice(), cache_write_price == Some(0)),
    ] {
        if zero {
            problems.push(path.member(member), What::Bound);
        }
    }
    object.finish(document, path, problems);
    Some(Model {
        provider: 0,
        name: Box::default(),
        dialect: 0,
        endpoint: 0,
        program: None,
        window: 0,
        cache_life_ms: 0,
        output: 0,
        input_price: input_price?,
        cached_price: cached_price?,
        cache_write_price,
        output_price: output_price?,
        price_unit: price_unit?,
        efforts: Box::default(),
        effort: None,
    })
}

// LLM and OAuth dial socket addresses, with host resolution at startup.
fn network_address(address: &Address, path: &Path, problems: &mut Problems) {
    match address {
        Address::Addr(skein_io::Addr::InProcess(_)) => problems.push(path.clone(), What::Form),
        Address::Addr(skein_io::Addr::Ip(_)) | Address::Host { .. } => {}
    }
}

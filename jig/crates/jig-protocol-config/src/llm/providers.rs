//! LLM providers: each one's number, its name for showing, the configured models it serves and
//! the sources it offers (configuration.md, 3 and 6; accounts.md, 3.4). No account, key or record.
use crate::{Directories, Path, Problems, What, check, read};
use alloc::boxed::Box;
use jig_domain_core_vocabulary::settings::CatalogueModel;
use skein_json::Document;
use skein_lib::List;

use jig_domain_core_vocabulary::settings::providers as shape;
pub use jig_domain_core_vocabulary::settings::{Provider, Registration};
/// The sources a provider offers, each optional: `{}` offers none.
#[derive(Debug)]
struct Sources {
    sign_in: Option<Registration>,
    keys: bool,
    programs: Box<[Box<[u8]>]>,
}

/// Every configured model belongs to exactly one provider, even when none are named.
pub(super) fn parse(
    object: &mut read::Object,
    document: &Document,
    path: &Path,
    models: &mut [CatalogueModel],
    directories: &Directories,
    defaults: crate::Defaults,
    problems: &mut Problems,
) -> List<Provider> {
    let mut providers = List::<Provider>::with_capacity(document.len());
    let Some(at) = object.member(document, b"providers") else {
        for (index, _) in models.iter().enumerate() {
            problems.push(check::index(&path.member(b"models"), index).member(b"name"), What::NoProvider);
        }
        return providers;
    };
    let items = match read::list(document, at, document.len()) {
        Ok(items) => items,
        Err(what) => {
            problems.push(path.member(b"providers"), what);
            return providers;
        }
    };
    let mut claimed = List::<bool>::with_capacity(u32::try_from(models.len()).expect("models bounded by u32"));
    for _ in models.iter() {
        claimed.push(false).expect("one flag per model");
    }
    for (index, at) in items.iter().enumerate() {
        let path = check::index(&path.member(b"providers"), index);
        if let Some(Parsed { value, served }) =
            provider(document, *at, &path, models, &mut claimed, directories, defaults, false, problems)
        {
            for model in models.iter_mut() {
                if served.contains(&model.model.name) {
                    model.model.provider = value.number;
                }
            }
            for previous in &providers {
                if previous.name == value.name {
                    problems.push(path.member(b"name"), What::Unique);
                }
                if previous.number == value.number {
                    problems.push(path.member(b"number"), What::Unique);
                }
            }
            providers.push(value).expect("input count bounded");
        }
    }
    for (index, claimed) in claimed.iter().enumerate() {
        if !claimed {
            problems.push(check::index(&path.member(b"models"), index).member(b"name"), What::NoProvider);
        }
    }
    providers
}

pub(super) struct Parsed {
    pub value: Provider,
    pub served: Box<[Box<[u8]>]>,
}
#[expect(clippy::too_many_arguments, reason = "provider parsing carries model claims and the JSON form")]
fn provider(
    document: &Document,
    at: u32,
    path: &Path,
    models: &[CatalogueModel],
    claimed: &mut List<bool>,
    directories: &Directories,
    defaults: crate::Defaults,
    setting: bool,
    problems: &mut Problems,
) -> Option<Parsed> {
    let mut object = if setting {
        super::setting::object(document, at, path, problems)?
    } else {
        check::object(document, at, path, problems, "llm-provider")?
    };
    let name = public_text(&mut object, document, b"name", jig_domain_accounts::PROVIDER_NAME, path, problems);
    let signed_at = object.member(document, b"signed_in_plan");
    let signed_in_plan = match signed_at {
        Some(at) => match read::boolean(document, at) {
            Ok(value) => value,
            Err(what) => {
                problems.push(path.member(b"signed_in_plan"), what);
                false
            }
        },
        None => false,
    };
    let manage_usage = match object.member(document, b"manage_usage") {
        Some(at) => match read::text(document, at, jig_domain_accounts::MANAGE_USAGE) {
            Ok(value) if public_address(&value) => Some(value),
            Ok(_) => {
                problems.push(path.member(b"manage_usage"), What::Form);
                None
            }
            Err(what) => {
                problems.push(path.member(b"manage_usage"), what);
                None
            }
        },
        None => None,
    };
    let number = check::count(&mut object, document, b"number", path, problems);
    let mut served = None;
    if defaults == crate::Defaults::Passed {
        if object.member(document, b"models").is_some() {
            problems.push(path.member(b"models"), What::Stored);
        }
        served = Some(Box::default());
    } else if let Some(items) = check::list(&mut object, document, b"models", document.len(), path, problems) {
        served = served_models(document, &items, &path.member(b"models"), models, claimed, problems);
    }
    let mut offered = None;
    if let Some(at) = object.required(document, b"sources", path, problems) {
        offered = sources(document, at, &path.member(b"sources"), directories, problems);
    }
    let endpoint = if defaults == crate::Defaults::Passed && !setting {
        check::text(&mut object, document, b"endpoint", path, problems)
    } else {
        super::optional_text(&mut object, document, b"endpoint", path, problems)
    };
    let direct = super::optional_text(&mut object, document, b"direct", path, problems);
    let keep_at = object.member(document, b"keep");
    let drop_at = object.member(document, b"drop");
    let keep = match keep_at {
        Some(at) => patterns(document, at, &path.member(b"keep"), problems),
        None => None,
    };
    let drop = match drop_at {
        Some(at) => patterns(document, at, &path.member(b"drop"), problems),
        None => Some(Box::default()),
    };
    if keep_at.is_some() && endpoint.is_none() {
        problems.push(path.member(b"keep"), What::Refers);
    }
    if drop_at.is_some() && keep_at.is_none() {
        problems.push(path.member(b"drop"), What::Refers);
    }
    let list = match keep {
        Some(keep) => Some(jig_domain_accounts::ProviderList { keep, drop: drop? }),
        None => None,
    };
    if setting {
        let _endpoints = object.member(document, b"endpoints");
        let _withdrawn = object.member(document, b"withdrawn");
    }
    object.finish(document, path, problems);
    let offered = offered?;
    if offered.sign_in.is_some() && signed_at.is_none() {
        problems.push(path.member(b"signed_in_plan"), What::Missing);
    }
    let value = Provider {
        name: name?,
        number: number?,
        endpoint,
        direct,
        list,
        signed_in_plan,
        manage_usage,
        sign_in: offered.sign_in,
        keys: offered.keys,
        programs: offered.programs,
    };
    match shape::terms(&value) {
        Some(shape::TermsFault::SignedInPlan) => problems.push(path.member(b"signed_in_plan"), What::Form),
        Some(shape::TermsFault::ManageUsage) => problems.push(path.member(b"manage_usage"), What::Missing),
        None => {}
    }
    Some(Parsed { value, served: served? })
}

/// Bounded whole-name patterns (jig/docs/design/protocol/configuration.md, 3).
fn patterns(document: &Document, at: u32, path: &Path, problems: &mut Problems) -> Option<Box<[Box<[u8]>]>> {
    let items = match read::list(document, at, 16) {
        Ok(items) => items,
        Err(what) => {
            problems.push(path.clone(), what);
            return None;
        }
    };
    let mut patterns = List::with_capacity(items.len());
    let mut failed = false;
    for (index, at) in items.iter().enumerate() {
        let path = check::index(path, index);
        match read::text(document, *at, 64) {
            Ok(pattern) => {
                let valid = shape::pattern(&pattern);
                if valid {
                    patterns.push(pattern).expect("pattern count bounded");
                } else {
                    problems.push(path, What::Form);
                    failed = true;
                }
            }
            Err(what) => {
                problems.push(path, what);
                failed = true;
            }
        }
    }
    if failed {
        return None;
    }
    Some(patterns.into_boxed())
}

/// Each name a configured model, claimed by this provider alone.
fn served_models(
    document: &Document,
    items: &List<u32>,
    path: &Path,
    models: &[CatalogueModel],
    claimed: &mut List<bool>,
    problems: &mut Problems,
) -> Option<Box<[Box<[u8]>]>> {
    let mut names = List::<Box<[u8]>>::with_capacity(items.len());
    let mut failed = false;
    for (index, at) in items.iter().enumerate() {
        let path = check::index(path, index);
        let name = match read::text(document, *at, 4_096) {
            Ok(name) => name,
            Err(what) => {
                problems.push(path, what);
                failed = true;
                continue;
            }
        };
        let mut found = None;
        for (model, value) in models.iter().enumerate() {
            if value.model.name == name {
                found = Some(u32::try_from(model).expect("models bounded by u32"));
                break;
            }
        }
        match found {
            Some(model) => {
                let flag = claimed.get_mut(model).expect("one flag per model");
                if *flag {
                    problems.push(path, What::Unique);
                    failed = true;
                }
                *flag = true;
            }
            None => {
                problems.push(path, What::Refers);
                failed = true;
            }
        }
        names.push(name).expect("input count bounded");
    }
    if failed {
        return None;
    }
    Some(names.into_boxed())
}

/// The sources offered: `sign_in`, `keys` and `programs`, each optional.
fn sources(
    document: &Document,
    at: u32,
    path: &Path,
    directories: &Directories,
    problems: &mut Problems,
) -> Option<Sources> {
    let mut object = check::object(document, at, path, problems, "sources")?;
    let mut failed = false;
    let mut sign_in = None;
    if let Some(at) = object.member(document, b"sign_in") {
        sign_in = registration(document, at, &path.member(b"sign_in"), directories, problems);
        if sign_in.is_none() {
            failed = true;
        }
    }
    let mut keys = false;
    if let Some(at) = object.member(document, b"keys") {
        match read::boolean(document, at) {
            Ok(value) => keys = value,
            Err(what) => {
                problems.push(path.member(b"keys"), what);
                failed = true;
            }
        }
    }
    let mut programs = Some(List::<Box<[u8]>>::with_capacity(0));
    if let Some(at) = object.member(document, b"programs") {
        programs = program_names(document, at, &path.member(b"programs"), problems);
    }
    object.finish(document, path, problems);
    let programs = programs?;
    if failed {
        return None;
    }
    Some(Sources { sign_in, keys, programs: programs.into_boxed() })
}

/// A public registration: a client secret makes it confidential, which is refused
/// (configuration.md, 6).
fn registration(
    document: &Document,
    at: u32,
    path: &Path,
    directories: &Directories,
    problems: &mut Problems,
) -> Option<Registration> {
    let mut object = check::object(document, at, path, problems, "registration")?;
    let issuer = present(check::text(&mut object, document, b"issuer", path, problems), b"issuer", path, problems);
    let client = present(check::text(&mut object, document, b"client", path, problems), b"client", path, problems);
    let mut scopes = None;
    if let Some(items) = check::list(&mut object, document, b"scopes", document.len(), path, problems) {
        scopes = scope_tokens(document, &items, &path.member(b"scopes"), problems);
    }
    let name_hint = super::optional_text(&mut object, document, b"name_hint", path, problems);
    let authorization =
        present(check::text(&mut object, document, b"authorization", path, problems), b"authorization", path, problems);
    let token = present(check::text(&mut object, document, b"token", path, problems), b"token", path, problems);
    let revocation = super::optional_text(&mut object, document, b"revocation", path, problems);
    let keys = present(check::text(&mut object, document, b"keys", path, problems), b"keys", path, problems);
    let endpoint_transport = super::transport(&mut object, document, path, b"token", None, directories, problems);
    let mut must_grant = None;
    if let Some(items) = check::list(&mut object, document, b"must_grant", document.len(), path, problems) {
        must_grant = scope_tokens(document, &items, &path.member(b"must_grant"), problems);
    }
    if let (Some(scopes), Some(required)) = (&scopes, &must_grant) {
        for (index, scope) in required.iter().enumerate() {
            if !scopes.contains(scope) {
                problems.push(check::index(&path.member(b"must_grant"), index), What::Refers);
            }
        }
    }
    let resource = super::optional_text(&mut object, document, b"resource", path, problems);
    let redirect_path = check::text(&mut object, document, b"redirect_path", path, problems);
    if let Some(value) = &redirect_path
        && !shape::redirect_path(value)
    {
        problems.push(path.member(b"redirect_path"), What::Form);
    }
    let mut redirect_port = None;
    if let Some(at) = object.member(document, b"redirect_port") {
        match read::integer(document, at, u64::from(u16::MAX)) {
            Ok(value) if value > 0 => redirect_port = Some(u16::try_from(value).expect("port bounded")),
            Ok(_) => problems.push(path.member(b"redirect_port"), What::Form),
            Err(what) => problems.push(path.member(b"redirect_port"), what),
        }
    }
    let secret = object.member(document, b"secret");
    if secret.is_some() {
        problems.push(path.member(b"secret"), What::Confidential);
    }
    object.finish(document, path, problems);
    if secret.is_some() {
        return None;
    }
    Some(Registration {
        issuer: issuer?,
        client: client?,
        scopes: scopes?,
        name_hint,
        authorization: authorization?,
        token: token?,
        revocation,
        keys: keys?,
        endpoint_transport: endpoint_transport?,
        must_grant: must_grant?,
        resource,
        redirect_path: redirect_path?,
        redirect_port,
    })
}

/// RFC 6749, 3.3: a scope token is one or more of `%x21 / %x23-5B / %x5D-7E`.
fn scope_tokens(
    document: &Document,
    items: &List<u32>,
    path: &Path,
    problems: &mut Problems,
) -> Option<Box<[Box<[u8]>]>> {
    let mut scopes = List::<Box<[u8]>>::with_capacity(items.len());
    let mut failed = false;
    for (index, at) in items.iter().enumerate() {
        let path = check::index(path, index);
        if index >= skein_lib::usize_of(shape::SCOPES) {
            problems.push(path, What::Form);
            failed = true;
            continue;
        }
        match read::text(document, *at, 4_096) {
            Ok(scope) => {
                let token = shape::scope_token(&scope);
                if token {
                    scopes.push(scope).expect("input count bounded");
                } else {
                    problems.push(path, What::Form);
                    failed = true;
                }
            }
            Err(what) => {
                problems.push(path, what);
                failed = true;
            }
        }
    }
    if failed {
        return None;
    }
    Some(scopes.into_boxed())
}

/// Program names, unique, each 1 to 32 of `a-z`, `0-9`, `-` and `_`: `claude_code` names Claude
/// Code (smith's programs.md, 3).
fn program_names(document: &Document, at: u32, path: &Path, problems: &mut Problems) -> Option<List<Box<[u8]>>> {
    let items = match read::list(document, at, document.len()) {
        Ok(items) => items,
        Err(what) => {
            problems.push(path.clone(), what);
            return None;
        }
    };
    let mut names = List::<Box<[u8]>>::with_capacity(items.len());
    let mut failed = false;
    for (index, at) in items.iter().enumerate() {
        let path = check::index(path, index);
        if index >= skein_lib::usize_of(shape::PROGRAMS) {
            problems.push(path, What::Form);
            failed = true;
            continue;
        }
        match read::text(document, *at, 4_096) {
            Ok(name) => {
                let form = shape::program_name(&name);
                if !form {
                    problems.push(path, What::Form);
                    failed = true;
                } else if names.as_slice().contains(&name) {
                    problems.push(path, What::Unique);
                    failed = true;
                } else {
                    names.push(name).expect("input count bounded");
                }
            }
            Err(what) => {
                problems.push(path, what);
                failed = true;
            }
        }
    }
    if failed {
        return None;
    }
    Some(names)
}

/// A required text that is not empty.
fn present(value: Option<Box<[u8]>>, name: &[u8], path: &Path, problems: &mut Problems) -> Option<Box<[u8]>> {
    let value = value?;
    if value.is_empty() {
        problems.push(path.member(name), What::Form);
        return None;
    }
    Some(value)
}

/// Text is nonempty, UTF-8, and free of control characters; diagnostics name its exact field.
fn public_text(
    object: &mut read::Object,
    document: &Document,
    name: &[u8],
    max: u32,
    path: &Path,
    problems: &mut Problems,
) -> Option<Box<[u8]>> {
    let at = object.required(document, name, path, problems)?;
    let value = match read::text(document, at, max) {
        Ok(value) => value,
        Err(what) => {
            problems.push(path.member(name), what);
            return None;
        }
    };
    if !shape::public_text(&value, max) {
        problems.push(path.member(name), What::Form);
        return None;
    }
    Some(value)
}

pub(crate) use jig_domain_core_vocabulary::settings::providers::{public_address, public_authority};

pub(super) fn setting(document: &Document, at: u32, path: &Path, problems: &mut Problems) -> Option<Provider> {
    let mut claimed = List::with_capacity(0);
    let parsed = provider(
        document,
        at,
        path,
        &[],
        &mut claimed,
        &Directories::working(),
        crate::Defaults::Passed,
        true,
        problems,
    )?;
    Some(parsed.value)
}

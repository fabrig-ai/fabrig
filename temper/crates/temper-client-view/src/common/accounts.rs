//! Shared setup items (temper/docs/design/client/view-crates.md, 3.3).
use crate::{
    accounts::words::{self, Word as W},
    words::Words,
};
use alloc::boxed::Box;
use jig_client::contract as c;
use skein_ui::{Builder, Run, phrases::Hole};
use temper_client_domain::{Domain, pages::accounts};

pub(crate) fn run(b: &mut Builder, w: &Words, word: W) {
    b.run(Run::Plain, &words::say(w, word, &[]));
}

pub(crate) fn named(w: &Words, word: W, name: &[u8]) -> Box<[u8]> {
    words::say(w, word, &[Hole::Name(crate::words::shown(name, w))])
}

pub(crate) fn provider_name(name: Option<&c::DocumentText>) -> Option<&[u8]> {
    let name = name?;
    if name.cut || name.bytes().is_empty() { None } else { Some(name.bytes()) }
}

#[expect(clippy::too_many_arguments, reason = "party and deployment rows share these borrowed facts")]
pub(crate) fn heading(
    w: &Words,
    number_value: u32,
    provider: Option<u32>,
    name: Option<&[u8]>,
    display: &c::DocumentText,
    email: Option<&[u8]>,
    held: c::AccountHolder,
    plan: bool,
    sources: &skein_lib::List<c::AccountSource>,
    me: Option<&c::Me>,
) -> Box<[u8]> {
    let separator = words::say(w, W::Separator, &[]);
    let display = words::who(email, display.bytes());
    let Some(name) = name else {
        let account = words::say(w, W::Account, &[]);
        let number = words::say(w, W::Number, &[Hole::Number(u64::from(number_value))]);
        let provider_label = words::say(w, W::Provider, &[]);
        let provider_number = match provider {
            Some(n) => words::say(w, W::Number, &[Hole::Number(u64::from(n))]),
            None => Box::new([]),
        };
        return words::join(&[
            &account,
            &number,
            if provider.is_some() { &separator } else { b"" },
            if provider.is_some() { &provider_label } else { b"" },
            &provider_number,
            if display.is_empty() { b"" } else { &separator },
            display,
        ]);
    };
    let (brand, plan_name) = words::provider_words(name);
    let mut program = false;
    let mut program_name = None;
    let mut sign_in = false;
    for source in sources {
        program |= source.kind == c::AccountKind::Program;
        if source.kind == c::AccountKind::Program && program_name.is_none() {
            program_name = source.name.as_deref();
        }
        sign_in |= source.kind == c::AccountKind::SignIn;
    }
    let label = if program {
        named(w, W::Subscription, plan_name)
    } else if plan || sign_in {
        crate::words::shown(if plan { plan_name } else { brand }, w)
    } else {
        named(w, W::ApiKeyBrand, brand)
    };
    let owner = match held {
        c::AccountHolder::Deployment => words::say(w, W::HeldDeployment, &[]),
        c::AccountHolder::Party(_) => match me {
            Some(me) => named(w, W::HeldBy, me.party.name.bytes()),
            None => named(w, W::HeldBy, &words::say(w, W::OwnHolder, &[])),
        },
    };
    let display = if program {
        words::who(email, program_name.unwrap_or(display))
    } else if plan || sign_in {
        display
    } else {
        b""
    };
    let brand = crate::words::shown(brand, w);
    words::join(&[
        &label,
        if display.is_empty() { b"" } else { &separator },
        display,
        if program || plan || sign_in { &separator } else { b"" },
        if program || plan || sign_in { &brand } else { b"" },
        &separator,
        &owner,
    ])
}

pub(crate) fn unloaded(b: &mut Builder, w: &Words, at: c::AccountPlace, why: c::AccountUnread) {
    run(b, w, W::Unloaded);
    run(
        b,
        w,
        match at {
            c::AccountPlace::Directory => W::Directory,
            c::AccountPlace::File => W::File,
        },
    );
    b.run(Run::Plain, b": ");
    run(
        b,
        w,
        match why {
            c::AccountUnread::Unreadable => W::Unreadable,
            c::AccountUnread::Refused => W::Refused,
            c::AccountUnread::Failed => W::Failed,
            c::AccountUnread::Stalled => W::Stalled,
            c::AccountUnread::TooLarge => W::TooLarge,
        },
    );
}

/// An account's heading as the accounts page draws it (5.5, An account).
pub(crate) fn account_heading(root: &Domain, w: &Words, row: &c::AccountLine) -> Box<[u8]> {
    let supplied = provider_name(row.provider_name.as_ref()).unwrap_or(b"");
    let shown = crate::choice::provider_shown(root, row.provider, supplied, w);
    let name = if supplied.is_empty() { None } else { Some(shown.as_ref()) };
    heading(
        w,
        row.number,
        row.provider,
        name,
        &row.display,
        row.email(),
        row.holder,
        row.plan,
        &row.sources,
        accounts::current_me(root.client()),
    )
}

pub(crate) fn first_usable_kind(root: &Domain, w: &Words) -> Option<Box<[u8]>> {
    first_usable(root, w, false)
}

/// Home's first usable source and the kinds of accounts the person holds (3.5, More accounts).
pub(crate) fn home_accounts(root: &Domain) -> (bool, crate::words::Word) {
    use crate::words::Word;
    let mut first_key = None;
    let mut chatgpt = false;
    let mut claude = false;
    let mut other = false;
    if let Some(document) = accounts::document(root.client()) {
        for row in &document.accounts {
            if playground(root, row.provider) {
                continue;
            }
            for source in &row.sources {
                if !source.usable {
                    continue;
                }
                if first_key.is_none() {
                    first_key = Some(source.kind == c::AccountKind::Key);
                }
                let provider = provider_name(row.provider_name.as_ref()).unwrap_or(b"");
                match source.kind {
                    c::AccountKind::SignIn if row.plan && provider == b"openai" => chatgpt = true,
                    c::AccountKind::Program if provider == b"anthropic" => claude = true,
                    c::AccountKind::SignIn | c::AccountKind::Program | c::AccountKind::Key | c::AccountKind::None => {
                        other = true;
                    }
                }
            }
        }
    }
    let more = if chatgpt && !claude && !other {
        Word::HomeAddClaude
    } else if claude && !chatgpt && !other {
        Word::HomeAddChatGpt
    } else {
        Word::HomeAddAccount
    };
    (first_key.unwrap_or(false), more)
}

pub(crate) fn first_usable(root: &Domain, w: &Words, with_display: bool) -> Option<Box<[u8]>> {
    let (row, source) = usable_account(root)?;
    Some(account_words(root, w, row, source, with_display))
}

pub(crate) fn usable_account(root: &Domain) -> Option<(&c::AccountLine, &c::AccountSource)> {
    let document = accounts::document(root.client())?;
    for row in &document.accounts {
        if playground(root, row.provider) {
            continue;
        }
        let mut usable = None;
        for source in &row.sources {
            if source.usable {
                usable = Some(source);
                break;
            }
        }
        let Some(source) = usable else { continue };
        return Some((row, source));
    }
    None
}

/// Account and identity words shared by Ready and a trial's selected account (domain/trial.md, 6.1).
pub(crate) fn account_words(
    root: &Domain,
    w: &Words,
    row: &c::AccountLine,
    source: &c::AccountSource,
    with_display: bool,
) -> Box<[u8]> {
    let Some(supplied) = provider_name(row.provider_name.as_ref()) else {
        return account_heading(root, w, row);
    };
    let (brand, plan_name) = words::provider_words(supplied);
    let (label, display) = match source.kind {
        c::AccountKind::Program => (named(w, W::Subscription, plan_name), row.email().unwrap_or(b"")),
        c::AccountKind::SignIn if row.plan => (
            w.say(crate::words::Word::WelcomePlanAccount, &[Hole::Name(crate::words::shown(plan_name, w))]),
            words::who(row.email(), row.display.bytes()),
        ),
        c::AccountKind::SignIn => (crate::words::shown(brand, w), words::who(row.email(), row.display.bytes())),
        c::AccountKind::Key => (named(w, W::ApiKeyBrand, brand), b"".as_slice()),
        c::AccountKind::None => return account_heading(root, w, row),
    };
    if !with_display {
        return label;
    }
    let detail = match source.kind {
        c::AccountKind::Program => {
            let signed_in = w.say(crate::words::Word::WelcomeProgramAccount, &[]);
            let separator = words::say(w, W::Separator, &[]);
            words::join(&[&signed_in, if display.is_empty() { b"" } else { &separator }, display])
        }
        c::AccountKind::Key => w.say(crate::words::Word::WelcomeKeyAccount, &[]),
        c::AccountKind::SignIn | c::AccountKind::None => display.into(),
    };
    let separator = words::say(w, W::Separator, &[]);
    words::join(&[&label, if detail.is_empty() { b"" } else { &separator }, &detail])
}

pub(crate) fn playground(root: &Domain, provider: Option<u32>) -> bool {
    match root.forge().playground_provider() {
        Some(number) => provider == Some(number),
        None => false,
    }
}

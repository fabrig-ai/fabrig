//! Folded account rows; every control carries the row that offered it (temper/docs/design/domain/trial.md, 6.2).
use super::{
    confirmation, heading, key_field, named, named_control, omitted, outcome, provider_name, run, state, text,
};
use crate::common::accounts::playground;
use crate::{
    accounts::{
        Bindings, Meaning,
        words::{self, Word as W},
    },
    words::Words,
};
use alloc::boxed::Box;
use jig_client::contract as c;
use skein_lib::List;
use skein_ui::{Builder, Role, Run};
use temper_client_domain::{
    Domain,
    pages::accounts::{self, Act, Operation, Row, SignInState, Way},
};

pub(super) fn own(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    account: &c::AccountLine,
    neighbours: (bool, bool),
    text_bytes: u32,
) {
    draw(
        b,
        root,
        w,
        bindings,
        Row { holder: c::AskAccountHolder::Own, account: account.number },
        account.place,
        neighbours,
        account.provider,
        account.provider_name.as_ref(),
        &account.display,
        account.email(),
        account.holder,
        account.plan,
        account.uses.as_ref(),
        &account.sources,
        account.manage_usage.as_deref(),
        None,
        None,
        account.cut,
        text_bytes,
    );
}
pub(super) fn deployment(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    account: &c::Account,
    neighbours: (bool, bool),
    text_bytes: u32,
) {
    draw(
        b,
        root,
        w,
        bindings,
        Row { holder: c::AskAccountHolder::Deployment, account: account.number },
        account.place,
        neighbours,
        account.provider,
        account.provider_name.as_ref(),
        &account.display,
        account.email(),
        account.holder,
        account.plan,
        account.uses.as_ref(),
        &account.sources,
        account.manage_usage.as_deref(),
        Some(&account.runs),
        Some(&account.waiting),
        account.cut,
        text_bytes,
    );
}
#[expect(
    clippy::too_many_arguments,
    reason = "borrow common fields of the two contract account types without copying documents"
)]
#[expect(clippy::too_many_lines, reason = "one row in document order, with its own sources, facts, acts and outcome")]
fn draw(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    row: Row,
    place: u32,
    neighbours: (bool, bool),
    provider: Option<u32>,
    provider_text: Option<&c::DocumentText>,
    display: &c::DocumentText,
    email: Option<&[u8]>,
    holder: c::AccountHolder,
    plan: bool,
    uses: Option<&List<c::AccountUse>>,
    sources: &List<c::AccountSource>,
    manage: Option<&[u8]>,
    runs: Option<&List<c::Chip>>,
    waiting: Option<&List<c::Chip>>,
    cut: bool,
    text_bytes: u32,
) {
    if playground(root, provider) {
        return;
    }
    let name = provider_name(provider_text);
    let title = heading(
        w,
        row.account,
        provider,
        name,
        display,
        email,
        holder,
        plan,
        sources,
        accounts::current_me(root.client()),
    );
    let deciding = sources.get(0);
    let mut moving = false;
    for source in sources {
        moving |= source.state == c::AccountState::Moving;
    }
    let pays = pays(w, plan, uses);
    let use_line: Box<[u8]> = match row.holder {
        c::AskAccountHolder::Own => Box::new([]),
        c::AskAccountHolder::Deployment => use_line(w, runs, waiting),
    };
    b.open(Role::Disclosure);
    b.style(crate::look::SETTINGS_ROW);
    b.name(&title);
    // Skein includes the description in the native disclosure summary.
    let separator = words::say(w, W::Separator, &[]);
    let summary = words::join(&[
        &summary_state(w, deciding),
        &words::say(w, W::Separator, &[]),
        &pays,
        if use_line.is_empty() { b"" } else { &separator },
        &use_line,
    ]);
    b.description(&summary);
    let Some(page) = root.pages().accounts() else {
        b.close();
        return;
    };
    let busy = page.request.busy(root.client());
    if moving {
        text(b, w, Role::Paragraph, W::Moving);
    } else if let Some(source) = deciding {
        notice(b, root, w, bindings, row, provider, name, source, sources, manage, busy);
    }
    if let Some(trial) = crate::pages::model_check::of_account(root, w, row.account) {
        crate::pages::model_check::line(b, root, w, bindings, &trial, crate::pages::model_check::Place::Row);
    }
    if let Some(uses) = uses {
        b.open(Role::Paragraph);
        b.style(crate::look::ACCOUNT_PAYS);
        let line = pays_names(root, w, uses, bindings.row_uses());
        b.name(&line);
        b.text(Run::Plain, &line);
        b.close();
    }
    crate::cards::text(b, Role::Paragraph, &w.say(crate::words::Word::AccountSources, &[]));
    b.open(Role::List);
    for (index, source) in
        sources.iter().take(usize::try_from(bindings.counts().sources).expect("bounded sources")).enumerate()
    {
        b.open(Role::Item);
        b.open(Role::Text);
        b.run(
            Run::Plain,
            &words::say(
                w,
                W::Number,
                &[skein_ui::phrases::Hole::Number(
                    u64::try_from(index).expect("bounded source").checked_add(1).expect("bounded source"),
                )],
            ),
        );
        b.run(Run::Plain, b". ");
        match source.kind {
            c::AccountKind::SignIn => {
                b.run(Run::Plain, &named(w, W::PlanSignIn, words::provider_words(name.unwrap_or(b"")).1));
            }
            c::AccountKind::Program => {
                b.run(Run::Plain, &named(w, W::CodeLogin, source.name.as_deref().unwrap_or(b"")));
            }
            c::AccountKind::Key | c::AccountKind::None => super::kind(b, w, source.kind),
        }
        b.run(Run::Plain, &words::say(w, W::Separator, &[]));
        b.close();
        tag(b, w, source);
        if source.kind == c::AccountKind::Program {
            program_hosts(b, root, w, source, false, bindings.counts());
        }
        if moving {
            b.close();
            continue;
        }
        match source.kind {
            c::AccountKind::Key => {
                key_for(b, w, bindings, W::Replace, row, Some(source.number), busy);
                button(b, root, w, bindings, W::Remove, row, Operation::RemoveKey { source: source.number }, busy);
            }
            c::AccountKind::SignIn | c::AccountKind::Program => {
                button(
                    b,
                    root,
                    w,
                    bindings,
                    if source.kind == c::AccountKind::Program { W::ProgramSignOut } else { W::SignOut },
                    row,
                    Operation::SignOut { source: source.number },
                    busy,
                );
            }
            c::AccountKind::None => {}
        }
        let position = u32::try_from(index).expect("bounded sources");
        let next = position.checked_add(1).expect("bounded sources");
        ranks(
            b,
            root,
            w,
            bindings,
            row,
            Some(source.number),
            source.place,
            (position != 0, next < sources.len() || cut),
            busy,
        );
        b.close();
    }
    b.close();
    omitted(b, w, W::SourcesOmitted, sources.len(), bindings.counts().sources);
    b.open(Role::Slot);
    if row.holder == c::AskAccountHolder::Deployment {
        pair(b, w, W::UsedBy, &chips(w, runs, waiting));
        pair(b, w, W::WaitingFor, &chips(w, waiting, None));
    }
    b.close();
    if !moving {
        let mut has_key = false;
        for source in sources {
            has_key |= source.kind == c::AccountKind::Key;
        }
        b.open(Role::Slot);
        b.style(crate::look::ACCOUNT_ACTS);
        if !has_key && provider.is_some() {
            key_for(b, w, bindings, W::AddApiKey, row, None, busy);
        }
        usage(b, w, manage);
        ranks(b, root, w, bindings, row, None, place, neighbours, busy);
        button(b, root, w, bindings, W::RemoveAccount, row, Operation::RemoveAccount, busy);
        b.close();
        if let Some((target, source)) = page.key_for
            && target == row
        {
            key_field(b, w, bindings, &page.request, busy);
            let operation = match (source, provider) {
                (Some(source), _) => Some(Operation::ReplaceKey { source }),
                (None, Some(provider)) => Some(Operation::AddKey { provider }),
                (None, None) => None,
            };
            if let Some(operation) = operation {
                button(
                    b,
                    root,
                    w,
                    bindings,
                    if source.is_some() { W::ReplaceKey } else { W::AddKey },
                    row,
                    operation,
                    busy,
                );
            }
        }
    }
    if page.request_row == Some(row) || (page.request.broken.is_some() && editing(page, row)) {
        outcome(b, root, w, bindings, &page.request, text_bytes);
        if let SignInState::Ended { source } = page.request.sign_in_state(root.client())
            && let Some(doc) = accounts::document(root.client())
            && let Some(last) = &doc.last_end
            && last.source == source
            && let Some(provider) = provider
        {
            super::super::connect_model::sign_in_end(b, root, w, Way::SignIn { provider, plan }, &last.end);
            if !moving {
                signin(b, root, w, bindings, row, Some(provider), busy);
            }
        }
        if !moving && let Some((_, ask)) = &page.request.confirm {
            confirmation(b, root, w, page, ask);
            super::control(b, w, bindings, W::Confirm, Meaning::Action(Act::Confirm), busy, false);
        }
        if !moving {
            super::control(b, w, bindings, W::Cancel, Meaning::Action(Act::Cancel), false, false);
        }
    }
    if let Some(doc) = accounts::document(root.client())
        && row.holder == c::AskAccountHolder::Own
    {
        for signing in doc.signing.iter().take(usize::try_from(bindings.counts().sign_ins).expect("bounded sign-ins")) {
            for source in sources {
                if source.number == signing.source {
                    super::fact(b, w, W::Signing, u64::from(signing.source));
                    break;
                }
            }
        }
        if let Some(last) = &doc.last_end
            && page.request_row != Some(row)
        {
            let mut belongs = false;
            for source in sources {
                belongs |= source.number == last.source;
            }
            match last.end {
                c::AccountEnd::Made { account, .. } | c::AccountEnd::Renewed { account, .. } => {
                    belongs = account == row.account;
                }
                c::AccountEnd::Refused { .. } | c::AccountEnd::Failed { .. } => {}
            }
            if belongs {
                super::end(b, w, &last.end, &doc.accounts);
            }
        }
    }
    if !moving && editing(page, row) && page.request_row != Some(row) {
        super::control(b, w, bindings, W::Cancel, Meaning::Action(Act::Cancel), false, false);
    }
    if cut || display.cut {
        text(b, w, Role::Status, W::Cut);
    }
    b.close();
}
fn tag(b: &mut Builder, w: &Words, source: &c::AccountSource) {
    b.open(Role::Status);
    b.style(if source.usable {
        crate::look::STATUS_ACTIVE
    } else if let c::AccountState::Later { .. } = source.state {
        crate::look::STATUS_RETRYING
    } else {
        crate::look::STATUS_HELD
    });
    let label = summary_state(w, Some(source));
    b.name(&label);
    b.open(Role::Text);
    state(b, w, source);
    b.close();
    b.close();
}
fn summary_state(w: &Words, source: Option<&c::AccountSource>) -> Box<[u8]> {
    let word = match source {
        None => W::Unknown,
        Some(source) => {
            if source.spent_until.is_some() && source.state != c::AccountState::Moving {
                W::Limited
            } else {
                match source.state {
                    c::AccountState::Moving => W::Moving,
                    c::AccountState::Granted if source.usable => W::Ready,
                    c::AccountState::Granted => W::Granted,
                    c::AccountState::Later { .. } => W::Limited,
                    c::AccountState::SignInAgain => W::NeedsSignIn,
                    c::AccountState::Misconfigured => W::Misconfigured,
                    c::AccountState::Unloaded { .. } => W::Unloaded,
                    c::AccountState::Asking => W::Asking,
                    c::AccountState::Cancelled => W::Cancelled,
                    c::AccountState::Closing => W::Closing,
                }
            }
        }
    };
    words::say(w, word, &[])
}
fn pair(b: &mut Builder, w: &Words, word: W, value: &[u8]) {
    b.open(Role::Region);
    b.style(crate::look::SETTINGS_SETTING);
    b.name(&words::say(w, word, &[]));
    text(b, w, Role::Term, word);
    crate::cards::text(b, Role::Description, value);
    b.close();
}
fn pays(w: &Words, plan: bool, uses: Option<&List<c::AccountUse>>) -> Box<[u8]> {
    let Some(uses) = uses else { return words::say(w, if plan { W::PlanPays } else { W::OwnPays }, &[]) };
    if uses.is_empty() {
        return words::say(w, W::None, &[]);
    }
    let mut people = 0_u32;
    let mut projects = 0_u32;
    for use_ in uses {
        match use_ {
            c::AccountUse::Party(_) => people = people.checked_add(1).expect("received uses fit"),
            c::AccountUse::Project(_) => projects = projects.checked_add(1).expect("received uses fit"),
        }
    }
    let and = words::say(w, W::And, &[]);
    let mut result = words::join(&[&words::say(w, W::PaysFor, &[]), b" "]);
    if people != 0 {
        result = words::join(&[&result, &count(w, people, W::PersonCount, W::PeopleCount)]);
    }
    if projects != 0 {
        result = words::join(&[
            &result,
            if people == 0 { b"" } else { &and },
            &count(w, projects, W::ProjectCount, W::ProjectsCount),
        ]);
    }
    result
}
fn count(w: &Words, number: u32, one: W, many: W) -> Box<[u8]> {
    words::join(&[
        &words::say(w, W::Number, &[skein_ui::phrases::Hole::Number(u64::from(number))]),
        &words::say(w, if number == 1 { one } else { many }, &[]),
    ])
}
fn pays_names(root: &Domain, w: &Words, uses: &List<c::AccountUse>, cap: u32) -> Box<[u8]> {
    if uses.is_empty() {
        return words::join(&[&words::say(w, W::PaysFor, &[]), b" ", &words::say(w, W::None, &[]), b"."]);
    }
    let and = words::say(w, W::And, &[]);
    let mut result = words::join(&[&words::say(w, W::PaysFor, &[]), b" "]);
    let shown = uses.len().min(cap);
    for (index, use_) in uses.iter().take(usize::try_from(shown).expect("bounded uses")).enumerate() {
        let name = match use_ {
            c::AccountUse::Party(party) => match party_name(root, *party) {
                Some(name) => crate::words::shown(name, w),
                None => words::join(&[
                    &words::say(w, W::Party, &[]),
                    &words::say(w, W::Number, &[skein_ui::phrases::Hole::Number(*party)]),
                ]),
            },
            c::AccountUse::Project(project) => {
                match crate::common::project_name(accounts::current_me(root.client()), *project) {
                    Some(name) => crate::words::shown(&name, w),
                    None => words::join(&[
                        &words::say(w, W::Project, &[]),
                        &words::say(w, W::Number, &[skein_ui::phrases::Hole::Number(u64::from(*project))]),
                    ]),
                }
            }
        };
        let last = u32::try_from(index).expect("bounded uses").checked_add(1).expect("bounded uses") == uses.len();
        result = words::join(&[
            &result,
            if index == 0 {
                b""
            } else if last {
                &and
            } else {
                b", "
            },
            &name,
        ]);
    }
    if uses.len() > shown {
        result = words::join(&[
            &result,
            if shown == 0 { b"" } else { &and },
            &count(w, uses.len().saturating_sub(shown), W::More, W::More),
        ]);
    }
    words::join(&[&result, b"."])
}
fn party_name(root: &Domain, party: u64) -> Option<&[u8]> {
    root.pages().accounts()?.party_name(party)
}
fn chips(w: &Words, rows: Option<&List<c::Chip>>, excluded: Option<&List<c::Chip>>) -> Box<[u8]> {
    let mut result = words::say(w, W::None, &[]);
    let mut first = true;
    if let Some(rows) = rows {
        for row in rows {
            let mut skip = false;
            if let Some(excluded) = excluded {
                for waiting in excluded {
                    skip |= waiting.task == row.task;
                }
            }
            if skip {
                continue;
            }
            let task = words::say(w, W::Number, &[skein_ui::phrases::Hole::Number(row.task)]);
            result = words::join(&[if first { b"" } else { &result }, if first { b"T" } else { b", T" }, &task]);
            first = false;
        }
    }
    result
}
fn use_line(w: &Words, runs: Option<&List<c::Chip>>, waiting: Option<&List<c::Chip>>) -> Box<[u8]> {
    if let Some(waiting) = waiting
        && !waiting.is_empty()
    {
        return words::join(&[
            &words::say(w, W::Number, &[skein_ui::phrases::Hole::Number(u64::from(waiting.len()))]),
            b" tasks waiting",
        ]);
    }
    if let Some(runs) = runs
        && !runs.is_empty()
    {
        return words::join(&[b"used by ", &chips(w, Some(runs), waiting)]);
    }
    words::say(w, W::NoUsers, &[])
}
#[expect(clippy::too_many_arguments, reason = "each bound control names its row and operation")]
fn button(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    word: W,
    row: Row,
    operation: Operation,
    busy: bool,
) {
    let Some(binding) =
        bindings.insert(Meaning::Action(Act::Submit { name: root.pages().account_name(), row, operation }))
    else {
        return;
    };
    let name = words::say(w, word, &[]);
    b.open(Role::Button);
    b.name(&name);
    b.bind(binding);
    if word == W::RemoveAccount {
        b.style(crate::look::ACCOUNT_REMOVE);
    }
    if word == W::UpAccount || word == W::UpSource || word == W::DownAccount || word == W::DownSource {
        b.hint(&name);
        b.style(crate::look::ACCOUNT_MOVE);
        b.style(if word == W::UpAccount || word == W::UpSource {
            crate::look::ACCOUNT_UP
        } else {
            crate::look::ACCOUNT_DOWN
        });
    }
    if busy {
        b.state(skein_ui::State::Disabled);
    }
    b.close();
}
fn key_for(b: &mut Builder, w: &Words, bindings: &mut Bindings, word: W, row: Row, source: Option<u32>, busy: bool) {
    super::control(b, w, bindings, word, Meaning::Action(Act::KeyFor { row, source }), busy, false);
}
fn signin(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    row: Row,
    provider: Option<u32>,
    busy: bool,
) {
    if row.holder == c::AskAccountHolder::Deployment {
        return;
    }
    let Some(provider) = provider else { return };
    let mut plan = b"".as_slice();
    if let Some(doc) = accounts::document(root.client()) {
        for account in &doc.accounts {
            if account.number == row.account
                && let Some(name) = provider_name(account.provider_name.as_ref())
            {
                plan = words::provider_words(name).1;
            }
        }
    }
    if let Some(me) = accounts::current_me(root.client()) {
        for offer in &me.providers {
            if offer.provider == provider {
                plan = words::provider_words(offer.name.bytes()).1;
            }
        }
    }
    named_control(
        b,
        bindings,
        &named(w, W::Continue, plan),
        Meaning::Action(Act::Submit {
            name: root.pages().account_name(),
            row,
            operation: Operation::SignIn { provider },
        }),
        busy,
        false,
    );
}
fn usage(b: &mut Builder, w: &Words, address: Option<&[u8]>) {
    if let Some(address) = address
        && c::valid_manage_usage(address)
        && skein_ui::markdown::safe_url(address)
    {
        b.open(Role::Link);
        b.name(&w.say(crate::words::Word::Usage, &[]));
        b.text(Run::Plain, &w.say(crate::words::Word::Usage, &[]));
        b.external(address);
        b.close();
    }
}
#[expect(clippy::too_many_arguments, reason = "the state's acts borrow their deciding source and row")]
#[expect(clippy::too_many_lines, reason = "each deciding source's notice keeps its words and renewal acts together")]
fn notice(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    row: Row,
    provider: Option<u32>,
    provider_text: Option<&[u8]>,
    source: &c::AccountSource,
    sources: &List<c::AccountSource>,
    manage: Option<&[u8]>,
    busy: bool,
) {
    if source.expiring && source.state != c::AccountState::SignInAgain {
        match source.kind {
            c::AccountKind::SignIn => {
                text(b, w, Role::Paragraph, W::ExpiringPlan);
                signin(b, root, w, bindings, row, provider, busy);
                return;
            }
            c::AccountKind::Program => {
                text(b, w, Role::Paragraph, W::ExpiringPlan);
                program_hosts(b, root, w, source, true, bindings.counts());
                crate::cards::text(
                    b,
                    Role::Code,
                    &super::super::connect_model::command_line(root, w, source.name.as_deref().unwrap_or(b"")),
                );
                button(
                    b,
                    root,
                    w,
                    bindings,
                    W::CheckAgain,
                    row,
                    Operation::CheckProgram { source: source.number },
                    busy,
                );
                return;
            }
            c::AccountKind::Key | c::AccountKind::None => {}
        }
    }
    let mut declined = false;
    if row.holder == c::AskAccountHolder::Own
        && let Some(doc) = accounts::document(root.client())
        && let Some(last) = &doc.last_end
    {
        match last.end {
            c::AccountEnd::Made { account, declined: yes } | c::AccountEnd::Renewed { account, declined: yes } => {
                declined = yes && account == row.account;
            }
            c::AccountEnd::Refused { .. } | c::AccountEnd::Failed { .. } => {}
        }
    }
    if declined {
        text(b, w, Role::Alert, W::DeclinedPlan);
        signin(b, root, w, bindings, row, provider, busy);
        key_for(b, w, bindings, W::KeyWayOnly, row, None, busy);
        return;
    }
    let limited = if let c::AccountState::Later { .. } = source.state { true } else { source.spent_until.is_some() };
    if limited {
        let reset = if let c::AccountState::Later { until } = source.state { until } else { None };
        if let Some(reset) = reset {
            b.open(Role::Paragraph);
            b.open(Role::Text);
            b.run(Run::Plain, &words::say(w, W::LimitedBy, &[]));
            let mut brand = words::provider_words(provider_text.unwrap_or(b"")).0;
            if let Some(me) = accounts::current_me(root.client()) {
                for offer in &me.providers {
                    if Some(offer.provider) == provider {
                        brand = words::provider_words(offer.name.bytes()).0;
                    }
                }
            }
            b.run(Run::Plain, brand);
            b.run(Run::Plain, &words::say(w, W::ResetUntil, &[skein_ui::phrases::Hole::Wall(reset)]));
            b.close();
            b.close();
        } else {
            text(
                b,
                w,
                Role::Paragraph,
                if source.kind == c::AccountKind::SignIn { W::PlanLimited } else { W::Limited },
            );
            usage(b, w, manage);
        }
        button(b, root, w, bindings, W::Resume, row, Operation::Resume { source: source.number }, busy);
        for next in sources {
            if next.place > source.place && next.kind == c::AccountKind::Key {
                button(
                    b,
                    root,
                    w,
                    bindings,
                    W::KeyFirst,
                    row,
                    Operation::MoveSource { source: next.number, place: source.place },
                    busy,
                );
                break;
            }
        }
    } else if source.kind == c::AccountKind::Program {
        if !source.usable {
            b.open(Role::Paragraph);
            b.open(Role::Text);
            state(b, w, source);
            b.close();
            b.close();
        }
        let needs_check = program_hosts(b, root, w, source, true, bindings.counts());
        if needs_check || source.state == c::AccountState::SignInAgain {
            button(b, root, w, bindings, W::CheckAgain, row, Operation::CheckProgram { source: source.number }, busy);
        }
    } else {
        match source.state {
            c::AccountState::SignInAgain => {
                text(b, w, Role::Paragraph, W::SignedOutPlan);
                signin(b, root, w, bindings, row, provider, busy);
            }
            c::AccountState::Misconfigured => text(b, w, Role::Paragraph, W::MisconfiguredPlan),
            c::AccountState::Granted if source.usable => {}
            c::AccountState::Granted => text(b, w, Role::Paragraph, W::Granted),
            c::AccountState::Moving
            | c::AccountState::Asking
            | c::AccountState::Later { .. }
            | c::AccountState::Unloaded { .. }
            | c::AccountState::Cancelled
            | c::AccountState::Closing => {
                b.open(Role::Paragraph);
                b.open(Role::Text);
                state(b, w, source);
                b.close();
                b.close();
            }
        }
    }
}
fn program_hosts(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    source: &c::AccountSource,
    notice: bool,
    counts: jig_client::AccountCounts,
) -> bool {
    let Some(doc) = accounts::document(root.client()) else { return false };
    let mut needs_check = false;
    for host in doc.program_hosts.iter().take(usize::try_from(counts.accounts).expect("bounded hosts")) {
        for login in host.programs.iter().take(usize::try_from(counts.sources).expect("bounded programs")) {
            if Some(login.name.as_ref()) != source.name.as_deref() {
                continue;
            }
            if notice && !needs_program(login.state.as_ref()) {
                continue;
            }
            b.open(Role::Paragraph);
            b.name(&host.name);
            b.open(Role::Text);
            if notice && login.state == Some(c::ProgramState::NotSignedIn) {
                needs_check = true;
                b.run(Run::Plain, &named(w, W::ProgramNeedsSignIn, &login.name));
            }
            if program_failed(login.state.as_ref()) {
                run(b, w, W::UnusableOn);
            }
            b.run(Run::Plain, &host.name);
            match &login.state {
                Some(c::ProgramState::Failed { reason, detail }) => {
                    b.run(Run::Plain, b": ");
                    super::super::account_programs::failure(b, w, *reason, detail, w.name_bytes);
                }
                Some(c::ProgramState::SignedInOtherwise { method }) => {
                    b.run(Run::Plain, b" ");
                    run(b, w, W::SignedInOtherwise);
                    b.run(Run::Plain, method);
                }
                Some(c::ProgramState::Usable) => {
                    b.run(Run::Plain, b" ");
                    run(b, w, W::Ready);
                }
                None => {
                    b.run(Run::Plain, b" ");
                    run(b, w, W::Unknown);
                }
                Some(c::ProgramState::NotSignedIn) => {}
            }
            if !host.connected {
                run(b, w, W::NotInContact);
            }
            if host.awaiting_logins {
                run(b, w, W::AwaitingLogins);
            }
            if host.cut || login.cut {
                run(b, w, W::Cut);
            }
            if login.cleanup {
                run(b, w, W::Cleanup);
            }
            if notice && login.state == Some(c::ProgramState::NotSignedIn) {
                b.run(Run::Plain, b".");
            }
            if let Some(sign_out) = &login.sign_out {
                match sign_out {
                    c::ProgramSignOut::SignedOut => run(b, w, W::SignedOut),
                    c::ProgramSignOut::Failed { reason, detail } => {
                        super::super::account_programs::failure(b, w, *reason, detail, w.name_bytes);
                    }
                }
            }
            b.close();
            b.close();
            if notice
                && login.state == Some(c::ProgramState::NotSignedIn)
                && !(source.expiring && source.state != c::AccountState::SignInAgain)
            {
                crate::cards::text(b, Role::Code, &super::super::connect_model::command_line(root, w, &login.name));
            }
            break;
        }
    }
    needs_check
}
#[expect(clippy::too_many_arguments, reason = "rank controls share the current row and source positions")]
fn ranks(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    row: Row,
    source: Option<u32>,
    place: u32,
    neighbours: (bool, bool),
    busy: bool,
) {
    for (available, target, word) in [
        (neighbours.0, place.checked_sub(1), if source.is_some() { W::UpSource } else { W::UpAccount }),
        (neighbours.1, place.checked_add(1), if source.is_some() { W::DownSource } else { W::DownAccount }),
    ] {
        if available && let Some(place) = target {
            let op = match source {
                Some(source) => Operation::MoveSource { source, place },
                None => Operation::MoveAccount { place },
            };
            button(b, root, w, bindings, word, row, op, busy);
        }
    }
}

fn editing(page: &accounts::Accounts, row: Row) -> bool {
    match page.key_for {
        Some((target, _)) => target == row,
        None => false,
    }
}
fn needs_program(state: Option<&c::ProgramState>) -> bool {
    match state {
        Some(c::ProgramState::NotSignedIn | c::ProgramState::Failed { .. }) => true,
        None | Some(c::ProgramState::Usable | c::ProgramState::SignedInOtherwise { .. }) => false,
    }
}
fn program_failed(state: Option<&c::ProgramState>) -> bool {
    match state {
        Some(c::ProgramState::Failed { .. }) => true,
        Some(c::ProgramState::Usable | c::ProgramState::NotSignedIn | c::ProgramState::SignedInOtherwise { .. })
        | None => false,
    }
}

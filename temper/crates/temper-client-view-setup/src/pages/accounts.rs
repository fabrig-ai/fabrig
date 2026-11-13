//! Supplied accounts and the playground model (temper/docs/design/client/pages/README.md and temper/docs/design/client/forge/family.md, 7 and temper/docs/design/client/pages/accounts.md, 2).
use super::settings;
pub(crate) use crate::common::accounts::{
    first_usable, first_usable_kind, heading, named, provider_name, run, unloaded,
};
use crate::look;
use crate::{
    accounts::{
        Bindings, Meaning,
        words::{self, Word as W},
    },
    words::Words,
};
use alloc::boxed::Box;
mod rows;
use jig_client::contract as c;
use skein_ui::{Builder, Role, Run, State, phrases::Hole};
use temper_client_domain::{
    Domain,
    pages::accounts::{self, Broken},
};
pub(super) fn number(b: &mut Builder, w: &Words, value: u64) {
    b.run(Run::Plain, &words::say(w, W::Number, &[Hole::Number(value)]));
}
fn text(b: &mut Builder, w: &Words, role: Role, word: W) {
    crate::cards::text(b, role, &words::say(w, word, &[]));
}
fn done(b: &mut Builder, w: &Words, word: W) {
    let value = words::say(w, word, &[]);
    b.open(Role::Status);
    b.style(look::STATUS_DONE);
    b.name(&value);
    b.text(Run::Plain, &value);
    b.close();
}
fn fact(b: &mut Builder, w: &Words, word: W, value: u64) {
    b.open(Role::Paragraph);
    b.open(Role::Text);
    run(b, w, word);
    number(b, w, value);
    b.close();
    b.close();
}
fn control(
    b: &mut Builder,
    w: &Words,
    bindings: &mut Bindings,
    word: W,
    meaning: Meaning,
    disabled: bool,
    pressed: bool,
) {
    named_control(b, bindings, &words::say(w, word, &[]), meaning, disabled, pressed);
}
fn named_control(
    b: &mut Builder,
    bindings: &mut Bindings,
    name: &[u8],
    meaning: Meaning,
    disabled: bool,
    pressed: bool,
) {
    let model = match &meaning {
        Meaning::Model(_) => true,
        Meaning::Try { .. }
        | Meaning::Copy { .. }
        | Meaning::Action(_)
        | Meaning::Go(_)
        | Meaning::Key
        | Meaning::SignInAddress
        | Meaning::Program => false,
    };
    let Some(binding) = bindings.insert(meaning) else { return };
    b.open(Role::Button);
    if model {
        b.style(look::FLOW_SECONDARY);
    }
    b.name(name);
    b.bind(binding);
    if pressed {
        b.state(State::Pressed);
    }
    if disabled {
        b.state(State::Disabled);
    }
    b.close();
}
fn kind(b: &mut Builder, w: &Words, value: c::AccountKind) {
    run(
        b,
        w,
        match value {
            c::AccountKind::SignIn => W::SignIn,
            c::AccountKind::Key => W::Key,
            c::AccountKind::Program => W::Program,
            c::AccountKind::None => W::None,
        },
    );
}
fn state(b: &mut Builder, w: &Words, source: &c::AccountSource) {
    match source.state {
        c::AccountState::Moving => run(b, w, W::Moving),
        c::AccountState::Later { until } => {
            run(b, w, W::Limited);
            if let Some(at) = until {
                b.run(Run::Plain, &words::say(w, W::Until, &[Hole::Wall(at)]));
            }
        }
        c::AccountState::Unloaded { at, why } => unloaded(b, w, at, why),
        c::AccountState::Asking => run(b, w, W::Asking),
        c::AccountState::Granted => run(b, w, if source.usable { W::Ready } else { W::Granted }),
        c::AccountState::SignInAgain => run(b, w, W::NeedsSignIn),
        c::AccountState::Misconfigured => run(b, w, W::Misconfigured),
        c::AccountState::Cancelled => run(b, w, W::Cancelled),
        c::AccountState::Closing => run(b, w, W::Closing),
    }
    if source.expiring {
        b.run(Run::Plain, b" \xc2\xb7 ");
        run(b, w, W::Expiring);
    }
    if source.attention {
        b.run(Run::Plain, b" \xc2\xb7 ");
        run(b, w, W::Attention);
    }
    if let Some(at) = source.spent_until {
        b.run(Run::Plain, b" \xc2\xb7 ");
        run(b, w, W::Spent);
        b.run(Run::Plain, b" \xc2\xb7 ");
        b.run(Run::Plain, &words::say(w, W::UsageUntil, &[Hole::Wall(at)]));
    }
}
pub(crate) fn end(b: &mut Builder, w: &Words, end: &c::AccountEnd, rows: &skein_lib::List<c::AccountLine>) {
    b.open(Role::Status);
    b.open(Role::Text);
    match end {
        c::AccountEnd::Made { account, declined } | c::AccountEnd::Renewed { account, declined } => {
            let (word, fallback) = if let c::AccountEnd::Renewed { .. } = end {
                (W::RenewedNamed, W::Renewed)
            } else {
                (W::MadeNamed, W::Made)
            };
            if !named_end(b, w, rows, *account, word) {
                run(b, w, fallback);
                number(b, w, u64::from(*account));
            }
            if *declined {
                run(b, w, W::Declined);
            }
        }
        c::AccountEnd::Refused { refusal } => refused(b, w, *refusal),
        c::AccountEnd::Failed { failure } => match failure {
            c::AccountFailure::Later { after } => {
                run(b, w, W::Later);
                if let Some(after) = after {
                    b.run(Run::Plain, &words::say(w, W::After, &[Hole::Span(*after)]));
                }
            }
            c::AccountFailure::SignInAgain => run(b, w, W::NeedsSignIn),
            c::AccountFailure::MissingScope { scope } => {
                run(b, w, W::Scope);
                number(b, w, u64::from(*scope));
            }
            c::AccountFailure::Misconfigured => run(b, w, W::Misconfigured),
            c::AccountFailure::Unloaded { at, why } => unloaded(b, w, *at, *why),
            c::AccountFailure::Cancelled => run(b, w, W::Cancelled),
        },
    }
    b.close();
    b.close();
}
fn named_end(b: &mut Builder, w: &Words, rows: &skein_lib::List<c::AccountLine>, account: u32, word: W) -> bool {
    for row in rows {
        if row.number != account {
            continue;
        }
        let Some(name) = provider_name(row.provider_name.as_ref()) else { return false };
        let (_, plan) = words::provider_words(name);
        let display = words::who(row.email(), row.display.bytes());
        if display.is_empty() {
            if word == W::RenewedNamed {
                return false;
            }
            b.run(Run::Plain, &words::say(w, W::MadePlan, &[Hole::Name(skein_lib::bytes::copy_of(plan))]));
            return true;
        }
        let holes = [
            Hole::Name(skein_lib::bytes::copy_of(plan)),
            Hole::Name(skein_lib::bytes::copy_of(display)),
            Hole::Name(skein_lib::bytes::copy_of(plan)),
        ];
        let holes = if word == W::RenewedNamed { &holes } else { holes.get(..2).expect("made has two holes") };
        b.run(Run::Plain, &words::say(w, word, holes));
        return true;
    }
    false
}
pub(super) fn refused(b: &mut Builder, w: &Words, refusal: c::AccountRefusal) {
    use c::AccountRefusal as R;
    let word = match refusal {
        R::NotSignedIn => W::NotSignedIn,
        R::NotOurs => W::NotOurs,
        R::Cancelled => W::MoveCancelled,
        R::Moving => W::Moving,
        R::NotLimited => W::NotLimited,
        R::Uses => W::UsesRefused,
        R::ProgramNameReserved => {
            crate::cards::refusal(b, w, &c::Refusal::ProgramNameReserved);
            return;
        }
        R::NotKept { why } => match why {
            c::AccountKeyFailure::TooLong => W::KeyLong,
            c::AccountKeyFailure::NotStored => W::NotStored,
        },
        R::Limit { bound } => match bound {
            c::AccountBound::Accounts => W::LimitAccounts,
            c::AccountBound::HolderAccounts => W::LimitHolder,
            c::AccountBound::Sources => W::LimitSources,
            c::AccountBound::SignIns => W::LimitSignIns,
        },
        R::Held { provider } | R::Provider { provider } => {
            run(b, w, if let R::Held { .. } = refusal { W::Held } else { W::ProviderRefused });
            number(b, w, u64::from(provider));
            return;
        }
        R::Subject => W::Subject,
        R::Unknown => W::UnknownNumber,
        R::Numbers => W::Numbers,
        R::Kind { kind: value } => {
            run(b, w, W::Kind);
            kind(b, w, value);
            return;
        }
        R::Closing => W::Closing,
        R::NotStarted => W::NotStarted,
    };
    run(b, w, word);
}
pub(crate) fn main(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    pages: &mut crate::bindings::Bindings,
    project: Option<u32>,
    text_per_node: u32,
) {
    b.open(Role::Main);
    b.style(look::MAIN);
    b.style(look::SETTINGS);
    match project {
        Some(project) => settings::open(b, root, w, pages, project, settings::Tab::Accounts),
        None => settings::open_alone(
            b,
            &words::say(w, W::Accounts, &[]),
            &w.say(crate::words::Word::SettingsAccountsLede, &[]),
        ),
    }
    let doc = accounts::document(root.client());
    let mut count = 0_u64;
    if let Some(doc) = doc {
        for row in &doc.accounts {
            if !crate::common::accounts::playground(root, row.provider) {
                count = count.checked_add(1).expect("received accounts fit");
            }
        }
    }
    let title = match doc {
        Some(_) => words::say(w, W::OwnCount, &[Hole::Number(count)]),
        None => words::say(w, W::Own, &[]),
    };
    settings::section_head(b, &title);
    b.style(look::ACCOUNTS_PAGE);
    if doc.is_some() && count != 0 && root.forge().playground().is_none() {
        connect(b, w, bindings, W::ConnectAdd);
    }
    b.close();
    added(b, root, w, bindings);
    if let Some(doc) = doc {
        if count == 0 {
            b.open(Role::Slot);
            b.style(look::SETTINGS_LINES);
            b.open(Role::Slot);
            b.style(look::CONNECT_EMPTY);
            text(b, w, Role::Paragraph, W::NoOwn);
            if root.forge().playground().is_none() {
                connect(b, w, bindings, W::Connect);
            }
            b.close();
            b.close();
        } else {
            b.open(Role::Slot);
            b.style(look::SETTINGS_LINES);
            let mut shown = 0;
            for row in &doc.accounts {
                if crate::common::accounts::playground(root, row.provider) || shown >= bindings.counts().accounts {
                    continue;
                }
                rows::own(b, root, w, bindings, row, text_per_node);
                shown = shown.checked_add(1).expect("displayed rows fit");
            }
            b.close();
        }
        omitted(b, w, W::AccountsOmitted, u32::try_from(count).expect("received rows fit"), bindings.counts().accounts);
        signings_without_rows(b, root, w, doc, bindings.counts());
        if doc.cut {
            text(b, w, Role::Status, W::Cut);
        }
    }
    b.close();
    if let Some(page) = root.pages().accounts()
        && is_owner(root)
        && let Some(doc) = page.deployment(root.client())
        && !doc.accounts.is_empty()
    {
        settings::section_head(b, &words::say(w, W::Deployment, &[]));
        b.close();
        b.open(Role::Slot);
        b.style(look::SETTINGS_LINES);
        for row in doc.accounts.iter().take(usize::try_from(bindings.counts().accounts).expect("u32 fits")) {
            rows::deployment(b, root, w, bindings, row, text_per_node);
        }
        b.close();
        omitted(b, w, W::AccountsOmitted, doc.accounts.len(), bindings.counts().accounts);
        if doc.cut {
            text(b, w, Role::Status, W::Cut);
        }
        b.close();
    }
    if root.forge().playground().is_some() {
        settings::section_head(b, &words::say(w, W::Playground, &[]));
        b.close();
        b.open(Role::Slot);
        b.style(look::SETTINGS_LINES);
        b.open(Role::Disclosure);
        b.style(look::SETTINGS_ROW);
        b.name(&words::say(w, W::PlaygroundAccount, &[]));
        text(b, w, Role::Paragraph, W::PlaygroundWords);
        b.close();
        b.close();
        b.close();
    }
    if project.is_some() {
        settings::sign_out(b, root, w, pages);
    }
    b.close();
    b.close();
}
fn added(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings) {
    let Some(page) = root.pages().accounts() else { return };
    let Some(added) = page.added else { return };
    let Some(me) = accounts::current_me(root.client()) else { return };
    for offer in &me.providers {
        if offer.provider != added.provider {
            continue;
        }
        let (brand, plan) = words::provider_words(offer.name.bytes());
        let (word, name) = match added.kind {
            temper_client_domain::WayKind::Key => (W::KeyAdded, brand),
            temper_client_domain::WayKind::Program => (W::ProgramAdded, plan),
            temper_client_domain::WayKind::SignIn => {
                (if offer.signed_in_plan { W::PlanAdded } else { W::AccountAdded }, plan)
            }
        };
        b.open(Role::Paragraph);
        b.style(look::CONNECT_CONFIRM);
        b.text(Run::Plain, &named(w, word, name));
        if let Some(to) = bindings.insert(Meaning::Go(temper_client_domain::Address::NewChat { draft: false })) {
            b.open(Role::Link);
            b.style(look::CONNECT_QUIET);
            b.name(&words::say(w, W::StartChat, &[]));
            b.text(Run::Plain, &words::say(w, W::StartChat, &[]));
            b.link(to);
            b.close();
        }
        b.close();
        break;
    }
}
fn signings_without_rows(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    doc: &c::Accounts,
    counts: jig_client::AccountCounts,
) {
    for signing in doc.signing.iter().take(usize::try_from(counts.sign_ins).expect("bounded sign-ins")) {
        let mut in_row = false;
        let mut shown = 0_u32;
        for row in &doc.accounts {
            if crate::common::accounts::playground(root, row.provider) || shown >= counts.accounts {
                continue;
            }
            shown = shown.checked_add(1).expect("displayed rows fit");
            for source in &row.sources {
                in_row |= source.number == signing.source;
            }
        }
        if in_row {
            continue;
        }
        b.open(Role::Status);
        b.open(Role::Text);
        run(b, w, W::Signing);
        number(b, w, u64::from(signing.source));
        b.run(Run::Plain, b" \xc2\xb7 ");
        run(b, w, W::Provider);
        number(b, w, u64::from(signing.provider));
        b.run(Run::Plain, b" \xc2\xb7 ");
        match signing.holder {
            c::AccountHolder::Deployment => run(b, w, W::Deployment),
            c::AccountHolder::Party(party) => {
                run(b, w, W::Party);
                number(b, w, party);
            }
        }
        b.close();
        b.close();
    }
    omitted(b, w, W::SignInsOmitted, doc.signing.len(), counts.sign_ins);
}
fn connect(b: &mut Builder, w: &Words, bindings: &mut Bindings, word: W) {
    if let Some(to) = bindings.insert(Meaning::Go(temper_client_domain::Address::ConnectModel { way: None })) {
        let name = words::say(w, word, &[]);
        if word == W::Connect {
            b.open(Role::Link);
            b.style(look::CONNECT_PRIMARY);
            b.name(&name);
            b.link(to);
            b.text(Run::Plain, &name);
            b.close();
        } else {
            settings::button_to(b, &name, to, false);
        }
    }
}
pub(super) fn key_field(
    b: &mut Builder,
    w: &Words,
    bindings: &mut Bindings,
    form: &accounts::request::Request,
    busy: bool,
) {
    let Some(binding) = bindings.insert(Meaning::Key) else { return };
    b.open(Role::TextField);
    b.name(&words::say(w, W::KeyField, &[]));
    b.entry(skein_ui::EntryKind::Secret);
    b.value(b"", form.key_written());
    b.bind(binding);
    if busy {
        b.state(State::Disabled);
    }
    b.close();
}
#[expect(clippy::too_many_lines, reason = "the account act's outcomes stay in one renderer")]
pub(super) fn outcome(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    form: &accounts::request::Request,
    text_per_node: u32,
) {
    if let Some(broken) = form.broken {
        text(
            b,
            w,
            Role::Alert,
            match broken {
                Broken::Empty => W::KeyEmpty,
                Broken::TooLong => W::KeyLong,
                Broken::Selection => W::Selection,
                Broken::Rank => W::Rank,
                Broken::ValueAgain => W::ValueAgain,
                Broken::ProgramName => W::ProgramInvalid,
            },
        );
    }
    if let Some(pending) = &form.pending {
        match &pending.outcome {
            None => {
                let progress = match root.client().pending(pending.key) {
                    Some(p) => p.progress,
                    None => jig_client::Progress::InFlight,
                };
                crate::cards::progress(b, w, progress);
            }
            Some(jig_client::Outcome::Full { held, room_at }) => crate::cards::answers_full(b, w, *held, *room_at),
            Some(jig_client::Outcome::Lost) => text(b, w, Role::Alert, W::Lost),
            Some(jig_client::Outcome::Unsent) => text(b, w, Role::Alert, W::Unsent),
            Some(jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(
                c::DocumentOutcome::AccountSignedOut { confirmed, failed_hosts },
            ))))) => {
                if *confirmed {
                    done(b, w, W::Confirmed);
                } else {
                    text(b, w, Role::Status, W::Unconfirmed);
                }
                super::account_programs::failures(b, w, failed_hosts, text_per_node);
            }
            Some(jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(
                c::DocumentOutcome::AccountRemoved { confirmed, failed_hosts },
            ))))) => {
                if *confirmed {
                    done(b, w, W::Removed);
                } else {
                    text(b, w, Role::Status, W::RemovedUnconfirmed);
                }
                super::account_programs::failures(b, w, failed_hosts, text_per_node);
            }
            Some(jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(
                c::DocumentOutcome::AccountKeyKept { source },
            ))))) => fact(b, w, W::Saved, u64::from(*source)),
            Some(jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(
                c::DocumentOutcome::AccountProgramDeclared { source, name },
            ))))) => {
                fact(b, w, W::Source, u64::from(*source));
                crate::cards::text(b, Role::Paragraph, &super::connect_model::command_line(root, w, name));
            }
            Some(jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused { refusal, .. }))) => {
                if let Some((word, name)) = act_refusal(&pending.ask, refusal) {
                    let hole = match name {
                        Some(name) => Some(Hole::Name(skein_lib::bytes::copy_of(name))),
                        None => None,
                    };
                    let value = words::say(w, word, hole.as_slice());
                    b.open(Role::Status);
                    b.style(look::REFUSAL);
                    b.name(&value);
                    b.text(Run::Plain, &value);
                    b.close();
                } else {
                    crate::cards::refusal(b, w, refusal);
                }
            }
            Some(jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(outcome)))) => {
                crate::cards::outcome(b, w, outcome);
            }
            Some(
                jig_client::Outcome::Answered(jig_client::Answered::Family(_))
                | jig_client::Outcome::NotKept
                | jig_client::Outcome::Held
                | jig_client::Outcome::Unknown
                | jig_client::Outcome::Limit
                | jig_client::Outcome::Unconfirmed,
            ) => text(b, w, Role::Status, W::Unknown),
        }
    }
    if let Some(address) = form.sign_in_address(root.client()) {
        text(b, w, Role::Paragraph, W::Address);
        if skein_ui::markdown::safe_url(address) {
            let label = words::say(w, W::OpenSignIn, &[]);
            let bytes = skein_lib::len_u64(label.len()).saturating_add(skein_lib::len_u64(address.len()));
            if bytes <= u64::from(text_per_node) {
                b.open(Role::Link);
                b.text(Run::Plain, &label);
                b.external(address);
                b.close();
            } else {
                b.open(Role::Alert);
                b.text(Run::Plain, &words::say(w, W::AddressLong, &[Hole::Number(skein_lib::len_u64(address.len()))]));
                b.close();
            }
        } else {
            text(b, w, Role::Alert, W::UnsafeAddress);
        }
        paste(b, root, w, bindings, form, true);
    } else if form.paste_state(root.client()) == accounts::PasteState::Ended {
        paste(b, root, w, bindings, form, false);
    } else if form.returned() && form.paste_state(root.client()) != accounts::PasteState::Closed {
        // A departed sign-in's address come back to by hand while it still waits (client/pages/accounts.md, 2).
        paste(b, root, w, bindings, form, false);
    }
}

pub(super) fn act_refusal<'a>(ask: &'a c::Ask, refusal: &c::Refusal) -> Option<(W, Option<&'a [u8]>)> {
    let change = match ask {
        c::Ask::AccountChange { change, .. } => change,
        c::Ask::ReadLists { .. }
        | c::Ask::Restore { .. }
        | c::Ask::ChangeSettings(_)
        | c::Ask::ChangeModel(_)
        | c::Ask::MoveToNewest(_)
        | c::Ask::Create { .. }
        | c::Ask::Propose { .. }
        | c::Ask::Withdraw { .. }
        | c::Ask::Talk { .. }
        | c::Ask::Leave { .. }
        | c::Ask::Message { .. }
        | c::Ask::Compound { .. }
        | c::Ask::Answer { .. }
        | c::Ask::Take { .. }
        | c::Ask::HandBack { .. }
        | c::Ask::Decide { .. }
        | c::Ask::Release { .. }
        | c::Ask::Lift { .. }
        | c::Ask::Amend { .. }
        | c::Ask::Stop { .. }
        | c::Ask::Cancel { .. }
        | c::Ask::Move { .. }
        | c::Ask::Seen { .. }
        | c::Ask::EditNote { .. }
        | c::Ask::DeleteNote { .. }
        | c::Ask::ChangePolicy { .. }
        | c::Ask::SetRoles { .. }
        | c::Ask::MakeService { .. }
        | c::Ask::Rename { .. }
        | c::Ask::RemoveProject { .. }
        | c::Ask::MakeProject { .. }
        | c::Ask::SetPool { .. }
        | c::Ask::SignIn { .. }
        | c::Ask::SignOut { .. }
        | c::Ask::SetVisibility { .. } => return None,
    };
    match change {
        c::AccountChange::DeclareProgram { name, .. } | c::AccountChange::LoginProgram { name, .. } => {
            if *refusal == c::Refusal::NameTaken {
                Some((W::ProgramNameTaken, Some(name)))
            } else if *refusal == c::Refusal::NotOffered {
                Some((W::ProgramNotOffered, None))
            } else {
                None
            }
        }
        c::AccountChange::Resume { .. } => {
            if *refusal == c::Refusal::NotOffered {
                Some((W::ResumeNotOffered, None))
            } else {
                None
            }
        }
        c::AccountChange::CancelLogin { .. }
        | c::AccountChange::LoginEnded { .. }
        | c::AccountChange::CheckProgram { .. }
        | c::AccountChange::AddKey { .. }
        | c::AccountChange::ReplaceKey { .. }
        | c::AccountChange::RemoveKey { .. }
        | c::AccountChange::Remove { .. }
        | c::AccountChange::MoveAccount { .. }
        | c::AccountChange::MoveSource { .. }
        | c::AccountChange::SetUses { .. }
        | c::AccountChange::MoveOut { .. }
        | c::AccountChange::Written { .. }
        | c::AccountChange::MoveIn { .. }
        | c::AccountChange::FinishSignIn { .. } => None,
    }
}
fn is_owner(root: &Domain) -> bool {
    match accounts::current_me(root.client()) {
        Some(me) => me.deployment_role == Some(c::HeldRole::Owner),
        None => false,
    }
}

pub(super) fn omitted(b: &mut Builder, w: &Words, word: W, actual: u32, limit: u32) {
    if actual > limit {
        fact(b, w, word, u64::from(actual.saturating_sub(limit)));
    }
}
fn confirmation(b: &mut Builder, root: &Domain, w: &Words, form: &accounts::Accounts, ask: &c::Ask) {
    let (held, account, source, word) = if let c::Ask::SignOut { holder, source } = ask {
        (*holder, source_account(root, form, *holder, *source), Some(*source), W::ConfirmSignOut)
    } else if let c::Ask::AccountChange { holder, change: c::AccountChange::RemoveKey { source } } = ask {
        (*holder, source_account(root, form, *holder, *source), Some(*source), W::ConfirmKey)
    } else if let c::Ask::AccountChange { holder, change: c::AccountChange::Remove { account } } = ask {
        (*holder, Some(*account), None, W::ConfirmAccount)
    } else {
        return;
    };
    b.open(Role::Alert);
    b.open(Role::Text);
    run(b, w, word);
    b.run(Run::Plain, b" \xc2\xb7 ");
    run(b, w, if held == c::AskAccountHolder::Own { W::OwnHolder } else { W::DeploymentHolder });
    b.run(Run::Plain, b" \xc2\xb7 ");
    if let Some(account) = account {
        run(b, w, W::Account);
        number(b, w, u64::from(account));
    } else {
        run(b, w, W::UnknownAccount);
    }
    if let Some(source) = source {
        b.run(Run::Plain, b" \xc2\xb7 ");
        run(b, w, W::Source);
        number(b, w, u64::from(source));
    }
    b.close();
    b.close();
}
// One confirmation names its source's account even when that row is beyond the display cap.
// Lookup allocates nothing; its input remains bounded by the admitted document.
fn source_account(root: &Domain, form: &accounts::Accounts, holder: c::AskAccountHolder, source: u32) -> Option<u32> {
    if holder == c::AskAccountHolder::Own {
        for row in &accounts::document(root.client())?.accounts {
            for entry in &row.sources {
                if entry.number == source {
                    return Some(row.number);
                }
            }
        }
    } else {
        for row in &form.deployment(root.client())?.accounts {
            for entry in &row.sources {
                if entry.number == source {
                    return Some(row.number);
                }
            }
        }
    }
    None
}

/// Ready names the first usable account by its kind and who it is, a
/// signed-in plan as "PLAN plan · WHO", with no holder: the model and
/// its effort are chosen elsewhere (client/pages/README.md).
pub(crate) fn first_usable_short(root: &Domain, w: &Words) -> Option<Box<[u8]>> {
    first_usable(root, w, true)
}
/// Ready names an own sign-in with the same provider words as its offered way.
pub(crate) fn own_signing_name(root: &Domain, w: &Words) -> Option<Box<[u8]>> {
    let document = accounts::document(root.client())?;
    let me = accounts::current_me(root.client())?;
    for signing in &document.signing {
        if signing.holder != c::AccountHolder::Party(me.party.number) {
            continue;
        }
        for offer in &me.providers {
            if offer.provider == signing.provider {
                let (brand, plan) = words::provider_words(offer.name.bytes());
                return Some(if offer.signed_in_plan {
                    w.say(crate::words::Word::WelcomePlanAccount, &[Hole::Name(crate::words::shown(plan, w))])
                } else {
                    crate::words::shown(brand, w)
                });
            }
        }
        let label = words::say(w, W::Provider, &[]);
        let number = words::say(w, W::Number, &[Hole::Number(u64::from(signing.provider))]);
        return Some(words::join(&[&label, &number]));
    }
    None
}

fn paste_way(root: &Domain, form: &accounts::request::Request) -> Option<accounts::Way> {
    if let Some(pending) = &form.pending
        && let c::Ask::SignIn { provider, .. } = pending.ask
    {
        for way in &accounts::ways(
            &accounts::current_me(root.client())?.providers,
            root.forge().playground_provider(),
            c::AskAccountHolder::Own,
        ) {
            if let accounts::Way::SignIn { provider: number, .. } = *way
                && number == provider
            {
                return Some(*way);
            }
        }
    }
    temper_client_domain::pages::connect_model::way(root.pages(), root.client())
}
pub(super) fn paste_plan<'a>(root: &'a Domain, form: &accounts::request::Request) -> &'a [u8] {
    if let Some(way) = paste_way(root, form)
        && let Some(me) = accounts::current_me(root.client())
    {
        for offer in &me.providers {
            if offer.provider == way.provider() {
                return words::provider_words(offer.name.bytes()).1;
            }
        }
    }
    b""
}
fn paste_restart(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    form: &accounts::request::Request,
) {
    let Some(way) = paste_way(root, form) else { return };
    let Some(me) = accounts::current_me(root.client()) else { return };
    let ways = accounts::ways(&me.providers, root.forge().playground_provider(), c::AskAccountHolder::Own);
    let meaning = if temper_client_domain::pages::connect_model::open(root.pages().address()) {
        Meaning::Model(temper_client_domain::pages::connect_model::Act::SignIn {
            name: root.pages().account_name(),
            way,
        })
    } else if let Some(page) = root.pages().accounts()
        && let Some(row) = page.request_row
    {
        Meaning::Action(accounts::Act::Submit {
            name: root.pages().account_name(),
            row,
            operation: accounts::Operation::SignIn { provider: way.provider() },
        })
    } else {
        return;
    };
    named_control(b, bindings, &super::connect_model::label(me, way, &ways, w), meaning, false, false);
}
/// The sign-in’s paste, folded while a model connection waits (client/pages/README.md).
pub(super) fn paste(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    form: &accounts::request::Request,
    folded: bool,
) {
    if form.paste_state(root.client()) == accounts::PasteState::Closed {
        return;
    }
    if folded {
        b.open(Role::Disclosure);
        if temper_client_domain::pages::connect_model::open(root.pages().address()) {
            b.style(look::MODEL_SIGN_IN);
            b.style(look::WELCOME_PASTE);
        }
        b.name(&words::say(w, W::PasteDisclosure, &[]));
    }
    paste_body(b, root, w, bindings, form);
    if folded {
        b.close();
    }
}
fn paste_body(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, form: &accounts::request::Request) {
    use accounts::PasteState as S;
    match form.paste_state(root.client()) {
        S::Closed
        | S::Refused { why: c::SignInRedirectWhy::NotWaiting | c::SignInRedirectWhy::TooMany }
        | S::Finishing
        | S::Ended => {}
        S::Refused { why: c::SignInRedirectWhy::Address | c::SignInRedirectWhy::State }
        | S::TooLong
        | S::Again
        | S::Full { .. }
        | S::Waiting { .. }
        | S::Sending => {
            crate::cards::text(b, Role::Paragraph, &named(w, W::PasteInstructions, paste_plan(root, form)));
        }
    }
    if paste_ending(b, root, w, bindings, form) {
        return;
    }
    paste_field(b, root, w, bindings, form, temper_client_domain::pages::connect_model::open(root.pages().address()));
    paste_finish(b, root, w, bindings, form, temper_client_domain::pages::connect_model::open(root.pages().address()));
}
/// Draws a paste’s ending or refusal, and reports whether its field is gone (3.5).
pub(super) fn paste_ending(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    form: &accounts::request::Request,
) -> bool {
    use accounts::PasteState as S;
    let alert = match form.paste_state(root.client()) {
        S::Closed | S::Refused { why: c::SignInRedirectWhy::NotWaiting } => return true,
        S::Finishing => {
            text(b, w, Role::Status, W::PasteFinishing);
            return true;
        }
        S::Ended => {
            text(b, w, Role::Status, W::PasteEnded);
            paste_restart(b, root, w, bindings, form);
            return true;
        }
        S::Refused { why: c::SignInRedirectWhy::TooMany } => {
            crate::cards::text(b, Role::Alert, &named(w, W::PasteTooMany, paste_plan(root, form)));
            paste_restart(b, root, w, bindings, form);
            return true;
        }
        S::Refused { why: c::SignInRedirectWhy::Address } => Some(W::PasteAddress),
        S::Refused { why: c::SignInRedirectWhy::State } => Some(W::PasteState),
        S::TooLong => Some(W::PasteLong),
        S::Again => Some(W::PasteAgain),
        S::Full { held, room_at } => {
            crate::cards::answers_full(b, w, held, room_at);
            None
        }
        S::Waiting { .. } | S::Sending => None,
    };
    if let Some(word) = alert {
        text(b, w, Role::Alert, word);
    }
    false
}
pub(super) fn paste_field(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    form: &accounts::request::Request,
    welcome: bool,
) {
    use accounts::PasteState as S;
    let state = form.paste_state(root.client());
    if let Some(binding) = bindings.insert(Meaning::SignInAddress) {
        b.open(Role::TextField);
        b.name(&if welcome {
            w.say(crate::words::Word::WelcomePasteField, &[])
        } else {
            words::say(w, W::PasteField, &[])
        });
        b.entry(skein_ui::EntryKind::Secret);
        b.value(b"", form.paste.written());
        b.bind(binding);
        if state == S::Sending {
            b.state(State::Disabled);
        }
        b.close();
    }
}
pub(super) fn paste_finish(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    form: &accounts::request::Request,
    welcome: bool,
) {
    use accounts::PasteState as S;
    let state = form.paste_state(root.client());
    let name = if welcome && state != S::Sending {
        w.say(crate::words::Word::WelcomePasteFinish, &[])
    } else {
        words::say(w, if state == S::Sending { W::PasteSending } else { W::PasteFinish }, &[])
    };
    if welcome {
        super::connect_model::button(
            b,
            bindings,
            &name,
            temper_client_domain::pages::connect_model::Act::Finish { name: root.pages().account_name() },
            state != (S::Waiting { ready: true }),
            true,
        );
    } else {
        let meaning = if temper_client_domain::pages::connect_model::open(root.pages().address()) {
            Meaning::Model(temper_client_domain::pages::connect_model::Act::Finish {
                name: root.pages().account_name(),
            })
        } else {
            Meaning::Action(accounts::Act::Finish { name: root.pages().account_name() })
        };
        named_control(b, bindings, &name, meaning, state != (S::Waiting { ready: true }), false);
    }
}

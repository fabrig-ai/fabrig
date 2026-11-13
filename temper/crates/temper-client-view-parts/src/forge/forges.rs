//! Kept forge rows and confirmed removal (temper/docs/design/client/forge/deployment.md; temper/docs/design/client/forge/family.md, 7).
use super::{bindings::Bindings as ForgeBindings, connect};
use crate::cards::CardWords as _;
use crate::{
    bindings::Bindings,
    common::settings,
    flow, look,
    words::{Word, Words},
};
use alloc::boxed::Box;
use skein_ui::{Builder, Role, phrases::Hole};
use temper_client_domain::{
    Address, Choice, Domain,
    forge::{
        self,
        forges::{Connection, Kind, Mode, Offer, Row, State},
        listed,
    },
    pages::Act,
};

pub fn main(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    forge: &mut ForgeBindings,
    sign_in: &crate::forge_sign_in::Forges,
    project: Option<u32>,
) {
    b.open(Role::Main);
    b.style(look::MAIN);
    b.style(look::SETTINGS);
    match project {
        Some(project) => settings::open(b, root, w, bindings, project, settings::Tab::Forges),
        None => settings::open_alone(b, &w.say(Word::ForgesTitle, &[]), &w.say(Word::SettingsForgesLede, &[])),
    }
    let rows = listed::rows(root.forges(), forge::connect::playground(root.forge()));
    let mut count = 0_u64;
    for row in &rows {
        if row.mode != Mode::Fake {
            count = count.checked_add(1).expect("bounded rows");
        }
    }
    let heading = match root.forges() {
        State::Rows(_) => w.say(Word::YourForges, &[Hole::Number(count)]),
        State::Unknown | State::Ended => w.say(Word::YourForgesTitle, &[]),
    };
    b.open(Role::Slot);
    b.style(look::SETTINGS_SECTION);
    b.style(look::FORGES);
    b.open(Role::Slot);
    b.style(look::SETTINGS_SECTION_HEAD);
    b.open(Role::Heading { level: 2 });
    b.style(look::SETTINGS_SECTION_TITLE);
    b.name(&heading);
    b.text(skein_ui::Run::Plain, &heading);
    b.close();
    if let State::Rows(_) = root.forges()
        && !root.forge().playground_running()
    {
        settings::button_to(
            b,
            &w.say(Word::AddForge, &[]),
            bindings.insert(Act::Go { address: Address::ConnectForge { forge: None } }),
            false,
        );
    }
    b.close();
    confirmation(b, root, w, bindings, project);
    match root.forges() {
        State::Unknown => {}
        State::Ended => crate::cards::refusal(b, w, &jig_client::contract::Refusal::NotOffered),
        State::Rows(_) => {
            if count == 0 && !root.forge().playground_running() {
                settings::quiet_band(b, &w.say(Word::NoForgeTag, &[]), &w.say(Word::NoForges, &[]));
                settings::button_to(
                    b,
                    &w.say(Word::ConnectForge, &[]),
                    bindings.insert(Act::Go { address: Address::ConnectForge { forge: None } }),
                    true,
                );
                b.close();
            } else if count != 0 {
                b.open(Role::Slot);
                b.style(look::SETTINGS_LINES);
                for row in &rows {
                    if row.mode != Mode::Fake {
                        personal(b, root, w, bindings, forge, row, sign_in);
                    }
                }
                b.close();
            }
        }
    }
    b.close();
    if let Some((playground, row)) = listed::playground(root) {
        settings::section_head(b, &w.say(Word::ForgePlaygroundHeading, &[]));
        b.close();
        b.open(Role::Slot);
        b.style(look::SETTINGS_LINES);
        b.open(Role::Disclosure);
        b.style(look::SETTINGS_ROW);
        b.name(&w.say(Word::ForgePlayground, &[]));
        b.description(&forge::words::full_name(&playground.owner, &playground.name));
        account(b, w, row);
        b.close();
        b.close();
        b.close();
    }
    b.close();
    b.close();
}

fn confirmation(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, project: Option<u32>) {
    if let Some(number) = root.pages().forges_opened()
        && let Some(row) = forge::connect::row(number, root.forges(), forge::connect::playground(root.forge()))
        && let Connection::Connected { .. } = row.connection
        && !forge::connect::underway(row)
        && !forge::connect::failed(row)
    {
        b.open(Role::Paragraph);
        b.style(look::CONNECT_CONFIRM);
        let line = w.say(Word::ForgeConnectedConfirm, &[Hole::Name(row_name(w, row, root))]);
        b.name(&line);
        b.text(skein_ui::Run::Plain, &line);
        if let Some(project) = project {
            flow::connect_control(
                b,
                flow::Control::Link {
                    name: &w.say(Word::ForgeAdopt, &[]),
                    to: bindings.insert(Act::Go { address: Address::Adopt { project } }),
                },
            );
        }
        b.close();
    }
}

fn connection(row: &Row, root: &Domain, w: &Words) -> Box<[u8]> {
    if let Some(draft) = root.forge().replacement(row.number)
        && draft.connecting()
    {
        return w.say(Word::Connecting, &[]);
    }
    if row.mode == Mode::Managed && forge::connect::underway(row) && !forge::connect::failed(row) {
        return w.say(Word::ForgeRowSettingUp, &[]);
    }
    if forge::connect::failed(row) {
        return forge::words::forge_connection(row);
    }
    if row.kind == Kind::GitHub && forge::connect::underway(row) {
        return forge::words::forge_connection(row);
    }
    match &row.connection {
        Connection::Connected { login, .. } => forge::words::forge_connected_as(login),
        Connection::NotConnected
        | Connection::NotSetUp
        | Connection::SettingUp { .. }
        | Connection::SetUpFailed { .. }
        | Connection::Connecting
        | Connection::Stopped { .. } => forge::words::forge_connection(row),
    }
}
fn row_name(w: &Words, row: &Row, root: &Domain) -> Box<[u8]> {
    if row.kind == Kind::Forgejo && row.mode == Mode::Managed {
        w.say(Word::ConnectManaged, &[])
    } else {
        connect::name(w, row, root)
    }
}
fn description(root: &Domain, w: &Words, row: &Row, setting_up: bool) -> Box<[u8]> {
    let kind = match row.kind {
        Kind::Forgejo => Word::ForgeOther,
        Kind::GitHub => Word::ForgeGitHub,
    };
    let summary = forge::words::forge_summary(
        &w.say(Word::ForgeSummary, &[Hole::Number(u64::from(row.number)), Hole::Name(w.say(kind, &[]))]),
        &connection(row, root, w),
    );
    let line = repository_line(root, w, row);
    if let Some(line) = line {
        let bytes = summary
            .len()
            .checked_add(if setting_up { 4 } else { 1 })
            .expect("summary fits")
            .checked_add(line.len())
            .expect("summary fits");
        let mut text = skein_lib::Writer::new(bytes);
        text.put(&summary).expect("summary counted");
        text.put(if setting_up { b" \xc2\xb7 " } else { b"\n" }).expect("separator counted");
        text.put(&line).expect("repository line counted");
        text.finish()
    } else {
        summary
    }
}

fn personal(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    forge: &mut ForgeBindings,
    row: &Row,
    sign_in: &crate::forge_sign_in::Forges,
) {
    b.open(Role::Disclosure);
    b.style(look::SETTINGS_ROW);
    let setting_up = forge::connect::underway(row) || forge::connect::failed(row);
    if setting_up || root.pages().forges_opened() == Some(row.number) {
        b.state(skein_ui::State::Expanded);
    }
    b.name(&row_name(w, row, root));
    b.description(&description(root, w, row, setting_up));
    if setting_up {
        connect::live(b, root, w, row);
        if row.kind == Kind::Forgejo && row.mode == Mode::Managed && !forge::connect::failed(row) {
            b.close();
            return;
        }
    }
    if row.admin && forge::connect::admin_stopped(row) {
        connect::admin_details(b, root, w, forge, bindings, row);
    }
    if let Some(first) = &row.first {
        match first.state {
            forge::forges::FirstState::Made { .. } => {}
            forge::forges::FirstState::Making | forge::forges::FirstState::Failed { .. } => {
                crate::cards::text(b, Role::Paragraph, &forge::words::first_repository(first));
            }
        }
    }
    account(b, w, row);
    access(b, w, row);
    if open_link(b, w, row, sign_in, None)
        && let Some(me) = root.pages().me(root.client())
    {
        b.open(Role::Paragraph);
        w.text(b, Word::SignInOpenBody, &[Hole::Name(me.party.login.bytes().into())]);
        b.close();
    }
    owned(b, w, row);
    repositories(b, w, row, sign_in);
    if row.admin
        && !setting_up
        && let Some(first) = &row.first
    {
        let repository = forge::words::full_name(&first.org, &first.repository);
        crate::cards::text(
            b,
            Role::Paragraph,
            &w.say(Word::AdminReady, &[Hole::Name(repository.clone()), Hole::Name(first.label.clone())]),
        );
        if let Some(web) = &row.web
            && let Some(address) = super::repository::web_address(web, &first.org, &first.repository)
        {
            b.open(Role::Link);
            b.name(&w.say(Word::AdminRepository, &[Hole::Name(repository)]));
            b.external(&address);
            b.close();
        }
    }
    connect::app_line(b, w, row, true);
    if let Some(line) = forge::words::last_connect(row) {
        crate::cards::text(b, Role::Paragraph, &line);
    }
    if row.kind == Kind::GitHub {
        github(b, root, w, bindings, forge, row);
    }
    tokens(b, root, w, bindings, forge, row);
    if forge::connect::failed(row) && row.offer == Offer::Offered {
        connect::setup_control(b, root, w, forge, row.number, Word::WelcomeTryAgain);
    } else {
        match row.connection {
            Connection::SetUpFailed { .. } if row.offer == Offer::Offered => {
                connect::setup_control(b, root, w, forge, row.number, Word::WelcomeTryAgain);
            }
            Connection::Stopped { .. } if row.mode == Mode::Owned => {
                connect::text(b, w, Role::Paragraph, Word::ForgeRestartBox);
            }
            Connection::Connected { .. } => connect::adopt(b, root, w, bindings),
            Connection::NotConnected
            | Connection::NotSetUp
            | Connection::SettingUp { .. }
            | Connection::Connecting
            | Connection::SetUpFailed { .. }
            | Connection::Stopped { .. } => {}
        }
    }
    if w.shell == crate::words::Shell::Browser && row.kind == Kind::Forgejo && sign_in.off(row.number) {
        connect::text(b, w, Role::Paragraph, Word::SignInOff);
    }
    replacement(b, root, w, forge, row);
    removal(b, root, w, forge, row);
    b.close();
}
fn repositories(b: &mut Builder, w: &Words, row: &Row, sign_in: &crate::forge_sign_in::Forges) {
    if row.repositories.is_empty() {
        if let Some(first) = &row.first
            && let forge::forges::FirstState::Made { .. } = first.state
            && let Connection::Connected { .. } = row.connection
        {
            repository(b, w, row, sign_in, &first.org, &first.repository);
        }
    } else {
        for adopted in &row.repositories {
            repository(b, w, row, sign_in, &adopted.owner, &adopted.name);
        }
    }
}
fn repository(
    b: &mut Builder,
    w: &Words,
    row: &Row,
    sign_in: &crate::forge_sign_in::Forges,
    owner: &[u8],
    name: &[u8],
) {
    b.open(Role::Slot);
    b.style(look::SETTINGS_SETTING);
    connect::text(b, w, Role::Term, Word::GitHubRepositories);
    b.open(Role::Description);
    b.name(&forge::words::full_name(owner, name));
    super::repository::name(b, w, Some(row), sign_in, owner, name);
    b.close();
    b.close();
}
fn tokens(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, forge: &mut ForgeBindings, row: &Row) {
    if forge::tokens::offered(row) {
        if root.pages().forges_opened() == Some(row.number) {
            connect::tokens_details(b, root, w, forge, bindings, row);
        } else {
            connect::link(
                b,
                w,
                bindings,
                Word::TokensLink,
                Address::ConnectForge { forge: Some(Choice::Forge(row.number)) },
            );
        }
    }
}

fn github(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, forge: &mut ForgeBindings, row: &Row) {
    if let Some(web) = &row.web {
        b.open(Role::Link);
        b.name(&w.say(Word::GitHubOpen, &[]));
        b.external(web);
        b.close();
    }
    match row.connection {
        Connection::Connected { .. } => {
            let accepted = forge::forges::accepted(row);
            if !accepted {
                flow::card(b, &w.say(Word::ForgeInstallTitle, &[]));
                if row.app == Some(forge::forges::AppWay::Manifest) {
                    let slug = forge::forges::slug(row).expect("connected App slug");
                    crate::cards::text(
                        b,
                        Role::Paragraph,
                        &w.say(Word::ForgeManifestInstall, &[Hole::Name(Box::from(slug))]),
                    );
                } else {
                    crate::cards::text(b, Role::Paragraph, &forge::words::app_install(row));
                }
            }
            connect::app_control(
                b,
                root,
                w,
                forge,
                row.number,
                if accepted { Word::ForgeInstallAnother } else { Word::ForgeInstall },
                Some(Address::Forges),
            );
            if !accepted {
                b.close();
            }
        }
        Connection::NotConnected | Connection::Stopped { .. } => connect::link(
            b,
            w,
            bindings,
            Word::ForgeConnectApp,
            Address::ConnectForge { forge: Some(Choice::Forge(row.number)) },
        ),
        Connection::NotSetUp
        | Connection::SetUpFailed { .. }
        | Connection::SettingUp { .. }
        | Connection::Connecting => {}
    }
}
fn repository_line(_root: &Domain, w: &Words, row: &Row) -> Option<Box<[u8]>> {
    if forge::connect::failed(row) {
        return None;
    }
    let first = row.first.as_ref()?;
    let word = match first.state {
        forge::forges::FirstState::Failed { .. } => return None,
        forge::forges::FirstState::Making => Word::SetupInto,
        forge::forges::FirstState::Made { .. } => match row.connection {
            Connection::Connected { .. } => Word::SetupMade,
            Connection::NotConnected
            | Connection::NotSetUp
            | Connection::SettingUp { .. }
            | Connection::SetUpFailed { .. }
            | Connection::Connecting
            | Connection::Stopped { .. } => Word::SetupInto,
        },
    };
    Some(w.say(word, &[Hole::Name(forge::words::full_name(&first.org, &first.repository))]))
}
fn account(b: &mut Builder, w: &Words, row: &Row) {
    if let Connection::Connected { id, login } = &row.connection {
        b.open(Role::Slot);
        b.style(look::SETTINGS_SETTING);
        connect::text(b, w, Role::Term, Word::ForgeAccountLabel);
        crate::cards::text(b, Role::Description, &forge::words::forge_account(login, *id));
        b.close();
    }
}
fn access(b: &mut Builder, w: &Words, row: &Row) {
    match row.kind {
        Kind::GitHub => return,
        Kind::Forgejo => {}
    }
    let word = match row.mode {
        Mode::Owned => Word::ForgeAccessOwned,
        Mode::Managed => Word::ForgeAccessManaged,
        Mode::Connected => Word::ForgeAccessConnected,
        Mode::Fake => return,
    };
    b.open(Role::Slot);
    b.style(look::SETTINGS_SETTING);
    connect::text(b, w, Role::Term, Word::ForgeAccessLabel);
    connect::text(b, w, Role::Description, word);
    b.close();
}

fn owned(b: &mut Builder, w: &Words, row: &Row) {
    let Some(owned) = &row.owned else { return };
    match row.connection {
        Connection::Connecting | Connection::Connected { .. } | Connection::Stopped { .. } => {}
        Connection::NotConnected
        | Connection::NotSetUp
        | Connection::SettingUp { .. }
        | Connection::SetUpFailed { .. } => return,
    }
    b.open(Role::Slot);
    b.style(look::SETTINGS_SETTING);
    connect::text(b, w, Role::Term, Word::ForgeRunsLabel);
    crate::cards::text(b, Role::Description, &forge::words::forge_runs(owned));
    b.close();
    b.open(Role::Slot);
    b.style(look::SETTINGS_SETTING);
    connect::text(b, w, Role::Term, Word::ForgePasswordLabel);
    crate::cards::text(b, Role::Description, &forge::words::forge_password(&owned.password));
    b.close();
}

/// Shared with the welcome's adopted Ready; the page facts opt a connected Forgejo in.
pub fn open_link(
    b: &mut Builder,
    w: &Words,
    row: &Row,
    sign_in: &crate::forge_sign_in::Forges,
    style: Option<skein_ui::Style>,
) -> bool {
    if row.kind != Kind::Forgejo {
        return false;
    }
    match row.connection {
        Connection::Connected { .. } => {}
        Connection::NotConnected
        | Connection::NotSetUp
        | Connection::SettingUp { .. }
        | Connection::SetUpFailed { .. }
        | Connection::Connecting
        | Connection::Stopped { .. } => return false,
    }
    let Some(forge) = sign_in.number(row.number) else { return false };
    let label = w.say(Word::SignInOpen, &[]);
    b.open(Role::Link);
    b.name(&label);
    if let Some(style) = style {
        b.style(style);
    }
    b.external(&forge.open());
    b.text(skein_ui::Run::Plain, &label);
    b.close();
    true
}

fn removal(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut ForgeBindings, row: &Row) {
    use super::bindings::Meaning;
    use forge::forges::Act;
    if let Some(removal) = &root.forge().setup_form().removal
        && removal.forge == row.number
    {
        connect::text(b, w, Role::Paragraph, Word::ForgeRemoveConfirm);
        removal_words(b, w, row);
        let name = w.say(Word::ForgeRemove, &[]);
        let control = if removal.request.is_some() {
            flow::Control::PrimaryDisabled { name: &name }
        } else {
            flow::Control::PrimaryButton { name: &name, to: bindings.insert(Meaning::Forges(Act::ConfirmRemove)) }
        };
        flow::control(b, control);
    } else {
        flow::control(
            b,
            flow::Control::Button {
                name: &w.say(Word::ForgeRemove, &[]),
                to: bindings.insert(Meaning::Forges(Act::Remove { forge: row.number })),
            },
        );
    }
}
pub(crate) fn removal_words(b: &mut Builder, w: &Words, row: &Row) {
    let word = match row.kind {
        Kind::GitHub => Word::ForgeRemoveGitHub,
        Kind::Forgejo => match row.mode {
            Mode::Owned => Word::ForgeRemoveOwned,
            Mode::Connected | Mode::Managed | Mode::Fake => Word::ForgeRemoveManaged,
        },
    };
    connect::text(b, w, Role::Paragraph, word);
    if row.kind == Kind::GitHub {
        let mut web = match &row.web {
            Some(web) => web.as_ref(),
            None => b"https://github.com",
        };
        while let Some(prefix) = web.strip_suffix(b"/") {
            web = prefix;
        }
        let suffix = b"/settings/apps";
        let mut address = skein_lib::Writer::new(web.len().checked_add(suffix.len()).expect("settings address fits"));
        address.put(web).expect("web counted");
        address.put(suffix).expect("suffix counted");
        b.open(Role::Link);
        b.name(&w.say(Word::ForgeRemoveGitHubLink, &[]));
        b.external(&address.finish());
        b.close();
    }
}

fn replacement(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut ForgeBindings, row: &Row) {
    use super::bindings::Meaning;
    use forge::{forges::Act, replace::Field};
    let draft = root.forge().replacement(row.number);
    if draft.is_none() {
        if forge::replace::offered(row) {
            flow::control(
                b,
                flow::Control::Button {
                    name: &w.say(Word::ForgeReplace, &[]),
                    to: bindings.insert(Meaning::Forges(Act::OpenReplace { forge: row.number })),
                },
            );
        }
        return;
    }
    let draft = draft.expect("opened replacement");
    let github = row.kind == Kind::GitHub;
    let terminal = github && w.shell == crate::words::Shell::Terminal;
    if terminal {
        crate::cards::text(
            b,
            Role::Paragraph,
            &w.say(Word::ForgeReplaceTerminal, &[Hole::Number(u64::from(row.number))]),
        );
    } else {
        let fields: &[(Field, Word)] = if github {
            &[(Field::Key, Word::ForgePrivateKey)]
        } else {
            &[(Field::Api, Word::TokensApi), (Field::Git, Word::TokensGit), (Field::Webhook, Word::TokensWebhook)]
        };
        let missing = if github { !draft.has(Field::Key) } else { !draft.has(Field::Api) && !draft.has(Field::Git) };
        for (field, word) in fields {
            b.open(Role::TextField);
            b.name(&w.say(*word, &[]));
            b.entry(skein_ui::EntryKind::Secret);
            b.value(b"", *draft.written.get(field.index()).expect("four writes"));
            b.bind(bindings.insert(Meaning::ReplaceValue { forge: row.number, field: *field }));
            if draft.checked && (draft.too_long || (missing && *field != Field::Webhook)) {
                b.state(skein_ui::State::Invalid);
            }
            b.close();
        }
        let title = w.say(if github { Word::ForgeReplaceKey } else { Word::ForgeReplaceTokens }, &[]);
        let control =
            if draft.busy() || !forge::replace::offered(row) || root.forge().served() == forge::Served::OutOfDate {
                flow::Control::PrimaryDisabled { name: &title }
            } else {
                flow::Control::PrimaryButton {
                    name: &title,
                    to: bindings.insert(Meaning::Forges(Act::Replace { forge: row.number })),
                }
            };
        flow::control(b, control);
        if draft.checked {
            if draft.too_long {
                connect::text(b, w, Role::Alert, Word::AdminTooLong);
            } else if missing {
                connect::text(b, w, Role::Alert, if github { Word::SetupRequired } else { Word::ForgeReplaceEmpty });
            }
        }
        if let Some(answer) = &draft.answer {
            match answer {
                forge::github::Answer::Refused(temper_client_protocol_wire::v1::ForgeRefusal::NotConnected) => {
                    connect::text(b, w, Role::Alert, Word::ForgeReplaceFirst);
                }
                forge::github::Answer::Connecting => {}
                forge::github::Answer::Entrance(refusal) => {
                    b.open(Role::Alert);
                    w.refusal(b, refusal.refusal(), w.name_bytes);
                    b.close();
                }
                forge::github::Answer::Link(_)
                | forge::github::Answer::Page(_)
                | forge::github::Answer::NoAccess
                | forge::github::Answer::NotFound
                | forge::github::Answer::Refused(_)
                | forge::github::Answer::Unknown => {
                    if let Some(line) = forge::words::app_refusal(answer.clone(), row.number) {
                        crate::cards::text(b, Role::Alert, &line);
                    }
                }
            }
        }
    }
    flow::control(
        b,
        flow::Control::Button {
            name: &w.say(Word::ForgeReplaceClose, &[]),
            to: bindings.insert(Meaning::Forges(Act::CloseReplace { forge: row.number })),
        },
    );
}

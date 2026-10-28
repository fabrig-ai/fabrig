//! Folded forge rows and the playground apart (client/forge.md, 5.4).
use super::{bindings::Bindings as ForgeBindings, connect};
use crate::{
    flow,
    frame::Bindings,
    look,
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
};

pub(crate) fn main(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    forge: &mut ForgeBindings,
    sign_in: &crate::forge_sign_in::Forges,
) {
    b.open(Role::Main);
    b.style(look::MAIN);
    b.style(look::FORGES);
    connect::text(b, w, Role::Heading { level: 1 }, Word::ForgesTitle);
    let rows = listed::rows(root.forges(), root.forge().demo());
    let count = u64::from(rows.len());
    let heading = match root.forges() {
        State::Rows(_) => w.say(Word::YourForges, &[Hole::Number(count)]),
        State::Unknown | State::Ended => w.say(Word::YourForgesTitle, &[]),
    };
    crate::cards::text(b, Role::Heading { level: 2 }, &heading);
    match root.forges() {
        State::Unknown => {}
        State::Ended => crate::cards::refusal(b, w, &jig_client::contract::Refusal::NotOffered),
        State::Rows(_) => {
            connect::text(b, w, Role::Paragraph, Word::ForgesDescription);
            connect::link(b, w, bindings, Word::AddForge, Address::ConnectForge { forge: None });
            if count == 0 {
                connect::text(b, w, Role::Paragraph, Word::NoForges);
                connect::link(b, w, bindings, Word::ConnectForge, Address::ConnectForge { forge: None });
            }
            for row in &rows {
                personal(b, root, w, bindings, forge, row, sign_in);
            }
            if let Some((demo, row)) = listed::playground(root) {
                connect::text(b, w, Role::Heading { level: 2 }, Word::ForgePlaygroundHeading);
                b.open(Role::Disclosure);
                b.style(look::FORGE_ROW);
                b.name(&w.say(
                    Word::ForgePlaygroundSummary,
                    &[Hole::Name(demo.owner.clone()), Hole::Name(demo.name.clone())],
                ));
                account(b, w, row);
                b.close();
            }
        }
    }
    b.close();
}
fn connection(row: &Row) -> Box<[u8]> {
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
    b.style(look::FORGE_ROW);
    b.name(&connect::name(w, row, root));
    let kind = match row.kind {
        Kind::Forgejo => Word::ForgeOther,
        Kind::GitHub => Word::ForgeGitHub,
    };
    let summary = forge::words::forge_summary(
        &w.say(Word::ForgeSummary, &[Hole::Number(u64::from(row.number)), Hole::Name(w.say(kind, &[]))]),
        &connection(row),
    );
    let line = repository_line(root, w, row);
    if let Some(line) = line {
        let bytes = summary.len().checked_add(1).expect("summary fits").checked_add(line.len()).expect("summary fits");
        let mut text = skein_lib::Writer::new(bytes);
        text.put(&summary).expect("summary counted");
        text.put(b"\n").expect("separator counted");
        text.put(&line).expect("repository line counted");
        b.description(&text.finish());
    } else {
        b.description(&summary);
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
    if let Some(first) = &row.first
        && let forge::forges::FirstState::Made { .. } = first.state
        && let Connection::Connected { .. } = row.connection
    {
        b.open(Role::Slot);
        connect::text(b, w, Role::Term, Word::GitHubRepositories);
        crate::cards::text(
            b,
            Role::Description,
            &w.say(
                Word::SetupMade,
                &[
                    Hole::Name(forge::words::full_name(&first.org, &first.repository)),
                    Hole::Name(connect::project_name(root, w, &first.project)),
                ],
            ),
        );
        b.close();
    }
    connect::app_line(b, w, row);
    if let Some(line) = forge::words::last_connect(row) {
        crate::cards::text(b, Role::Paragraph, &line);
    }
    if row.kind == Kind::GitHub {
        github(b, root, w, bindings, forge, row);
    }
    if forge::connect::failed(row) && row.offer == Offer::Offered {
        connect::setup_control(b, root, w, forge, row.number, Word::ForgeAgain);
    } else {
        match row.connection {
            Connection::SetUpFailed { .. } if row.offer == Offer::Offered => {
                connect::setup_control(b, root, w, forge, row.number, Word::ForgeAgain);
            }
            Connection::SettingUp { .. } | Connection::Connecting => connect::link(
                b,
                w,
                bindings,
                Word::ForgeOpenSetup,
                Address::ConnectForge { forge: Some(Choice::Forge(row.number)) },
            ),
            Connection::Stopped { .. } if row.mode == Mode::Owned => {
                connect::text(b, w, Role::Paragraph, Word::ForgeRestartBox);
            }
            Connection::Connected { .. } => connect::adopt(b, root, w, bindings),
            Connection::NotConnected
            | Connection::NotSetUp
            | Connection::SetUpFailed { .. }
            | Connection::Stopped { .. } => {}
        }
    }
    b.close();
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
fn repository_line(root: &Domain, w: &Words, row: &Row) -> Option<Box<[u8]>> {
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
    Some(w.say(
        word,
        &[
            Hole::Name(forge::words::full_name(&first.org, &first.repository)),
            Hole::Name(connect::project_name(root, w, &first.project)),
        ],
    ))
}
fn account(b: &mut Builder, w: &Words, row: &Row) {
    if let Connection::Connected { id, login } = &row.connection {
        b.open(Role::Slot);
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
    connect::text(b, w, Role::Term, Word::ForgeRunsLabel);
    crate::cards::text(b, Role::Description, &forge::words::forge_runs(owned));
    b.close();
    b.open(Role::Slot);
    connect::text(b, w, Role::Term, Word::ForgePasswordLabel);
    crate::cards::text(b, Role::Description, &forge::words::forge_password(&owned.password));
    b.close();
}

/// Shared with the welcome's adopted Ready; the page facts opt a connected Forgejo in.
pub(crate) fn open_link(
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

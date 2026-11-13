//! Forge choices and unfinished setup (temper/docs/design/client/forge/deployment.md; temper/docs/design/client/pages/README.md).
//! `ConnectForge` and the welcome's shared forge body (client/forge/deployment.md).
use super::bindings::{Bindings as ForgeBindings, Meaning};
use crate::{
    bindings::Bindings,
    flow::{self, Control, Mark},
    look,
    words::{Shell, Word, Words},
};
use alloc::boxed::Box;
use skein_lib::bytes::copy_of;
use skein_ui::{Builder, Fraction, Role, Run, Shortcut, State, phrases::Hole};
use temper_client_domain::{
    Address, Choice, Domain, Step,
    forge::{
        self,
        connect::{self, Name, Shown},
        forges::{Connection, FirstState, Kind, Mode, Offer, Row, SetUpAnswer, SetUpOutcome, SetupField, Start},
    },
    pages::Act,
};

/// "Run temper on a server" (temper/docs/help/server.md), on GitHub: the
/// browser serves no help pages (the look request welcome-forgejo-card).
pub(crate) const DEPLOYMENT: &[u8] = b"https://github.com/fabrig-ai/fabrig/blob/main/temper/docs/help/server.md";

#[derive(Clone, Copy)]
pub(crate) enum Place {
    Connect,
    Welcome,
}
impl Place {
    fn address(self, forge: Option<Choice>) -> Address {
        match self {
            Self::Connect => Address::ConnectForge { forge },
            Self::Welcome => Address::Welcome { step: Step::Forge { forge } },
        }
    }
}
pub(crate) fn text(b: &mut Builder, w: &Words, role: Role, word: Word) {
    crate::cards::text(b, role, &w.say(word, &[]));
}
fn styled_text(b: &mut Builder, role: Role, words: &[u8], style: skein_ui::Style) {
    b.open(role);
    b.style(style);
    b.name(words);
    b.text(Run::Plain, words);
    b.close();
}
pub(crate) fn link(b: &mut Builder, w: &Words, bindings: &mut Bindings, word: Word, address: Address) {
    let name = w.say(word, &[]);
    let to = bindings.insert(Act::Go { address });
    let action = if word == Word::ConnectForge {
        Control::PrimaryLink { name: &name, to }
    } else if word == Word::AddForge || word == Word::ForgeConnectApp {
        Control::SecondaryLink { name: &name, to }
    } else {
        Control::Link { name: &name, to }
    };
    flow::control(b, action);
}
pub(crate) fn name(w: &Words, row: &Row, root: &Domain) -> Box<[u8]> {
    w.say(
        match connect::name(row, connect::playground(root.forge())) {
            Name::Here => Word::ForgeHere,
            Name::GitHub => Word::ForgeGitHub,
            Name::Forgejo => Word::ForgeOther,
            Name::Playground => Word::ForgePlayground,
        },
        &[],
    )
}
pub(crate) fn personal(root: &Domain) -> Option<&Row> {
    connect::personal(root.forges(), connect::playground(root.forge()))
}
#[expect(clippy::too_many_arguments, reason = "the flow path takes its project beside the forge bindings")]
pub(crate) fn main(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    forge: &mut ForgeBindings,
    accounts: &mut crate::accounts::Bindings,
    choice: Option<Choice>,
    project: Option<u32>,
    sign_in: &crate::forge_sign_in::Forges,
) {
    b.open(Role::Main);
    b.style(look::MAIN);
    b.style(look::SETTINGS);
    b.open(Role::Slot);
    b.style(look::SETTINGS_COL);
    b.style(look::CONNECT);
    let shown = if choice.is_none() {
        match connect::unfinished(connect::records(root.forges()), connect::playground(root.forge())) {
            Some(row) => Shown::Unfinished(row.number),
            None => Shown::Choose,
        }
    } else {
        connect::step(choice, root.forges(), connect::playground(root.forge()), owned(root))
    };
    let yours = match shown {
        Shown::Details(choice) => choice == Choice::Managed || your_forgejo(root, Some(choice)),
        Shown::Choose | Shown::Unfinished(_) | Shown::SettingUp(_) | Shown::Ready(_) => false,
    };
    if choice == Some(Choice::Elsewhere) || yours {
        elsewhere_path(b, root, w, bindings, project);
    } else {
        crate::common::settings::path(
            b,
            root,
            w,
            bindings,
            project,
            crate::common::settings::FlowTab::Forges,
            Word::PathAddForge,
        );
    }
    styled_text(
        b,
        Role::Heading { level: 1 },
        &w.say(
            if choice == Some(Choice::Elsewhere) {
                Word::ForgeElsewhere
            } else if yours {
                Word::ConnectManaged
            } else {
                Word::WelcomeForgeTitle
            },
            &[],
        ),
        look::SETTINGS_HEADLINE,
    );
    match shown {
        Shown::Details(Choice::Elsewhere) => elsewhere(b, root, w, bindings, Some(accounts)),
        Shown::Choose | Shown::Details(_) => body(b, root, w, bindings, forge, choice, Place::Connect, sign_in),
        Shown::Unfinished(number) => {
            if let Some(row) = connect::row(number, root.forges(), connect::playground(root.forge())) {
                unfinished(b, root, w, forge, row);
            }
        }
        Shown::SettingUp(_) | Shown::Ready(_) => {}
    }
    b.close();
    b.close();
}
fn elsewhere_path(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, project: Option<u32>) {
    crate::common::settings::way_path(
        b,
        root,
        w,
        bindings,
        project,
        crate::common::settings::FlowTab::Forges,
        &crate::common::settings::FlowWay {
            choices: Word::PathAddForge,
            address: Place::Connect.address(None),
            name: &w.say(Word::ConnectManaged, &[]),
        },
    );
}
#[expect(clippy::too_many_arguments, reason = "the shared forge body draws both places with page-owned sign-in facts")]
pub(crate) fn body(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    forge: &mut ForgeBindings,
    choice: Option<Choice>,
    place: Place,
    sign_in: &crate::forge_sign_in::Forges,
) {
    let shown = match place {
        Place::Connect => connect::step(choice, root.forges(), connect::playground(root.forge()), owned(root)),
        Place::Welcome => connect::welcome_step(choice, root.forges(), connect::playground(root.forge()), owned(root)),
    };
    match shown {
        Shown::Choose => choose(b, root, w, bindings, forge, place, sign_in),
        Shown::Details(choice) => {
            match choice {
                Choice::Managed => managed(b, root, w, forge, bindings),
                Choice::Owned => {
                    choose(b, root, w, bindings, forge, place, sign_in);
                    sheet(b, root, w, bindings, forge, place);
                }
                Choice::GitHub => flow::control(
                    b,
                    Control::PrimaryButton {
                        name: &w.say(Word::ConnectGitHub, &[]),
                        to: forge.insert(Meaning::Forges(forge::forges::Act::GitHub {
                            back: temper_client_domain::address::print(&match place {
                                Place::Connect => Address::Forges,
                                Place::Welcome => place.address(None),
                            }),
                        })),
                    },
                ),
                Choice::Elsewhere => elsewhere(b, root, w, bindings, None),
                Choice::Forge(number) => {
                    if let Some(row) = connect::row(number, root.forges(), connect::playground(root.forge())) {
                        match row.kind {
                            Kind::GitHub => github(b, root, w, forge, row),
                            Kind::Forgejo if row.mode == Mode::Connected => {
                                tokens_details(b, root, w, forge, bindings, row);
                            }
                            Kind::Forgejo if row.admin => admin_details(b, root, w, forge, bindings, row),
                            Kind::Forgejo => here(b, root, w, forge, row),
                        }
                    }
                }
            }
            if let Place::Connect = place
                && choice != Choice::Owned
                && choice != Choice::Elsewhere
                && choice != Choice::Managed
                && !your_forgejo(root, Some(choice))
            {
                b.open(Role::Slot);
                b.style(look::CONNECT_ACTS);
                flow::connect_control(
                    b,
                    Control::Link {
                        name: &w.say(Word::ForgeAll, &[]),
                        to: bindings.insert(Act::Go { address: place.address(None) }),
                    },
                );
                b.close();
            }
        }
        Shown::Unfinished(number) => {
            if let Some(row) = connect::row(number, root.forges(), connect::playground(root.forge())) {
                unfinished(b, root, w, forge, row);
            }
        }
        Shown::SettingUp(_) | Shown::Ready(_) => {}
    }
}
/// The welcome keeps GitHub's installation and Ready steps in its own flow.
pub(crate) fn welcome(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    forge: &mut ForgeBindings,
    choice: Option<Choice>,
    sign_in: &crate::forge_sign_in::Forges,
) {
    if choice.is_none()
        && let Some(row) = connect::unfinished(connect::records(root.forges()), connect::playground(root.forge()))
    {
        unfinished(b, root, w, forge, row);
        return;
    }
    if let Some(row) = personal(root) {
        if connect::status_shown(row, connect::playground(root.forge())) {
            forge_status(b, root, w, row, sign_in);
            b.open(Role::Slot);
            b.style(look::WELCOME_ACTS);
            flow::control(
                b,
                Control::PrimaryLink {
                    name: &w.say(Word::WelcomeContinue, &[]),
                    to: bindings.insert(Act::Go {
                        address: Address::Welcome { step: temper_client_domain::pages::welcome::go_on(root) },
                    }),
                },
            );
            b.close();
        } else if row.admin {
            match connect::step(
                Some(Choice::Forge(row.number)),
                root.forges(),
                connect::playground(root.forge()),
                owned(root),
            ) {
                Shown::Details(_) => admin_details(b, root, w, forge, bindings, row),
                Shown::Ready(_) => status(b, root, w, row),
                Shown::Unfinished(_) | Shown::SettingUp(_) => setting_up(b, root, w, forge, row),
                Shown::Choose => {}
            }
        } else if row.kind == Kind::GitHub && !forge::forges::accepted(row) {
            setting_up(b, root, w, forge, row);
        } else {
            status(b, root, w, row);
            first_line(b, root, w, row);
            if connect::failed(row) {
                progress_in(b, Some(root), w, row);
            }
        }
    } else {
        body(b, root, w, bindings, forge, choice, Place::Welcome, sign_in);
    }
}
pub(crate) fn owned(root: &Domain) -> bool {
    match root.forge().deployment() {
        Some(answer) => answer.owned,
        None => false,
    }
}

fn choose(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    forge: &mut ForgeBindings,
    place: Place,
    sign_in: &crate::forge_sign_in::Forges,
) {
    let kinds = connect::choices(connect::records(root.forges()), connect::playground(root.forge()), owned(root));
    match place {
        Place::Welcome => {
            b.open(Role::Slot);
            b.style(look::FLOW_CHOICES);
            welcome_choices(b, root, w, bindings, forge, &kinds);
            b.close();
        }
        Place::Connect => connect_choices(b, root, w, bindings, forge, &kinds, sign_in),
    }
}
fn connect_choices(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    forge: &mut ForgeBindings,
    kinds: &[connect::Choice],
    sign_in: &crate::forge_sign_in::Forges,
) {
    styled_text(b, Role::Paragraph, &w.say(Word::WelcomeForgeLead, &[]), look::SETTINGS_LEDE);
    if kinds.is_empty() {
        return;
    }
    let rows = connect::records(root.forges());
    let default = connect::default_choice(rows, connect::playground(root.forge()), owned(root));
    b.open(Role::Slot);
    b.style(look::CONNECT_CARDS);
    for (kind, title, description, time) in [
        (connect::Choice::Owned, Word::WelcomeForgejo, Word::WelcomeForgejoDescription, Word::ForgeHereTime),
        (connect::Choice::GitHub, Word::ConnectGitHub, Word::ForgeGitHubDescription, Word::ForgeGitHubTime),
    ] {
        let connected = connect::connected_choice(rows, kind);
        if !kinds.contains(&kind) && connected.is_none() {
            continue;
        }
        let heading = w.say(title, &[]);
        b.open(Role::Region);
        b.style(look::CONNECT_CARD);
        b.name(&heading);
        if default == Some(kind) {
            b.style(look::CONNECT_DEFAULT);
        }
        crate::cards::text(b, Role::Heading { level: 2 }, &heading);
        text(b, w, Role::Paragraph, description);
        if let Some(row) = connected {
            b.open(Role::Paragraph);
            b.style(look::CONNECT_MUTED);
            b.open(Role::Text);
            b.style(look::CONNECT_YES);
            w.text(b, Word::ForgeCardConnected, &[]);
            b.close();
            if let Some((org, name)) = welcome_target(root, row) {
                b.text(Run::Plain, b" \xc2\xb7 ");
                b.text(Run::Plain, &forge::words::full_name(org, name));
            } else if let Some(repository) = row.repositories.first() {
                b.text(Run::Plain, b" \xc2\xb7 ");
                b.text(Run::Plain, &forge::words::full_name(&repository.owner, &repository.name));
            }
            b.close();
        } else {
            styled_text(b, Role::Paragraph, &w.say(time, &[]), look::CONNECT_MUTED);
        }
        b.open(Role::Slot);
        b.style(look::CONNECT_ACTS);
        if let Some(row) = connected {
            super::forges::open_link(b, w, row, sign_in, Some(look::CONNECT_SECONDARY));
        } else {
            match kind {
                connect::Choice::Owned => {
                    let name = w.say(Word::ForgeRunHere, &[]);
                    let to = bindings.insert(Act::Go { address: Place::Connect.address(Some(Choice::Owned)) });
                    flow::connect_control(
                        b,
                        if default == Some(kind) {
                            Control::PrimaryLink { name: &name, to }
                        } else {
                            Control::SecondaryLink { name: &name, to }
                        },
                    );
                }
                connect::Choice::GitHub => {
                    app_control(b, root, w, forge, 0, Word::ForgeConnectGitHub, Some(Place::Connect.address(None)));
                }
                connect::Choice::Managed => {}
            }
        }
        b.close();
        b.close();
    }
    b.close();
    if kinds.contains(&connect::Choice::Managed) {
        b.open(Role::Slot);
        b.style(look::CONNECT_MORE);
        text(b, w, Role::Heading { level: 2 }, Word::ForgeOwnServer);
        b.open(Role::Slot);
        b.style(look::CONNECT_ROWS);
        b.open(Role::Slot);
        b.style(look::CONNECT_ROW);
        b.open(Role::Slot);
        b.open(Role::Text);
        b.run(Run::Strong, &w.say(Word::ConnectManaged, &[]));
        b.close();
        text(b, w, Role::Text, Word::ForgeOwnServerDescription);
        b.close();
        flow::connect_control(
            b,
            Control::SecondaryLink {
                name: &w.say(Word::ForgeOwnServerConnect, &[]),
                to: bindings.insert(Act::Go { address: Place::Connect.address(Some(Choice::Elsewhere)) }),
            },
        );
        b.close();
        b.close();
        b.close();
    }
}
/// The welcome's Forge step: Forgejo's card, the default, whose button opens
/// its sheet, then GitHub's (`temper/docs/design/client/forge/deployment.md`).
fn welcome_choices(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    forge: &mut ForgeBindings,
    kinds: &[connect::Choice],
) {
    let owned = kinds.contains(&connect::Choice::Owned);
    if owned {
        let note = w.say(Word::ForgeHereTime, &[]);
        flow::choice_card(b, &w.say(Word::WelcomeForgejo, &[]));
        text(b, w, Role::Paragraph, Word::WelcomeForgejoDescription);
        b.open(Role::Paragraph);
        b.style(look::FLOW_MUTED);
        b.name(&note);
        b.text(Run::Plain, &note);
        b.close();
        b.open(Role::Slot);
        b.style(look::FLOW_ACTIONS);
        let run = w.say(Word::WelcomeForgejoRun, &[]);
        flow::control(
            b,
            Control::PrimaryLink {
                name: &run,
                to: bindings.insert(Act::Go { address: Place::Welcome.address(Some(Choice::Owned)) }),
            },
        );
        b.close();
        b.close();
    }
    if kinds.contains(&connect::Choice::GitHub) {
        flow::choice_card(b, &w.say(Word::ConnectGitHub, &[]));
        text(b, w, Role::Paragraph, Word::ForgeGitHubDescription);
        let time = w.say(Word::ForgeGitHubTime, &[]);
        b.open(Role::Paragraph);
        b.style(look::FLOW_MUTED);
        b.name(&time);
        b.text(Run::Plain, &time);
        b.close();
        app_control(b, root, w, forge, 0, Word::ForgeConnectGitHub, Some(Place::Welcome.address(None)));
        b.close();
    }
}
/// The welcome's sheet over the Forge step: what Forgejo on this machine
/// starts with, the chosen one's own fields under it, then Cancel and Run.
/// Run sends its choice; copies wait for Ready (`temper/docs/design/client/pages/README.md`).
fn sheet(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, forge: &mut ForgeBindings, place: Place) {
    let draft = root.forge().setup_form();
    b.open(Role::Dialog);
    b.style(look::WELCOME_SHEET);
    b.name(&w.say(Word::ConnectOwned, &[]));
    b.open(Role::Slot);
    b.style(look::WELCOME_SHEET_HEAD);
    text(b, w, Role::Heading { level: 2 }, Word::ConnectOwned);
    text(b, w, Role::Paragraph, Word::WelcomeForgejoLead);
    b.close();
    let start_with = w.say(Word::WelcomeForgejoStartWith, &[]);
    b.open(Role::ChoiceGroup);
    b.style(look::WELCOME_START);
    b.name(&start_with);
    b.open(Role::Heading { level: 3 });
    b.style(look::WELCOME_LEGEND);
    b.name(&start_with);
    b.text(Run::Plain, &start_with);
    b.close();
    for (start, name, detail) in [
        (Start::Empty, Word::WelcomeForgejoEmpty, Word::WelcomeForgejoEmptyDetail),
        (Start::Copies, Word::WelcomeForgejoCopies, Word::WelcomeForgejoCopiesDetail),
        (Start::Nothing, Word::WelcomeForgejoNothing, Word::WelcomeForgejoNothingDetail),
    ] {
        b.open(Role::Slot);
        b.style(look::WELCOME_OPTION);
        b.open(Role::Choice);
        b.name(&w.say(name, &[]));
        if draft.start == start {
            b.state(State::Selected);
        }
        b.bind(forge.insert(Meaning::Forges(forge::forges::Act::Start(start))));
        b.close();
        text(b, w, Role::Paragraph, detail);
        b.close();
        if draft.start != start {
            continue;
        }
        match start {
            Start::Empty => {
                b.open(Role::Slot);
                b.style(look::WELCOME_OPTION_MORE);
                for (field, word) in [
                    (SetupField::Organization, Word::SetupOrganization),
                    (SetupField::Repository, Word::WelcomeForgejoName),
                ] {
                    setup_field(b, root, w, forge, field, word, 0, None, Place::Welcome);
                }
                b.close();
            }
            Start::Copies => {
                let addresses = w.say(Word::WelcomeForgejoAddresses, &[]);
                b.open(Role::Slot);
                b.style(look::WELCOME_OPTION_MORE);
                crate::cards::text(b, Role::Paragraph, &addresses);
                b.open(Role::TextArea);
                b.name(&addresses);
                b.placeholder(&w.say(Word::WelcomeForgejoAddressHint, &[]));
                b.value(&draft.copies, draft.copies_written);
                b.bind(forge.insert(Meaning::Copies));
                b.close();
                b.close();
            }
            Start::Nothing => {}
        }
    }
    b.close();
    sheet_alerts(b, w, draft);
    // A new forge's refusal has no record to name.
    setup_refusal(b, root, w, None);
    b.open(Role::Slot);
    b.style(look::WELCOME_SHEET_FOOT);
    let cancel = w.say(Word::Cancel, &[]);
    b.open(Role::Button);
    b.name(&cancel);
    b.bind(bindings.insert(Act::Go { address: place.address(None) }));
    b.shortcut(Shortcut::ESCAPE);
    b.close();
    let run = w.say(Word::WelcomeForgejoRun, &[]);
    flow::control(
        b,
        if setup_disabled(root) {
            Control::PrimaryDisabled { name: &run }
        } else {
            Control::PrimaryButton { name: &run, to: forge.insert(Meaning::Forges(forge::forges::Act::RunOwned)) }
        },
    );
    b.close();
    b.close();
}
fn sheet_alerts(b: &mut Builder, w: &Words, draft: &forge::forges::Draft) {
    if draft.blank {
        text(b, w, Role::Alert, Word::WelcomeSheetBlank);
    }
    if let Some(refused) = &draft.copies_refused {
        copies_alert(b, w, refused);
    }
}
pub(crate) fn copies_alert(b: &mut Builder, w: &Words, refused: &forge::making::CopiesRefused) {
    use forge::making::CopiesRefused;
    let words = match refused {
        CopiesRefused::NoAddress => w.say(Word::WelcomeCopiesNone, &[]),
        CopiesRefused::TooMany => w.say(Word::WelcomeCopiesTooMany, &[]),
        CopiesRefused::NotAnAddress(line) => {
            let line = forge::words::lossy(line);
            let bound = usize::try_from(w.name_bytes).expect("word bound fits usize");
            let mut chunks = line.utf8_chunks();
            let line = chunks.next().expect("refused line has words").valid();
            let end = line.floor_char_boundary(line.len().min(bound));
            let line = line.get(..end).expect("bounded character prefix");
            w.say(Word::WelcomeCopiesNotAnAddress, &[Hole::Name(copy_of(line.as_bytes()))])
        }
    };
    crate::cards::text(b, Role::Alert, &words);
}
pub(crate) fn unfinished(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut ForgeBindings, row: &Row) {
    flow::card(b, &w.say(Word::ForgeNotFinished, &[]));
    if let Address::ConnectForge { .. } = root.pages().address() {
        b.style(look::CONNECT_PANEL);
    }
    b.open(Role::Paragraph);
    b.text(Run::Plain, b"FORGE \xc2\xb7 ");
    w.text(b, Word::ForgeNotFinished, &[]);
    b.close();
    super::forges::removal_words(b, w, row);
    let removing = match &root.forge().setup_form().removal {
        Some(removal) => removal.forge == row.number && removal.request.is_some(),
        None => false,
    };
    for act in &connect::unfinished_acts(row) {
        let (word, act) = match act {
            connect::UnfinishedAct::GoOn => (
                Word::ForgeGoOn,
                forge::forges::Act::GoOn {
                    forge: row.number,
                    back: temper_client_domain::address::print(
                        &if let Address::Welcome { .. } = root.pages().address() {
                            root.pages().address().clone()
                        } else {
                            Address::Forges
                        },
                    ),
                },
            ),
            connect::UnfinishedAct::StartOver => {
                let end = if let Address::Welcome { .. } = root.pages().address() {
                    forge::forges::StartOverEnd::Welcome
                } else {
                    forge::forges::StartOverEnd::Forges
                };
                (Word::ForgeStartOver, forge::forges::Act::StartOver { forge: row.number, end })
            }
        };
        let name = w.say(word, &[]);
        let control = if removing {
            Control::Disabled { name: &name }
        } else {
            Control::Button { name: &name, to: bindings.insert(Meaning::Forges(act)) }
        };
        flow::control(b, control);
    }
    b.close();
}
fn your_forgejo(root: &Domain, choice: Option<Choice>) -> bool {
    match choice {
        Some(Choice::Forge(number)) => match connect::row(number, root.forges(), connect::playground(root.forge())) {
            Some(row) => row.kind == Kind::Forgejo && (row.admin || row.mode == Mode::Connected),
            None => false,
        },
        None | Some(Choice::Owned | Choice::Managed | Choice::GitHub | Choice::Elsewhere) => false,
    }
}
// Each way owns its controls' identity (skein/docs/design/ui/README.md, 3.4).
fn panel(b: &mut Builder, w: &Words, way: forge::admin::Way, number: Option<u16>) {
    b.open(Role::Slot);
    let way = match way {
        forge::admin::Way::Admin => 0,
        forge::admin::Way::Tokens => 1,
    };
    b.key(skein_ui::Key { owner: look::owner(), kind: 62, a: way, b: u64::from(number.unwrap_or(0)) });
    b.name(&w.say(Word::ConnectManaged, &[]));
    b.style(look::CONNECT_PANEL);
}
fn address_field(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut ForgeBindings,
    field: forge::admin::PlaceField,
    word: Word,
) {
    let draft = &root.forge().admin_form().place;
    b.open(Role::Slot);
    b.open(Role::TextField);
    b.style(look::SETTINGS_TEXT);
    b.name(&w.say(word, &[]));
    let index = field.index();
    b.value(draft.fields.get(index).expect("place field"), *draft.written.get(index).expect("place writes"));
    b.bind(bindings.insert(Meaning::PlaceField(field)));
    b.close();
    b.close();
}
fn addresses(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut ForgeBindings, row: Option<&Row>) {
    use forge::admin::PlaceField;
    if let Some(row) = row {
        b.open(Role::Slot);
        b.open(Role::TextField);
        b.style(look::SETTINGS_TEXT);
        b.name(&w.say(Word::ConnectManagedAddress, &[]));
        b.value(&row.host, 0);
        b.state(State::Disabled);
        b.close();
        b.close();
        return;
    }
    address_field(b, root, w, bindings, PlaceField::Host, Word::ConnectManagedAddress);
    disclosure(b, &w.say(Word::ConnectAddresses, &[]), false);
    address_field(b, root, w, bindings, PlaceField::Git, Word::ConnectManagedGit);
    address_field(b, root, w, bindings, PlaceField::Web, Word::ConnectManagedWeb);
    b.close();
    b.close();
}
fn disclosure(b: &mut Builder, name: &[u8], expanded: bool) {
    b.open(Role::Disclosure);
    b.name(name);
    if expanded {
        b.state(State::Expanded);
    }
    b.open(Role::Slot);
}
fn how(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut ForgeBindings, fixed: Option<forge::admin::Way>) {
    use forge::admin::Way;
    styled_text(b, Role::Paragraph, &w.say(Word::ConnectHow, &[]), look::CONNECT_TITLE);
    b.open(Role::ChoiceGroup);
    b.name(&w.say(Word::ConnectHow, &[]));
    b.style(look::CONNECT_HOW);
    for (way, word, description) in [
        (Way::Admin, Word::ConnectAdminWay, Word::AdminDescription),
        (Way::Tokens, Word::ConnectTokensWay, Word::TokensDescription),
    ] {
        if fixed.is_some() && fixed != Some(way) {
            continue;
        }
        b.open(Role::Slot);
        b.open(Role::Choice);
        b.name(&w.say(word, &[]));
        b.description(&w.say(description, &[]));
        if fixed.unwrap_or(root.forge().admin_form().way) == way {
            b.state(State::Selected);
        }
        if fixed.is_none() {
            b.bind(bindings.insert(Meaning::AdminWay(way)));
        }
        b.close();
        b.close();
    }
    b.close();
}
fn managed(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut ForgeBindings, pages: &mut Bindings) {
    panel(b, w, root.forge().admin_form().way, None);
    addresses(b, root, w, bindings, None);
    how(b, root, w, bindings, None);
    match root.forge().admin_form().way {
        forge::admin::Way::Admin => admin_form(b, root, w, bindings, pages, None),
        forge::admin::Way::Tokens => tokens_form(b, root, w, bindings, pages, None),
    }
    b.close();
}
fn other_forges(b: &mut Builder, root: &Domain, w: &Words, pages: &mut Bindings) {
    let place = if root.pages().welcome().is_some() { Place::Welcome } else { Place::Connect };
    flow::connect_control(
        b,
        Control::Link { name: &w.say(Word::ForgeAll, &[]), to: pages.insert(Act::Go { address: place.address(None) }) },
    );
}
fn github(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut ForgeBindings, row: &Row) {
    flow::details(b, &w.say(Word::ForgeAppTitle, &[]));
    text(b, w, Role::Paragraph, Word::ForgeAppDescription);
    crate::cards::text(b, Role::Paragraph, &w.say(Word::ForgeSetupUrl, &[Hole::Number(u64::from(row.number))]));
    if w.shell == Shell::Terminal {
        crate::cards::text(b, Role::Paragraph, &forge::words::app_terminal(row.number));
    } else {
        let draft = root.forge().app_form();
        for (secret, word, meaning, required) in [
            (false, Word::ForgeAppId, Meaning::AppId, draft.id.is_empty()),
            (true, Word::ForgePrivateKey, Meaning::AppKey, !draft.has_key() && !draft.too_long),
        ] {
            b.open(Role::Slot);
            b.style(look::FLOW_FIELD);
            b.open(Role::TextField);
            b.name(&w.say(word, &[]));
            b.bind(bindings.insert(meaning));
            if secret {
                b.entry(skein_ui::EntryKind::Secret);
                b.value(b"", draft.key_written);
            } else {
                b.value(&draft.id, draft.written);
            }
            if draft.checked && (required || (!secret && draft.number().is_none())) {
                b.state(State::Invalid);
            }
            b.close();
            if draft.checked && required {
                text(b, w, Role::Alert, Word::SetupRequired);
            } else if draft.checked && !secret && draft.number().is_none() {
                crate::cards::text(b, Role::Alert, forge::words::APP_ID);
            }
            b.close();
        }
        app_control(b, root, w, bindings, row.number, Word::ForgeConnectAppButton, None);
        if draft.checked && draft.too_long {
            crate::cards::text(b, Role::Alert, forge::words::APP_TOO_LONG);
        }
        text(b, w, Role::Paragraph, Word::ForgePasted);
    }
    if let Some(line) = forge::words::last_connect(row) {
        crate::cards::text(b, Role::Alert, &line);
    }
    b.close();
}
pub(crate) fn app_control(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut ForgeBindings,
    number: u16,
    word: Word,
    back: Option<Address>,
) {
    // The first installation is primary on Forges too; another account is maintenance.
    // On the welcome, GitHub's card stands beside Forgejo's, the default (client/forge/deployment.md).
    let primary = if word == Word::ForgeConnectGitHub && back == Some(Place::Connect.address(None)) {
        connect::default_choice(connect::records(root.forges()), connect::playground(root.forge()), owned(root))
            == Some(connect::Choice::GitHub)
    } else {
        word != Word::ForgeInstallAnother && back != Some(Place::Welcome.address(None))
    };
    let connect_card = back == Some(Place::Connect.address(None));
    let pending = root.forge().app();
    let busy = match pending {
        Some(pending) => pending.answer.is_none(),
        None => false,
    };
    let name = w.say(word, &[]);
    if busy || root.forge().served() == forge::Served::OutOfDate {
        app_card_control(
            b,
            connect_card,
            if primary { Control::PrimaryDisabled { name: &name } } else { Control::Disabled { name: &name } },
        );
    } else {
        let meaning = match back {
            Some(back) if word == Word::ForgeConnectGitHub => {
                if number == 0 {
                    Meaning::Forges(forge::forges::Act::GitHub {
                        back: temper_client_domain::address::print(&if connect_card { Address::Forges } else { back }),
                    })
                } else {
                    Meaning::ConnectManifest { forge: number, back: if connect_card { Address::Forges } else { back } }
                }
            }
            Some(back) => Meaning::Install { forge: number, back },
            None => Meaning::ConnectApp(number),
        };
        let to = bindings.insert(meaning);
        app_card_control(
            b,
            connect_card,
            if primary { Control::PrimaryButton { name: &name, to } } else { Control::Button { name: &name, to } },
        );
    }
    if let Some(pending) = pending
        && pending.forge == number
        && let Some(answer) = &pending.answer
    {
        if let Some(words) = forge::words::app_refusal(answer.clone(), number) {
            crate::cards::text(b, Role::Alert, &words);
        }
        if let forge::github::Answer::Entrance(refusal) = answer {
            b.open(Role::Alert);
            w.refusal(b, refusal.refusal(), w.name_bytes);
            b.close();
        }
        // A path's departure is never refused, so only a whole address needs a link meanwhile.
        if let forge::github::Answer::Link(address) = answer
            && !address.starts_with(b"/")
        {
            b.open(Role::Link);
            b.name(&name);
            b.text(Run::Plain, &name);
            b.external(address);
            b.style(if connect_card {
                if primary { look::CONNECT_PRIMARY } else { look::CONNECT_SECONDARY }
            } else if primary {
                look::FLOW_PRIMARY
            } else {
                look::FLOW_SECONDARY
            });
            b.close();
        }
    }
}
fn app_card_control(b: &mut Builder, connect: bool, control: Control<'_>) {
    if connect {
        flow::connect_control(b, control);
    } else {
        flow::control(b, control);
    }
}
fn here(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut ForgeBindings, row: &Row) {
    let terminal = row.offer == Offer::NotUnderTheBox;
    flow::details(b, &w.say(if terminal { Word::ForgeTerminal } else { Word::ForgeHere }, &[]));
    if terminal {
        b.open(Role::Paragraph);
        b.text(Run::Plain, &w.say(Word::ForgeTerminalDetails, &[]));
        b.text(Run::Plain, b" ");
        b.text(Run::Plain, &w.say(Word::ForgeTerminalNext, &[]));
        b.close();
    } else {
        if let Some(owned) = &row.owned {
            crate::cards::text(
                b,
                Role::Paragraph,
                &w.say(Word::ForgeHereVersionDetails, &[Hole::Name(copy_of(&owned.forgejo))]),
            );
        } else {
            text(b, w, Role::Paragraph, Word::ForgeHereDetails);
        }
        setup_fields(b, root, w, bindings, row.number);
        b.open(Role::List);
        b.style(look::FLOW_FACTS);
        for note in [Word::ForgeIsolation, Word::ForgeStorage] {
            text(b, w, Role::Item, note);
        }
        b.close();
        setup_control(b, root, w, bindings, row.number, Word::ForgeSetUp);
    }
    b.close();
}
fn code(b: &mut Builder, words: &[u8]) {
    b.open(Role::Paragraph);
    b.style(look::FLOW_CODE);
    b.text(Run::Code, words);
    b.close();
}
/// Configuration keys keep their code face inside the instruction's sentence.
fn instruction(b: &mut Builder, w: &Words, word: Word, key: &[u8]) {
    let words = w.say(word, &[]);
    let start = skein_lib::bytes::find(&words, key).expect("instruction names its configuration key");
    let end = start.checked_add(key.len()).expect("instruction key fits");
    b.open(Role::Paragraph);
    b.name(&words);
    b.text(Run::Plain, words.get(..start).expect("instruction prefix"));
    b.text(Run::Code, key);
    b.text(Run::Plain, words.get(end..).expect("instruction suffix"));
    b.close();
}
fn elsewhere(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    accounts: Option<&mut crate::accounts::Bindings>,
) {
    if let Some(accounts) = accounts {
        b.open(Role::Slot);
        b.style(look::CONNECT_PANEL);
        b.name(&w.say(Word::ForgeElsewhere, &[]));
        instruction(b, w, Word::ForgeConfigLead, b"~/.config/temper/temper.json");
        b.open(Role::List);
        b.ordered(1);
        b.style(look::CONNECT_STEPS);
        let number = u64::from(connect::next_number(root.forges()));
        for (word, snippet, key) in [
            (Word::ForgeConfigStep, Word::ForgeConfigCode, b"application.forges".as_slice()),
            (Word::ForgeIdentityStep, Word::ForgeIdentityCode, b"application.identities".as_slice()),
            (Word::ForgeSetupStep, Word::ForgeSetupCode, b"application.setup".as_slice()),
        ] {
            b.open(Role::Item);
            instruction(b, w, word, key);
            let holes: &[Hole] = if snippet == Word::ForgeIdentityCode {
                &[Hole::Number(number), Hole::Number(number)]
            } else {
                &[Hole::Number(number)]
            };
            flow::command(b, root, w, accounts, &w.say(snippet, holes), true);
            b.close();
        }
        b.open(Role::Item);
        instruction(b, w, Word::ForgeRestart, b"temper box");
        b.close();
        b.close();
        b.open(Role::Slot);
        b.style(look::CONNECT_ACTS);
        flow::connect_control(
            b,
            Control::Link {
                name: &w.say(Word::ForgeAll, &[]),
                to: bindings.insert(Act::Go { address: Place::Connect.address(None) }),
            },
        );
        b.close();
        b.close();
        return;
    }
    flow::details(b, &w.say(Word::ForgeElsewhere, &[]));
    text(b, w, Role::Paragraph, Word::ForgeElsewhereDetails);
    let number = u64::from(connect::next_number(root.forges()));
    for (word, snippet) in [
        (Word::ForgeConfig, Word::ForgeConfigCode),
        (Word::ForgeIdentity, Word::ForgeIdentityCode),
        (Word::ForgeSetupEntry, Word::ForgeSetupCode),
    ] {
        text(b, w, Role::Paragraph, word);
        let holes: &[Hole] = if snippet == Word::ForgeIdentityCode {
            &[Hole::Number(number), Hole::Number(number)]
        } else {
            &[Hole::Number(number)]
        };
        code(b, &w.say(snippet, holes));
    }
    for word in [Word::ForgeBotOnly, Word::ForgeRestart] {
        text(b, w, Role::Paragraph, word);
    }
    b.close();
}
pub(crate) fn setup_control(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut ForgeBindings,
    number: u16,
    word: Word,
) {
    let primary = word == Word::ForgeSetUp;
    let name = w.say(word, &[]);
    if setup_disabled(root) {
        flow::control(
            b,
            if primary { Control::PrimaryDisabled { name: &name } } else { Control::Disabled { name: &name } },
        );
    } else {
        let to = bindings.insert(Meaning::SetUp(number));
        flow::control(
            b,
            if primary { Control::PrimaryButton { name: &name, to } } else { Control::Button { name: &name, to } },
        );
    }
    setup_refusal(b, root, w, Some(number));
}
fn setup_disabled(root: &Domain) -> bool {
    let pending = match root.forge().set_up() {
        Some(last) => last.progress.is_some(),
        None => false,
    };
    pending || root.forge().served() == forge::Served::OutOfDate
}
fn setup_refusal(b: &mut Builder, root: &Domain, w: &Words, number: Option<u16>) {
    if let Some(last) = root.forge().set_up()
        && last.forge == number
        && let Some(answer) = &last.answer
    {
        match answer {
            SetUpAnswer::Outcome(SetUpOutcome::Refused(why)) => {
                crate::cards::text(b, Role::Alert, forge::words::set_up_refusal(*why));
            }
            SetUpAnswer::EntranceRefused(refusal) => {
                b.open(Role::Alert);
                w.refusal(b, refusal.refusal(), w.name_bytes);
                b.close();
            }
            SetUpAnswer::Outcome(SetUpOutcome::SettingUp | SetUpOutcome::Unknown) => {}
        }
    }
}
fn full_setup(b: &mut Builder, root: Option<&Domain>, w: &Words, row: &Row) -> bool {
    let Some(root) = root else { return false };
    let Some(setup) = root.forge().set_up() else { return false };
    if setup.forge != Some(row.number) {
        return false;
    }
    crate::cards::full_progress(b, w, root, setup.request)
}
pub(crate) fn progress_in(b: &mut Builder, root: Option<&Domain>, w: &Words, row: &Row) {
    progress(b, root, w, row, false);
}
pub(crate) fn live(b: &mut Builder, root: &Domain, w: &Words, row: &Row) {
    if row.kind == Kind::Forgejo && row.mode == Mode::Connected {
        flow::live(b, &[(&w.say(Word::TokensChecking, &[]), Mark::UnderWay)]);
        return;
    }
    if row.admin && !connect::failed(row) {
        let making = match welcome_target(root, row) {
            Some((org, repository)) => {
                w.say(Word::ForgeMakingAndAdopting, &[Hole::Name(forge::words::full_name(org, repository))])
            }
            None => w.say(Word::ForgeMakingFirst, &[]),
        };
        let current = if let Connection::Connected { .. } = row.connection { 2 } else { 0 };
        flow::live(
            b,
            &[
                (&w.say(Word::ForgeCheckedAddress, &[]), mark(current, 0)),
                (&w.say(Word::ForgeMadeBot, &[]), mark(current, 1)),
                (&making, mark(current, 2)),
                (&w.say(Word::ForgeGiveAccess, &[]), mark(current, 3)),
            ],
        );
    } else {
        progress(b, Some(root), w, row, true);
    }
}
fn progress(b: &mut Builder, root: Option<&Domain>, w: &Words, row: &Row, in_row: bool) {
    if full_setup(b, root, w, row) {
        return;
    }
    if row.kind == Kind::GitHub {
        let line = forge::words::forge_connection(row);
        let steps = [(line.as_ref(), if forge::forges::accepted(row) { Mark::Done } else { Mark::UnderWay })];
        if in_row {
            flow::live(b, &steps);
        } else {
            flow::progress(b, w, &w.say(Word::ForgeProgress, &[]), &steps);
        }
        return;
    }
    let repository = match row.starts_with {
        forge::forges::StartsWith::Repository => true,
        forge::forges::StartsWith::Copies { .. } | forge::forges::StartsWith::Nothing => false,
    };
    let failed = connect::failed_step(row);
    let failed_index = match failed {
        Some(connect::FailedStep::Fetch) => Some(0),
        Some(connect::FailedStep::Prepare) => Some(1),
        Some(connect::FailedStep::Run) => Some(2),
        Some(connect::FailedStep::Connect) => Some(3),
        Some(connect::FailedStep::Repository) => Some(4),
        None => None,
    };
    let current = if connect::failed(row) {
        failed_index
    } else {
        match row.connection {
            Connection::SettingUp { step: Some(forge::forges::Step::Fetch) } => Some(0),
            Connection::SettingUp { step: Some(forge::forges::Step::Prepare) } => Some(1),
            Connection::SettingUp { step: Some(forge::forges::Step::Run | forge::forges::Step::Source) } => Some(2),
            Connection::Connecting => Some(3),
            Connection::Connected { .. } => Some(match &row.first {
                Some(first) => match first.state {
                    FirstState::Made { .. } => 5,
                    FirstState::Making | FirstState::Failed { .. } => 4,
                },
                None => 4,
            }),
            Connection::SettingUp { step: None }
            | Connection::NotConnected
            | Connection::NotSetUp
            | Connection::SetUpFailed { .. }
            | Connection::Stopped { .. } => None,
        }
    };
    let fetch = if let Some(owned) = &row.owned
        && let Some(runner) = &owned.runner
    {
        w.say(
            if current > Some(0) { Word::ForgeFetchedVersions } else { Word::ForgeFetchVersions },
            &[Hole::Name(copy_of(&owned.forgejo)), Hole::Name(copy_of(runner))],
        )
    } else {
        w.say(if current > Some(0) { Word::ForgeFetched } else { Word::ForgeFetch }, &[])
    };
    let fetch = if let Some(fetched) = row.fetched {
        crate::accounts::words::join(&[&fetch, b" \xc2\xb7 ", &forge::words::fetched_figure(fetched)])
    } else {
        fetch
    };
    let prepare = w.say(Word::ForgePrepare, &[]);
    let run = w.say(run_word(row), &[]);
    let connecting = w.say(Word::ForgeConnectingAsTemper, &[]);
    let target = match &row.first {
        Some(first) => Some((first.org.as_ref(), first.repository.as_ref())),
        None => match root {
            Some(root) => setup_target(root, row),
            None => None,
        },
    };
    let making = match target {
        Some((org, repository)) => {
            w.say(Word::ForgeMakingAndAdopting, &[Hole::Name(forge::words::full_name(org, repository))])
        }
        None => w.say(Word::ForgeMakingFirst, &[]),
    };
    if current.is_none()
        && let Connection::SettingUp { step: None } = row.connection
    {
        text(b, w, Role::Status, Word::FlowUnderWay);
    }
    let steps = [
        (fetch.as_ref(), stage_mark(current, failed_index, 0)),
        (prepare.as_ref(), stage_mark(current, failed_index, 1)),
        (run.as_ref(), stage_mark(current, failed_index, 2)),
        (connecting.as_ref(), stage_mark(current, failed_index, 3)),
        (making.as_ref(), stage_mark(current, failed_index, 4)),
    ];
    let steps = if repository { steps.as_slice() } else { steps.get(..4).expect("four forge steps") };
    if in_row {
        flow::live(b, steps);
    } else {
        flow::progress(b, w, &w.say(Word::ForgeProgress, &[]), steps);
    }
    if connect::failed(row) {
        failure(b, w, row);
    }
}
fn run_word(row: &Row) -> Word {
    if let Connection::SettingUp { step: Some(forge::forges::Step::Source) } = row.connection {
        Word::ForgeSource
    } else {
        Word::ForgeRun
    }
}
fn stage_mark(current: Option<usize>, failed: Option<usize>, index: usize) -> Mark {
    if failed == Some(index) {
        return Mark::Failed;
    }
    match current {
        Some(current) => mark(current, index),
        None => Mark::ToCome,
    }
}
fn setup_target<'a>(root: &'a Domain, row: &Row) -> Option<(&'a [u8], &'a [u8])> {
    match root.forge().set_up() {
        Some(last) if last.forge == Some(row.number) => match &last.first {
            forge::forges::SetUpFirst::Empty(first) => Some((first.org(), first.repository())),
            forge::forges::SetUpFirst::Copies(_) | forge::forges::SetUpFirst::Nothing => None,
        },
        Some(_) | None => None,
    }
}

fn mark(current: usize, index: usize) -> Mark {
    match index.cmp(&current) {
        core::cmp::Ordering::Less => Mark::Done,
        core::cmp::Ordering::Equal => Mark::UnderWay,
        core::cmp::Ordering::Greater => Mark::ToCome,
    }
}
pub(crate) fn setting_up(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut ForgeBindings, row: &Row) {
    if row.kind == Kind::Forgejo && row.mode == Mode::Connected {
        flow::card(b, &w.say(Word::TokensConnecting, &[Hole::Number(u64::from(row.number))]));
        text(b, w, Role::Paragraph, Word::TokensChecking);
        b.close();
        return;
    }
    if row.admin {
        flow::card(b, &w.say(Word::AdminConnecting, &[Hole::Number(u64::from(row.number))]));
        let repository = match &row.first {
            Some(first) => forge::words::full_name(&first.org, &first.repository),
            None => {
                let draft = root.forge().admin_form();
                forge::words::full_name(&draft.fields[0], &draft.fields[1])
            }
        };
        crate::cards::text(b, Role::Paragraph, &w.say(Word::AdminMaking, &[Hole::Name(repository)]));
        b.close();
        return;
    }
    if row.kind == Kind::GitHub {
        if forge::forges::slug(row).is_some() {
            flow::details(b, &w.say(Word::ForgeInstallTitle, &[]));
            if row.app == Some(forge::forges::AppWay::Manifest) {
                let slug = forge::forges::slug(row).expect("connected App slug");
                crate::cards::text(
                    b,
                    Role::Paragraph,
                    &w.say(Word::ForgeManifestInstall, &[Hole::Name(copy_of(slug))]),
                );
            } else {
                crate::cards::text(b, Role::Paragraph, &forge::words::app_install(row));
            }
            app_control(b, root, w, bindings, row.number, Word::ForgeInstall, Some(root.pages().address().clone()));
        } else {
            flow::details(b, &w.say(Word::ForgeConnecting, &[]));
            text(b, w, Role::Paragraph, Word::ForgeLeave);
        }
        if let Some(line) = forge::words::last_connect(row) {
            crate::cards::text(b, Role::Alert, &line);
        }
        b.close();
        return;
    }
    flow::details(b, &w.say(Word::ForgeSettingUpTitle, &[]));
    first_line(b, root, w, row);
    progress_in(b, Some(root), w, row);
    if connect::failed(row) {
        setup_ending(b, root, w, bindings, row);
    } else {
        text(b, w, Role::Paragraph, Word::ForgeLeave);
    }
    b.close();
}
pub(crate) fn setup_ending(b: &mut Builder, root: &Domain, w: &Words, forge: &mut ForgeBindings, row: &Row) {
    if connect::failed(row) {
        setup_control(b, root, w, forge, row.number, Word::ForgeAgain);
    }
}
pub(crate) fn adopt(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings) {
    if let Some(project) = root.pages().project() {
        link(b, w, bindings, Word::ForgeAdopt, Address::Adopt { project });
    }
}
pub(crate) fn status(b: &mut Builder, root: &Domain, w: &Words, row: &Row) {
    crate::cards::text(b, Role::Paragraph, &status_words(root, w, row));
}
/// "FORGE · CONNECTION", as Forges and Ready's checklist say it.
pub(crate) fn status_words(root: &Domain, w: &Words, row: &Row) -> Box<[u8]> {
    w.say(Word::ForgeStatus, &[Hole::Name(name(w, row, root)), Hole::Name(forge::words::forge_connection(row))])
}
/// First repository facts, with the pending setup as fallback (client/pages/README.md).
pub(crate) fn welcome_target<'a>(root: &'a Domain, row: &'a Row) -> Option<(&'a [u8], &'a [u8])> {
    if let Some(first) = &row.first {
        return Some((&first.org, &first.repository));
    }
    setup_target(root, row)
}
/// The Forge step's line in the column (client/pages/README.md, The Forge step with a forge).
fn forge_status(b: &mut Builder, root: &Domain, w: &Words, row: &Row, sign_in: &crate::forge_sign_in::Forges) {
    let done = !connect::underway(row);
    let forge = name(w, row, root);
    let detail = if done { forge_done(w, row) } else { forge::words::forge_connection(row) };
    let line = crate::accounts::words::join(&[&forge, " · ".as_bytes(), &detail]);
    b.open(Role::Slot);
    b.style(look::WELCOME_FORGE_STATUS);
    if done {
        b.style(look::FLOW_DONE);
    }
    b.open(Role::Paragraph);
    b.name(&line);
    match w.shell {
        Shell::Browser => {
            b.open(Role::Slot);
            if done {
                b.style(look::WELCOME_OK);
                b.text(Run::Plain, "✓".as_bytes());
            } else {
                b.style(look::WELCOME_SPIN);
            }
            b.close();
            b.open(Role::Text);
            b.run(Run::Strong, &forge);
            b.close();
            b.open(Role::Text);
            b.style(look::WELCOME_SEP);
            b.run(Run::Plain, " · ".as_bytes());
            b.close();
            b.open(Role::Text);
            b.style(look::FLOW_MUTED);
            if done
                && row.kind == Kind::Forgejo
                && let Connection::Connected { login, .. } = &row.connection
                && let Some(first) = &row.first
                && let FirstState::Made { .. } = first.state
                && super::repository::address(row, sign_in, &first.org, &first.repository).is_some()
            {
                w.text(b, Word::WelcomeForgeDoneDetail, &[Hole::Name(copy_of(login))]);
                b.text(Run::Plain, b" \xc2\xb7 ");
                super::repository::name(b, w, Some(row), sign_in, &first.org, &first.repository);
                b.text(Run::Plain, b" made");
            } else {
                b.run(Run::Plain, &detail);
            }
            b.close();
        }
        Shell::Terminal => {
            let words = if done { crate::accounts::words::join(&["✓ ".as_bytes(), &line]) } else { line };
            b.text(Run::Plain, &words);
        }
    }
    b.close();
    if !done && row.kind != Kind::GitHub {
        each_step(b, root, w, row);
    }
    b.close();
}
fn forge_done(w: &Words, row: &Row) -> Box<[u8]> {
    if row.kind == Kind::GitHub {
        for installation in &row.installations {
            if !installation.stray && !installation.suspended {
                let account = match &installation.account {
                    Some(account) => copy_of(&account.login),
                    None => forge::words::installation_words(installation),
                };
                return w.say(Word::WelcomeForgeGitHubApp, &[Hole::Name(account)]);
            }
        }
    }
    let Connection::Connected { login, .. } = &row.connection else { return forge::words::forge_connection(row) };
    let login = w.say(Word::WelcomeForgeDoneDetail, &[Hole::Name(copy_of(login))]);
    if let Some(first) = &row.first
        && let FirstState::Made { .. } = first.state
    {
        let repository = w.say(
            Word::WelcomeForgeDoneRepository,
            &[Hole::Name(forge::words::full_name(&first.org, &first.repository))],
        );
        return crate::accounts::words::join(&[&login, " · ".as_bytes(), &repository]);
    }
    login
}
/// The welcome's Model step says what Forge did (client/pages/README.md).
pub(crate) fn welcome_line(b: &mut Builder, root: &Domain, w: &Words, row: &Row) {
    let forge = name(w, row, root);
    let connection = forge::words::forge_connection(row);
    let detail = forge::words::forge_connection_bare(row);
    if temper_client_domain::pages::welcome::forge_connected(row) && !connect::underway(row) && !connect::failed(row) {
        if let Connection::Connected { login, .. } = &row.connection
            && let Some((org, repository)) = welcome_target(root, row)
        {
            let login = w.say(Word::WelcomeForgeDoneDetail, &[Hole::Name(copy_of(login))]);
            let repository =
                w.say(Word::WelcomeForgeDoneRepository, &[Hole::Name(forge::words::full_name(org, repository))]);
            let line = crate::accounts::words::join(&[&forge, b" \xc2\xb7 ", &login, b" \xc2\xb7 ", &repository]);
            stacked(b, w, &line, &forge, &[&login, &repository], Mark::Done);
        } else {
            let line = w.say(Word::ForgeStatus, &[Hole::Name(forge.clone()), Hole::Name(connection.clone())]);
            stacked(b, w, &line, &forge, &[&connection], Mark::Done);
        }
        return;
    }
    if let Some((org, repository)) = welcome_target(root, row) {
        let target = forge::words::full_name(org, repository);
        let line = w.say(
            Word::WelcomeForgeLine,
            &[Hole::Name(forge.clone()), Hole::Name(target.clone()), Hole::Name(connection.clone())],
        );
        stacked(b, w, &line, &forge, &[&target, &detail], Mark::ToCome);
    } else {
        let line = w.say(Word::ForgeStatus, &[Hole::Name(forge.clone()), Hole::Name(connection.clone())]);
        stacked(b, w, &line, &forge, &[&detail], Mark::ToCome);
    }
    if busy(row) && w.shell == Shell::Browser {
        b.open(Role::Progress);
        if let Some(fetched) = row.fetched {
            b.style(look::FLOW_METER);
            b.name(forge::words::fetching_name(fetched));
            b.run(Run::Plain, &forge::words::fetched_figure(fetched));
            if let Some(total) = fetched.total {
                b.fraction(Fraction { done: fetched.done, total });
            }
        } else {
            b.style(look::FLOW_BUSY);
            b.name(&forge::words::forge_connection(row));
        }
        b.close();
    }
    each_step(b, root, w, row);
}
fn busy(row: &Row) -> bool {
    connect::underway(row) && row.kind != Kind::GitHub
}
/// The forge's line (client/pages/README.md): one paragraph named by its words. The browser
/// stacks its name and details by style; the terminal has neither, so says them on a line.
pub(crate) fn stacked(b: &mut Builder, w: &Words, line: &[u8], forge: &[u8], details: &[&[u8]], mark: Mark) {
    b.open(Role::Paragraph);
    b.name(line);
    let done = match mark {
        Mark::Done => {
            b.style(look::FLOW_DONE);
            true
        }
        Mark::Stopped | Mark::Failed => {
            b.style(look::FLOW_FAILED);
            false
        }
        Mark::UnderWay | Mark::ToCome => false,
    };
    match w.shell {
        Shell::Browser => {
            b.open(Role::Text);
            b.run(Run::Strong, forge);
            b.close();
            for detail in details {
                b.open(Role::Text);
                b.style(look::FLOW_MUTED);
                b.run(Run::Plain, detail);
                b.close();
            }
        }
        Shell::Terminal => {
            let words = if done { crate::accounts::words::join(&["✓ ".as_bytes(), line]) } else { copy_of(line) };
            b.text(Run::Plain, &words);
        }
    }
    b.close();
}
pub(crate) fn each_step(b: &mut Builder, root: &Domain, w: &Words, row: &Row) {
    b.open(Role::Disclosure);
    b.name(&w.say(Word::ForgeProgress, &[]));
    progress_in(b, Some(root), w, row);
    b.close();
}

fn setup_fields(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut ForgeBindings, number: u16) {
    b.open(Role::Slot);
    b.style(look::FLOW_PAIR);
    for (field, word) in
        [(SetupField::Organization, Word::SetupOrganization), (SetupField::Repository, Word::SetupRepository)]
    {
        setup_field(b, root, w, bindings, field, word, number, Some(look::FLOW_FIELD), Place::Connect);
    }
    b.close();
}
#[expect(clippy::too_many_arguments, reason = "the set up's form and the welcome's sheet share one field")]
fn setup_field(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut ForgeBindings,
    field: SetupField,
    word: Word,
    number: u16,
    style: Option<skein_ui::Style>,
    place: Place,
) {
    let draft = root.forge().setup_form();
    let invalid = match root.forge().set_up() {
        Some(last) if last.forge == Some(number) => {
            last.answer == Some(SetUpAnswer::Outcome(SetUpOutcome::Refused(forge::forges::SetUpRefusal::Invalid)))
        }
        Some(_) | None => false,
    };
    b.open(Role::Slot);
    if let Some(style) = style {
        b.style(style);
    }
    b.open(Role::TextField);
    b.name(&w.say(word, &[]));
    b.value(draft.text(field), draft.written(field));
    b.bind(bindings.insert(Meaning::SetupField(field)));
    if draft.required(field) || invalid {
        b.state(State::Invalid);
    }
    b.close();
    match place {
        Place::Connect => {
            if draft.required(field) {
                text(b, w, Role::Alert, Word::SetupRequired);
            }
        }
        Place::Welcome => {}
    }
    b.close();
}
pub(crate) fn first_line(b: &mut Builder, root: &Domain, w: &Words, row: &Row) {
    if let Some((org, repository)) = welcome_target(root, row) {
        flow::subtitle(b, &w.say(Word::SetupInto, &[Hole::Name(forge::words::full_name(org, repository))]));
    }
}
pub(crate) fn app_line(b: &mut Builder, w: &Words, row: &Row, setting: bool) {
    if let Some(slug) = forge::forges::slug(row) {
        b.open(Role::Slot);
        if setting {
            b.style(look::SETTINGS_SETTING);
        }
        text(b, w, Role::Term, Word::ForgeAppLabel);
        b.open(Role::Description);
        crate::cards::text(b, Role::Slot, &forge::words::app_head(slug, row.installations.is_empty()));
        for (index, installation) in row.installations.iter().enumerate() {
            crate::cards::text(b, Role::Slot, &forge::words::app_installation(installation, index == 0));
        }
        b.close();
        b.close();
    }
}

fn failure(b: &mut Builder, w: &Words, row: &Row) {
    b.open(Role::Alert);
    b.style(look::FLOW_FAILED);
    let word = match connect::failed_step(row) {
        Some(connect::FailedStep::Repository) => {
            let words = forge::words::first_repository(row.first.as_ref().expect("failed first repository"));
            b.name(&words);
            b.text(Run::Plain, &words);
            b.close();
            return;
        }
        Some(connect::FailedStep::Fetch) => Word::ForgeFetchFailed,
        Some(connect::FailedStep::Prepare) => Word::ForgePrepareFailed,
        Some(connect::FailedStep::Run) => Word::ForgeRunFailed,
        Some(connect::FailedStep::Connect) => Word::ForgeConnectFailed,
        None => Word::ForgeSetupFailed,
    };
    let why = match &row.connection {
        Connection::SetUpFailed { why, .. } => forge::words::lossy(why),
        Connection::NotConnected
        | Connection::NotSetUp
        | Connection::SettingUp { .. }
        | Connection::Connecting
        | Connection::Connected { .. }
        | Connection::Stopped { .. } => {
            b.close();
            return;
        }
    };
    let words = w.say(word, &[Hole::Name(why)]);
    b.name(&words);
    b.text(Run::Plain, &words);
    b.close();
}

pub(crate) fn admin_details(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut ForgeBindings,
    pages: &mut Bindings,
    row: &Row,
) {
    panel(b, w, forge::admin::Way::Admin, Some(row.number));
    addresses(b, root, w, bindings, Some(row));
    how(b, root, w, bindings, Some(forge::admin::Way::Admin));
    admin_stop(b, w, row);
    if let Some(pending) = root.forge().admin()
        && pending.forge == row.number
        && let Some(answer) = &pending.answer
        && *answer != forge::github::Answer::Connecting
    {
        if let Some(line) = forge::words::app_refusal(answer.clone(), row.number) {
            crate::cards::text(b, Role::Alert, &line);
        }
        text(b, w, Role::Paragraph, Word::AdminKept);
    }
    admin_form(b, root, w, bindings, pages, Some(row.number));
    b.close();
}
fn admin_form(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut ForgeBindings,
    pages: &mut Bindings,
    number: Option<u16>,
) {
    let draft = root.forge().admin_form();
    b.open(Role::Slot);
    b.open(Role::TextField);
    b.style(look::SETTINGS_TEXT);
    b.name(&w.say(Word::AdminToken, &[]));
    b.entry(skein_ui::EntryKind::Secret);
    b.value(b"", draft.token_written);
    b.bind(bindings.insert(Meaning::AdminToken));
    if draft.checked && !draft.has_token() {
        b.state(State::Invalid);
    }
    b.close();
    if draft.checked && draft.too_long {
        text(b, w, Role::Alert, Word::AdminTooLong);
    } else if draft.checked && !draft.has_token() {
        text(b, w, Role::Alert, Word::SetupRequired);
    }
    styled_text(b, Role::Paragraph, &w.say(Word::ConnectAdminScopes, &[]), look::CONNECT_MUTED);
    b.close();
    admin_fields(b, root, w, bindings, number);
    b.open(Role::Slot);
    b.style(look::CONNECT_ACTS);
    let name = w.say(Word::AdminTitle, &[]);
    let control = match number {
        Some(number) => Control::PrimaryButton { name: &name, to: bindings.insert(Meaning::ConnectAdmin(number)) },
        None => match managed_place(&draft.place) {
            Some(place) => Control::PrimaryButton {
                name: &name,
                to: bindings.insert(Meaning::Forges(forge::forges::Act::Managed { place })),
            },
            None => Control::PrimaryDisabled { name: &name },
        },
    };
    flow::connect_control(b, control);
    other_forges(b, root, w, pages);
    b.close();
    if number.is_none()
        && let Some(pending) = root.forge().admin()
        && let Some(answer) = &pending.answer
    {
        if let Some(line) = forge::words::app_refusal(answer.clone(), pending.forge) {
            crate::cards::text(b, Role::Alert, &line);
        }
        if let forge::github::Answer::Entrance(refusal) = answer {
            b.open(Role::Alert);
            w.refusal(b, refusal.refusal(), w.name_bytes);
            b.close();
        }
    }
}
fn admin_fields(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut ForgeBindings, number: Option<u16>) {
    let draft = root.forge().admin_form();
    b.open(Role::Slot);
    b.style(look::CONNECT_PAIR);
    let named_bot = if let Some(pending) = root.forge().admin()
        && (number.is_none() || number == Some(pending.forge))
        && pending.answer
            == Some(forge::github::Answer::Refused(temper_client_protocol_wire::v1::ForgeRefusal::FirstIsBot))
    {
        true
    } else {
        false
    };
    for (field, word) in [
        (forge::admin::Field::Organization, Word::SetupOrganization),
        (forge::admin::Field::Repository, Word::ConnectFirst),
        (forge::admin::Field::Label, Word::AdminLabel),
    ] {
        if field == forge::admin::Field::Label {
            b.close();
            disclosure(
                b,
                &w.say(
                    Word::ConnectRunner,
                    &[Hole::Name(draft.fields.get(field.index()).expect("runner label").clone())],
                ),
                draft.checked && !draft.valid_label(),
            );
        }
        let index = field.index();
        let label = field == forge::admin::Field::Label;
        let invalid =
            if label { !draft.valid_label() } else { draft.fields.get(index).expect("three fields").is_empty() };
        b.open(Role::Slot);
        b.open(Role::TextField);
        b.style(look::SETTINGS_TEXT);
        b.name(&w.say(word, &[]));
        b.value(draft.fields.get(index).expect("three fields"), *draft.written.get(index).expect("three writes"));
        b.bind(bindings.insert(Meaning::AdminField(field)));
        if (draft.checked && invalid) || (field == forge::admin::Field::Organization && named_bot) {
            b.state(State::Invalid);
        }
        b.close();
        if draft.checked && invalid {
            text(b, w, Role::Alert, if label { Word::AdminLabelInvalid } else { Word::SetupRequired });
        }
        if label {
            styled_text(b, Role::Paragraph, &w.say(Word::AdminLabelHint, &[]), look::CONNECT_MUTED);
        }
        b.close();
    }
    b.close();
    b.close();
}
fn admin_stage(stage: forge::forges::SetupStage) -> &'static [u8] {
    use forge::forges::SetupStage;
    match stage {
        SetupStage::Admin => b"checking the admin's token",
        SetupStage::Bot => b"making the bot",
        SetupStage::Token => b"making the bot's token",
        SetupStage::Org => b"making the organization",
        SetupStage::Repository => b"making the repository",
        SetupStage::File => b"adding the starter workflow",
        SetupStage::App => b"making the App",
        SetupStage::Installation => b"installing the App",
        SetupStage::State => b"saving the state",
    }
}
fn admin_stop(b: &mut Builder, w: &Words, row: &Row) {
    use forge::forges::AdminStop;
    if let Some(first) = &row.first
        && let FirstState::Failed { step } = first.state
    {
        let why = match &row.ended {
            Some(why) => copy_of(why),
            None => copy_of(b"Forgejo stopped the request"),
        };
        let step = match step {
            forge::forges::CreateStep::Admin => b"checking the admin's token".as_slice(),
            forge::forges::CreateStep::Org => b"its organization",
            forge::forges::CreateStep::Repository => b"its repository",
            forge::forges::CreateStep::File => b"its starter workflow",
        };
        crate::cards::text(
            b,
            Role::Alert,
            &w.say(
                Word::AdminFirstFailed,
                &[
                    Hole::Name(forge::words::full_name(&first.org, &first.repository)),
                    Hole::Name(copy_of(step)),
                    Hole::Name(why),
                ],
            ),
        );
    } else if let Some(stop) = row.ended_stop {
        match stop {
            AdminStop::NotAdmin => text(b, w, Role::Alert, Word::AdminNotAdmin),
            AdminStop::Refused { status: 401, .. } => text(b, w, Role::Alert, Word::AdminUnauthorized),
            AdminStop::Refused { status: 403, stage } => {
                crate::cards::text(
                    b,
                    Role::Alert,
                    &w.say(Word::AdminScope, &[Hole::Name(copy_of(admin_stage(stage)))]),
                );
            }
            AdminStop::Unmarked => text(b, w, Role::Alert, Word::AdminUnmarked),
            AdminStop::Refused { .. } | AdminStop::Other => {
                if let Some(line) = &row.ended {
                    crate::cards::text(b, Role::Alert, line);
                }
            }
        }
    } else {
        return;
    }
    text(b, w, Role::Paragraph, Word::AdminKept);
}
/// The address field supplies the managed connection's public endpoint (temper/docs/design/client/forge/deployment.md).
fn managed_place(draft: &forge::admin::PlaceDraft) -> Option<forge::forges::Managed> {
    use forge::admin::PlaceField;
    let address = draft.fields.get(PlaceField::Host.index())?.as_ref();
    let (authority, secure) = match address.strip_prefix(b"http://") {
        Some(authority) => (authority, false),
        None => (address.strip_prefix(b"https://").unwrap_or(address), true),
    };
    let authority = authority.strip_suffix(b"/").unwrap_or(authority);
    if authority.is_empty() {
        return None;
    }
    for byte in authority {
        if byte.is_ascii_whitespace() || b"/?#@".contains(byte) {
            return None;
        }
    }
    let (host, port) = if authority.starts_with(b"[") {
        let end = skein_lib::bytes::find(authority, b"]")?;
        let host = authority.get(1..end)?;
        let rest = authority.get(end.checked_add(1)?..)?;
        (host, if rest.is_empty() { None } else { Some(rest.strip_prefix(b":")?) })
    } else if let Some(colon) = skein_lib::bytes::find(authority, b":") {
        (authority.get(..colon)?, Some(authority.get(colon.checked_add(1)?..)?))
    } else {
        (authority, None)
    };
    if host.is_empty() {
        return None;
    }
    let port = match port {
        Some(port) => {
            let port = u16::try_from(skein_ui::number(port)?).ok()?;
            if port == 0 {
                return None;
            }
            port
        }
        None => {
            if secure {
                443
            } else {
                80
            }
        }
    };
    let web = draft.fields.get(PlaceField::Web.index())?;
    Some(forge::forges::Managed {
        host: copy_of(host),
        port,
        tls: if secure { Some(copy_of(host)) } else { None },
        git: draft.fields.get(PlaceField::Git.index())?.clone(),
        web: if web.is_empty() { None } else { Some(web.clone()) },
    })
}

pub(crate) fn tokens_details(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut ForgeBindings,
    pages: &mut Bindings,
    row: &Row,
) {
    panel(b, w, forge::admin::Way::Tokens, Some(row.number));
    addresses(b, root, w, bindings, Some(row));
    how(b, root, w, bindings, Some(forge::admin::Way::Tokens));
    tokens_form(b, root, w, bindings, pages, Some(row));
    b.close();
}
fn tokens_form(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut ForgeBindings,
    pages: &mut Bindings,
    row: Option<&Row>,
) {
    use forge::tokens::Field;
    let draft = root.forge().tokens_form();
    for (field, word) in
        [(Field::Api, Word::TokensApi), (Field::Git, Word::TokensGit), (Field::Webhook, Word::TokensWebhook)]
    {
        if field == Field::Webhook {
            disclosure(b, &w.say(Word::ConnectWebhook, &[]), draft.checked && draft.too_long);
        }
        b.open(Role::Slot);
        let required = field != Field::Webhook && !draft.has(field);
        b.open(Role::TextField);
        b.style(look::SETTINGS_TEXT);
        b.name(&w.say(word, &[]));
        b.entry(skein_ui::EntryKind::Secret);
        b.value(b"", *draft.written.get(field.index()).expect("three secret writes"));
        b.bind(bindings.insert(Meaning::ForgejoToken(field)));
        if draft.checked && (required || draft.too_long) {
            b.state(State::Invalid);
        }
        b.close();
        if draft.checked && required && !draft.too_long {
            text(b, w, Role::Alert, Word::SetupRequired);
        }
        match field {
            Field::Api => styled_text(b, Role::Paragraph, &w.say(Word::ConnectApiScopes, &[]), look::CONNECT_MUTED),
            Field::Git => styled_text(b, Role::Paragraph, &w.say(Word::ConnectGitScopes, &[]), look::CONNECT_MUTED),
            Field::Webhook => {}
        }
        b.close();
        if field == Field::Webhook {
            b.close();
            b.close();
        }
    }
    if draft.checked && draft.too_long {
        text(b, w, Role::Alert, Word::AdminTooLong);
    }
    let title = w.say(Word::TokensConnect, &[]);
    let pending = root.forge().tokens();
    let busy = match pending {
        Some(pending) => pending.answer.is_none(),
        None => false,
    };
    let control = if busy || root.forge().served() == forge::Served::OutOfDate {
        Control::PrimaryDisabled { name: &title }
    } else if let Some(row) = row {
        Control::PrimaryButton { name: &title, to: bindings.insert(Meaning::ConnectTokens(row.number)) }
    } else if let Some(place) = managed_place(&root.forge().admin_form().place) {
        Control::PrimaryButton {
            name: &title,
            to: bindings.insert(Meaning::Forges(forge::forges::Act::Connected { place })),
        }
    } else {
        Control::PrimaryDisabled { name: &title }
    };
    b.open(Role::Slot);
    b.style(look::CONNECT_ACTS);
    flow::connect_control(b, control);
    other_forges(b, root, w, pages);
    b.close();
    tokens_answers(b, root, w, row);
}
fn tokens_answers(b: &mut Builder, root: &Domain, w: &Words, row: Option<&Row>) {
    let pending = root.forge().tokens();
    if let Some(pending) = pending
        && pending_matches(row, pending.forge)
        && let Some(answer) = &pending.answer
    {
        use forge::github::Answer;
        use temper_client_protocol_wire::v1::ForgeRefusal as F;
        match answer {
            Answer::Refused(F::Connected) => crate::cards::text(
                b,
                Role::Alert,
                &w.say(Word::TokensConnected, &[Hole::Number(u64::from(pending.forge))]),
            ),
            Answer::Refused(F::NotTaken) => crate::cards::text(
                b,
                Role::Alert,
                &w.say(Word::TokensNotTaken, &[Hole::Number(u64::from(pending.forge))]),
            ),
            Answer::Refused(F::NoKey) => text(b, w, Role::Alert, Word::TokensMissing),
            Answer::Unknown => text(b, w, Role::Alert, Word::TokensUnknown),
            Answer::Entrance(refusal) => {
                b.open(Role::Alert);
                w.refusal(b, refusal.refusal(), w.name_bytes);
                b.close();
            }
            Answer::Connecting
            | Answer::Link(_)
            | Answer::Page(_)
            | Answer::NoAccess
            | Answer::NotFound
            | Answer::Refused(F::Role | F::NoSuchForge | F::Busy | F::FirstIsBot | F::NotConnected) => {
                if let Some(line) = forge::words::app_refusal(answer.clone(), pending.forge) {
                    crate::cards::text(b, Role::Alert, &line);
                }
            }
        }
    }
    if let Some(row) = row
        && let Some(line) = forge::words::last_connect(row)
    {
        crate::cards::text(b, Role::Alert, &line);
    }
}

fn pending_matches(row: Option<&Row>, number: u16) -> bool {
    match row {
        Some(row) => row.number == number,
        None => true,
    }
}

//! Deployment's Workers tab (temper/docs/design/client/pages/workers.md, 1).
use super::settings::{self, Tab};
use crate::{
    bindings::Bindings,
    look,
    words::{Shell, TimeForm, Times, Word, Words},
};
use alloc::boxed::Box;
use jig_client::{Outcome, contract as c};
use skein_ui::{Builder, Role, Run, State, Style, phrases::Hole};
use temper_client_domain::{
    Domain,
    pages::{
        Act,
        builders::{self, Read},
    },
};

pub fn main(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, project: u32, times: &mut Times) {
    settings::main(b);
    settings::open(b, root, w, bindings, project, Tab::Builders);
    if !settings::missing(b, root, w, bindings, project) {
        let read = match root.pages().builders() {
            Some(page) => page.read(root.client()),
            None => Read::Unknown,
        };
        match read {
            Read::NoRole => crate::cards::text(b, Role::Paragraph, &w.say(Word::BuildersNoRole, &[])),
            Read::Unknown => {
                settings::section_head(b, &w.say(Word::SettingsBuildersTab, &[]));
                b.close();
                b.close();
            }
            Read::Hosts(deployment) => workers(b, root, deployment, w, bindings, times),
        }
    }
    b.close();
    b.close();
}
fn owner(root: &Domain) -> bool {
    match root.pages().me(root.client()) {
        Some(me) => me.deployment_role == Some(c::HeldRole::Owner),
        None => false,
    }
}
fn workers(
    b: &mut Builder,
    root: &Domain,
    deployment: &c::Deployment,
    w: &Words,
    bindings: &mut Bindings,
    times: &mut Times,
) {
    let mut count = 0_u64;
    for worker in &deployment.worker_settings {
        if !worker.withdrawn {
            count = count.checked_add(1).expect("document bounds workers");
        }
    }
    settings::section_head(b, &w.say(Word::BuildersCount, &[Hole::Number(count)]));
    b.close();
    if count == 0 {
        text(b, Role::Paragraph, &w.say(Word::BuildersNone, &[]), look::SETTINGS_EMPTY);
    } else {
        b.open(Role::Slot);
        b.style(look::SETTINGS_LINES);
        for worker in &deployment.worker_settings {
            if !worker.withdrawn {
                row(b, root, deployment, worker, w, bindings, times);
            }
        }
        b.close();
    }
    if deployment.cut {
        text(b, Role::Paragraph, &w.say(Word::BuildersCut, &[]), look::SETTINGS_FACTS);
    }
    b.close();
    if owner(root) {
        add(b, root, deployment, w, bindings);
    }
    text(b, Role::Paragraph, &w.say(Word::BuildersFacts, &[]), look::SETTINGS_FACTS);
}
fn row(
    b: &mut Builder,
    root: &Domain,
    deployment: &c::Deployment,
    worker: &c::WorkerState,
    w: &Words,
    bindings: &mut Bindings,
    times: &mut Times,
) {
    let terminal = w.shell == Shell::Terminal;
    let mut used = 0;
    let mut draining = false;
    for host in &deployment.hosts {
        if host.connected && host.name.as_deref() == Some(worker.name.as_ref()) {
            used = host.used;
            draining = host.draining;
            break;
        }
    }
    let places = if worker.places == 1 {
        w.say(Word::WorkerPlace, &[])
    } else {
        w.say(Word::WorkerPlaces, &[Hole::Number(u64::from(worker.places))])
    };
    let runs = match used {
        0 => w.say(Word::WorkerNoRuns, &[]),
        1 => w.say(Word::WorkerRun, &[]),
        used => w.say(Word::WorkerRuns, &[Hole::Number(u64::from(used))]),
    };
    let digest = match worker.digest {
        Some(digest) => {
            let mut hex = skein_lib::Writer::new(8);
            for byte in digest.iter().take(4) {
                for nibble in [byte.checked_div(16).expect("nonzero divisor"), byte % 16] {
                    let digit = if nibble < 10 {
                        b'0'.checked_add(nibble)
                    } else {
                        b'a'.checked_add(nibble.saturating_sub(10))
                    }
                    .expect("hex digit fits");
                    hex.put(&[digit]).expect("eight digest digits");
                }
            }
            w.say(Word::WorkerDigest, &[Hole::Name(hex.finish())])
        }
        None => w.say(Word::WorkerInBox, &[]),
    };
    let detail = crate::accounts::words::join(&[&places, " · ".as_bytes(), &runs, " · ".as_bytes(), &digest]);
    let when = if draining {
        w.say(Word::BuilderDraining, &[])
    } else if let Some(since) = worker.since {
        w.say(
            if worker.linked { Word::WorkerUpSince } else { Word::WorkerDownSince },
            &[Hole::Name(times.say(w, TimeForm::Clock, since))],
        )
    } else {
        w.say(if worker.linked { Word::BuilderConnected } else { Word::BuilderNotConnected }, &[])
    };
    b.open(Role::Slot);
    b.open(if terminal { Role::Paragraph } else { Role::Slot });
    b.style(look::SETTINGS_SETTING);
    b.open(Role::Slot);
    crate::cards::text(b, Role::Text, &worker.name);
    if terminal {
        b.text(Run::Plain, b" ");
    }
    text(b, Role::Text, &detail, look::SETTINGS_DETAIL);
    b.close();
    if terminal {
        b.text(Run::Plain, b" ");
    }
    text(b, Role::Text, &when, look::SETTINGS_WHEN);
    b.close();
    if owner(root)
        && let Some(page) = root.pages().builders()
    {
        b.open(Role::Slot);
        b.style(look::MODELS_ADD);
        field(
            b,
            root,
            bindings,
            &w.say(Word::WorkerPlacesFor, &[Hole::Name(worker.name.clone())]),
            &[],
            &page.places(worker),
            builders::Act::Places { number: worker.number, value: Box::default() },
        );
        button(b, w, bindings, Word::WorkerChange, builders::Act::Change(worker.number), !page.can_change(worker));
        if worker.digest.is_some() {
            button(b, w, bindings, Word::WorkerRemove, builders::Act::Remove(worker.number), page.busy());
        }
        b.close();
        if let Some(outcome) = page.refused(Some(worker.number)) {
            refusal(b, w, outcome, &worker.name);
        }
    }
    b.close();
}
fn add(b: &mut Builder, root: &Domain, deployment: &c::Deployment, w: &Words, bindings: &mut Bindings) {
    let Some(page) = root.pages().builders() else { return };
    settings::section_head(b, &w.say(Word::WorkerAddHeading, &[]));
    b.close();
    b.open(Role::Slot);
    b.style(look::MODELS_ADD);
    field(
        b,
        root,
        bindings,
        &w.say(Word::WorkerName, &[]),
        &w.say(Word::WorkerNameHint, &[]),
        page.name(),
        builders::Act::Name(Box::default()),
    );
    field(
        b,
        root,
        bindings,
        &w.say(Word::WorkerPlacesField, &[]),
        &[],
        page.add_places(),
        builders::Act::AddPlaces(Box::default()),
    );
    field(
        b,
        root,
        bindings,
        &w.say(Word::WorkerSecretDigest, &[]),
        &w.say(Word::WorkerDigestHint, &[]),
        page.digest(),
        builders::Act::Digest(Box::default()),
    );
    button(b, w, bindings, Word::WorkerAdd, builders::Act::Add, !page.can_add(deployment));
    b.close();
    text(b, Role::Text, &w.say(Word::WorkerAddHelp, &[]), look::SETTINGS_DETAIL);
    if page.duplicate(deployment) {
        text(b, Role::Text, &w.say(Word::WorkerDuplicate, &[Hole::Name(page.name().into())]), look::SETTINGS_REFUSAL);
    }
    if let Some(outcome) = page.refused(None) {
        refusal(b, w, outcome, page.name());
    }
    b.close();
}
fn field(
    b: &mut Builder,
    root: &Domain,
    bindings: &mut Bindings,
    name: &[u8],
    hint: &[u8],
    value: &[u8],
    act: builders::Act,
) {
    b.open(Role::TextField);
    b.style(look::SETTINGS_TEXT);
    b.state(State::SubmitsOnEnter);
    b.name(name);
    b.placeholder(hint);
    b.value(value, temper_client_domain::pages::shown(root.pages()));
    b.bind(bindings.insert(Act::Builders(act)));
    if let Some(page) = root.pages().builders()
        && page.busy()
    {
        b.state(State::Disabled);
    }
    b.close();
}
fn button(b: &mut Builder, w: &Words, bindings: &mut Bindings, word: Word, act: builders::Act, disabled: bool) {
    b.open(Role::Button);
    b.style(look::SETTINGS_SMALL);
    b.name(&w.say(word, &[]));
    b.bind(bindings.insert(Act::Builders(act)));
    if disabled {
        b.state(State::Disabled);
    }
    b.close();
}
fn refusal(b: &mut Builder, w: &Words, outcome: &Outcome, name: &[u8]) {
    b.open(Role::Slot);
    b.style(look::SETTINGS_REFUSAL);
    if let Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused {
        refusal:
            c::Refusal::Malformed {
                why: c::Malformed::Settings(c::MalformedSettings { fault: c::SettingFault::Linked, .. }),
                ..
            },
        ..
    })) = outcome
    {
        crate::cards::text(b, Role::Paragraph, &w.say(Word::WorkerLinked, &[Hole::Name(name.into())]));
    } else {
        crate::confirm::outcome_words(b, w, outcome);
    }
    b.close();
}
fn text(b: &mut Builder, role: Role, name: &[u8], style: Style) {
    b.open(role);
    b.style(style);
    b.name(name);
    b.text(Run::Plain, name);
    b.close();
}

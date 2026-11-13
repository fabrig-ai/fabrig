//! Deployment's Workers tab (temper/docs/design/client/pages/README.md, Workers).
use super::settings::{self, Tab};
use crate::{
    bindings::Bindings,
    look,
    words::{Word, Words},
};
use jig_client::contract as c;
use skein_ui::{Builder, Role, Style, phrases::Hole};
use temper_client_domain::{Domain, pages::builders::Read};

pub fn main(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, project: u32) {
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
            Read::Hosts(deployment) => hosts(b, deployment, w),
        }
    }
    b.close();
    b.close();
}
fn hosts(b: &mut Builder, deployment: &c::Deployment, w: &Words) {
    settings::section_head(b, &w.say(Word::BuildersCount, &[Hole::Number(u64::from(deployment.hosts.len()))]));
    b.close();
    let mut connected = false;
    for host in &deployment.hosts {
        connected |= host.connected;
    }
    if !connected {
        text(b, Role::Paragraph, &w.say(Word::BuildersNone, &[]), look::SETTINGS_EMPTY);
    }
    if !deployment.hosts.is_empty() {
        b.open(Role::Slot);
        b.style(look::SETTINGS_LINES);
        for host in &deployment.hosts {
            b.open(Role::Slot);
            b.open(Role::Slot);
            let name = match &host.name {
                Some(name) => name.clone(),
                None => w.say(Word::BuilderUnnamed, &[Hole::Number(host.channel)]),
            };
            crate::cards::text(b, Role::Text, &name);
            let slots = if host.slots == 1 { Word::BuildersSlot } else { Word::BuildersSlots };
            text(
                b,
                Role::Text,
                &w.say(slots, &[Hole::Number(u64::from(host.slots)), Hole::Number(u64::from(host.used))]),
                look::SETTINGS_DETAIL,
            );
            b.close();
            let state = if !host.connected {
                Word::BuilderNotConnected
            } else if host.draining {
                Word::BuilderDraining
            } else {
                Word::BuilderConnected
            };
            text(b, Role::Text, &w.say(state, &[]), look::SETTINGS_WHEN);
            b.close();
        }
        b.close();
    }
    if deployment.cut {
        text(b, Role::Paragraph, &w.say(Word::BuildersCut, &[]), look::SETTINGS_FACTS);
    }
    text(b, Role::Paragraph, &w.say(Word::BuildersFacts, &[]), look::SETTINGS_FACTS);
    b.close();
}
fn text(b: &mut Builder, role: Role, name: &[u8], style: Style) {
    b.open(role);
    b.style(style);
    b.name(name);
    b.text(skein_ui::Run::Plain, name);
    b.close();
}

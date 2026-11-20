//! A page waits while another tab does its part (client/forge/deployment.md, GitHub's way, in a new tab).
use crate::{
    accounts::words::{self, Word as W},
    look,
    words::{Shell, Words},
};
use alloc::boxed::Box;
use skein_ui::{Builder, Role, Run, phrases::Hole};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Form {
    Welcome,
    Connection,
}
#[derive(Debug)]
pub enum Target {
    External { address: Box<[u8]> },
    Bound { address: Box<[u8]>, binding: skein_ui::Binding },
    Button { binding: skein_ui::Binding },
}
#[derive(Debug)]
pub struct Note {
    pub title: Box<[u8]>,
    pub body: Box<[u8]>,
    pub link: Box<[u8]>,
    pub address: Option<Target>,
    pub form: Form,
    pub refused: bool,
}

pub fn waiting(b: &mut Builder, w: &Words, note: &Note, text_per_node: u32) {
    match note.form {
        Form::Welcome => {
            b.open(Role::Slot);
            b.style(look::WELCOME_AWAY);
            if w.shell == Shell::Browser {
                b.open(Role::Slot);
                b.style(look::WELCOME_SPIN);
                b.close();
            }
            b.open(Role::Slot);
        }
        Form::Connection => {}
    }
    b.open(Role::Status);
    if note.form == Form::Connection {
        b.style(look::CONNECT_WAITING);
    }
    b.name(&note.title);
    b.open(Role::Text);
    b.run(Run::Strong, &note.title);
    b.close();
    if note.form == Form::Connection {
        b.open(Role::Text);
        b.text(Run::Plain, &note.body);
        b.close();
    }
    b.close();
    match note.form {
        Form::Welcome => {
            b.open(Role::Paragraph);
            b.style(look::FLOW_MUTED);
            b.name(&note.body);
            b.text(Run::Plain, &note.body);
            b.close();
        }
        Form::Connection => {
            b.open(Role::Slot);
            b.style(look::CONNECT_ACTS);
        }
    }
    if let Some(address) = &note.address {
        link(b, w, note, address, text_per_node);
    }
    b.close();
    if note.form == Form::Welcome {
        b.close();
    }
}

fn link(b: &mut Builder, w: &Words, note: &Note, target: &Target, text_per_node: u32) {
    let (address, safe) = match target {
        Target::External { address } => (address.as_ref(), skein_ui::markdown::safe_url(address)),
        // A bound link's caller supplies its checked href; the tree holds only the binding.
        Target::Bound { address, .. } => (address.as_ref(), true),
        Target::Button { .. } => (b"".as_slice(), true),
    };
    if !safe {
        crate::cards::text(b, Role::Alert, &words::say(w, W::UnsafeAddress, &[]));
        return;
    }
    let bytes = skein_lib::len_u64(note.link.len()).saturating_add(skein_lib::len_u64(address.len()));
    if bytes > u64::from(text_per_node) {
        crate::cards::text(
            b,
            Role::Alert,
            &words::say(w, W::AddressLong, &[Hole::Number(skein_lib::len_u64(address.len()))]),
        );
        return;
    }
    let role = match target {
        Target::External { .. } | Target::Bound { .. } => Role::Link,
        Target::Button { .. } => Role::Button,
    };
    b.open(role);
    match note.form {
        Form::Welcome if w.shell == Shell::Terminal => {}
        Form::Welcome => b.style(if note.refused { look::FLOW_PRIMARY } else { look::WELCOME_AGAIN }),
        Form::Connection => b.style(if note.refused { look::CONNECT_PRIMARY } else { look::CONNECT_SECONDARY }),
    }
    if note.form == Form::Connection || w.shell == Shell::Browser || role == Role::Button {
        b.name(if note.refused { &note.link } else { note.link.strip_suffix(b" \xe2\x80\xba").unwrap_or(&note.link) });
    }
    b.text(Run::Plain, &note.link);
    match target {
        Target::External { .. } => b.external(address),
        Target::Bound { binding, .. } => b.link(*binding),
        Target::Button { binding } => b.bind(*binding),
    }
    b.close();
}

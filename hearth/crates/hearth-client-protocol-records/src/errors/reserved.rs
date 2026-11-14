use hearth_domain_render_vocabulary::errors::{Reserved, Rule};
//! Closed reserved rules (hearth/docs/design/protocol/format/tasks.md, 11; hearth/docs/design/domain/machine/boot.md, 3.2).
use alloc::boxed::Box;


use crate::form::{json::field, text::Lines};
use skein_json::writer::Encoder;

pub(super) const fn rule_name(item: Rule) -> &'static [u8] {
    match item {
        Rule::ActivatorKey => b"activator_key",
        Rule::RootKey => b"root_key",
        Rule::RootWait => b"rootwait",
        Rule::CmdlineLength { .. } => b"cmdline_length",
        Rule::PasswordPreseed => b"password_preseed",
        Rule::TrialName => b"trial_name",
        Rule::Overlay => b"overlay",
        Rule::TmpfilesSubvolume => b"tmpfiles_subvolume",
        Rule::TmpfilesWrite => b"tmpfiles_write",
        Rule::TmpfilesPath => b"tmpfiles_path",
        Rule::ContainerModule => b"container_module",
        Rule::ContainerParam => b"container_param",
        Rule::ActivatorFile => b"activator_file",
        Rule::Dropin => b"dropin",
        Rule::Accounts => b"accounts",
        Rule::SecretPath => b"secret_path",
        Rule::HearthState => b"hearth_state",
        Rule::PackagesLayer => b"packages_layer",
        Rule::ClassMount => b"class_mount",
        Rule::DataRoot => b"data_root",
        Rule::Runtime => b"runtime",
        Rule::LogFile => b"log_file",
        Rule::InitramfsConf => b"initramfs_conf",
        Rule::SecretMode => b"secret_mode",
        Rule::HearthOwner => b"hearth_owner",
        Rule::LiveNone => b"live_none",
        Rule::NeverLiveUnit => b"never_live_unit",
        Rule::NeverLivePath => b"never_live_path",
        Rule::HearthContainer => b"hearth_container",
        Rule::HearthNull => b"hearth_null",
    }
}


pub(super) fn reserved_text(item: &Reserved, lines: &mut Lines) {
    if item.rule == Rule::SecretPath {
        lines.put(&item.what);
        lines.put(b" is a secret's path; declare it under secrets (protocol/format/README.md, 7.1)");
        return;
    }
    lines.quote(&item.what);
    lines.put(b" is reserved: ");
    match item.rule {
        Rule::ActivatorKey => lines.put(b"the activator owns this command-line key (domain/machine/boot.md, 3.2)"),
        Rule::RootKey => lines.put(b"it changes how tiny-initramfs finds or starts the root (domain/machine/boot.md, 3.2)"),
        Rule::RootWait => lines.put(b"tiny-initramfs would wait for ever for a missing root (domain/machine/boot.md, 3.2)"),
        Rule::CmdlineLength { bytes } => {
            lines.put(b"the command line is ");
            lines.decimal(u64::from(bytes));
            lines.put(b" bytes, past 2048 with the activator's tokens and NUL (domain/machine/boot.md, 3.2)");
        }
        Rule::PasswordPreseed => {
            lines.put(b"a password preseed leaves its value in /var/cache/debconf/passwords.dat (protocol/format/tasks.md, 11)");
        }
        Rule::TrialName => lines.put(b"hearth owns trial and check units (domain/slots/prepare.md, 5)"),
        Rule::Overlay => lines.put(b"extensions would merge over hearth's live overlays (protocol/format/tasks.md, 11)"),
        Rule::TmpfilesSubvolume => lines.put(b"a nested subvolume breaks a layer (protocol/format/tasks.md, 11)"),
        Rule::TmpfilesWrite => lines
            .put(b"a tmpfiles write outside /var, /run or /tmp would fail on the read-only root (protocol/format/tasks.md, 11)"),
        Rule::TmpfilesPath => lines.put(b"a tmpfiles path holds no glob or specifier (protocol/format/tasks.md, 11)"),
        Rule::ContainerModule => lines.put(b"a container has no initramfs module (domain/render/spec.md, 1.2)"),
        Rule::ContainerParam => lines.put(b"a container has no boot parameter (domain/render/spec.md, 1.2)"),
        Rule::ActivatorFile => lines.put(b"the activator writes this path at prepare (protocol/format/tasks.md, 11)"),
        Rule::Dropin => lines.put(b"the watchdog's drop-in must sort last (domain/slots/prepare.md, 5)"),
        Rule::Accounts => lines.put(b"accounts and machine identity are written at prepare (protocol/format/tasks.md, 11)"),
        Rule::SecretPath => lines.put(b""),
        Rule::HearthState => lines.put(b"hearth owns its state and store (domain/machine/README.md, 3.4)"),
        Rule::PackagesLayer => lines.put(b"the packages layer owns /usr outside /usr/local (protocol/format/tasks.md, 5)"),
        Rule::ClassMount => lines.put(b"the class mount hides this path (protocol/format/tasks.md, 7)"),
        Rule::DataRoot => lines.put(b"a data path must be strictly beneath /srv, /var or /home (protocol/format/tasks.md, 7)"),
        Rule::Runtime => lines.put(b"runtime, noise and history are not task files (protocol/format/tasks.md, 11)"),
        Rule::LogFile => lines.put(b"a file beneath /var/log must be empty (protocol/format/tasks.md, 11)"),
        Rule::InitramfsConf => {
            lines.put(b"the packages part writes tiny-initramfs configuration (domain/machine/boot.md, 4.1)");
        }
        Rule::SecretMode => {
            lines.put(b"a secret or identity mode gives others nothing and has no special bits (protocol/format/README.md, 7.1)");
        }
        Rule::HearthOwner => {
            lines.put(b"hearth must not be able to read this secret or identity file (protocol/format/README.md, 7.1)");
        }
        Rule::LiveNone => lines.put(b"action none names no unit (protocol/format/tasks.md, 14)"),
        Rule::NeverLiveUnit => lines.put(b"this core service must not restart live (protocol/format/tasks.md, 14)"),
        Rule::NeverLivePath => lines.put(b"this path must not change live (protocol/format/tasks.md, 14)"),
        Rule::HearthContainer => lines.put(b"a container has no hearth task (protocol/format/tasks.md, 12)"),
        Rule::HearthNull => lines.put(b"the hearth task cannot be removed (protocol/format/tasks.md, 12)"),
    }
}
pub(super) fn reserved_json(item: &Reserved, json: &mut Encoder) {
    json.object_start();
    if let Rule::CmdlineLength { bytes } = item.rule {
        json.key(b"bytes");
        json.unsigned(u64::from(bytes));
    }
    field(json, b"rule", rule_name(item.rule));
    field(json, b"what", &item.what);
    json.object_end();
}


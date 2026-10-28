//! Supplied configured-worker observations beside account facts.
use super::{Cut, accounts::Window};
use jig_client::contract as c;
use skein_json::writer::Encoder;

pub(super) fn hosts(
    hosts: &skein_lib::List<c::OwnerProgramHost>,
    out: &mut Encoder,
    cut: &mut Cut,
    window: &mut Window,
) {
    out.key(b"program_hosts");
    out.array_start();
    for host in hosts {
        if !window.take() {
            break;
        }
        out.object_start();
        out.key(b"name");
        super::offers::whole(&host.name, out, cut, window);
        out.key(b"channel");
        out.unsigned(host.channel);
        out.key(b"connected");
        out.boolean(host.connected);
        out.key(b"awaiting_logins");
        out.boolean(host.awaiting_logins);
        programs(&host.programs, out, cut, window);
        out.key(b"cut");
        out.boolean(host.cut);
        window.cut |= host.cut;
        out.object_end();
    }
    out.array_end();
}
fn failure(reason: c::OwnerProgramFailure, detail: &[u8], out: &mut Encoder, cut: &mut Cut) {
    out.object_start();
    out.key(b"failed");
    out.string(match reason {
        c::OwnerProgramFailure::Missing => b"missing",
        c::OwnerProgramFailure::Deadline => b"deadline",
        c::OwnerProgramFailure::Unreadable => b"unreadable",
        c::OwnerProgramFailure::Exited => b"exited",
        c::OwnerProgramFailure::Unconfigured => b"unconfigured",
        c::OwnerProgramFailure::Unknown => b"unknown",
        c::OwnerProgramFailure::Closed => b"closed",
    });
    out.key(b"detail");
    cut.words(detail, out);
    out.object_end();
}

fn programs(values: &skein_lib::List<c::OwnerProgramLogin>, out: &mut Encoder, cut: &mut Cut, window: &mut Window) {
    out.key(b"programs");
    out.array_start();
    for program in values {
        if !window.take() {
            break;
        }
        out.object_start();
        out.key(b"name");
        super::offers::whole(&program.name, out, cut, window);
        out.key(b"state");
        match &program.state {
            None => out.null(),
            Some(c::OwnerProgramState::Usable) => out.string(b"usable"),
            Some(c::OwnerProgramState::NotSignedIn) => out.string(b"not_signed_in"),
            Some(c::OwnerProgramState::SignedInOtherwise { method }) => {
                out.object_start();
                out.key(b"signed_in_otherwise");
                cut.words(method, out);
                out.object_end();
            }
            Some(c::OwnerProgramState::Failed { reason, detail }) => failure(*reason, detail, out, cut),
        }
        out.key(b"sign_out");
        match &program.sign_out {
            None => out.null(),
            Some(c::OwnerProgramSignOut::SignedOut) => out.string(b"signed_out"),
            Some(c::OwnerProgramSignOut::Failed { reason, detail }) => failure(*reason, detail, out, cut),
        }
        out.key(b"cleanup");
        out.boolean(program.cleanup);
        out.key(b"cut");
        out.boolean(program.cut);
        window.cut |= program.cut;
        out.object_end();
    }
    out.array_end();
}

pub(super) fn deployment_hosts(deployment: &c::OwnerDeployment, out: &mut Encoder, cut: &mut Cut, window: &mut Window) {
    out.key(b"program_hosts");
    out.array_start();
    for host in &deployment.hosts {
        if !window.take() {
            break;
        }
        out.object_start();
        out.key(b"name");
        match &host.name {
            Some(name) => super::offers::whole(name, out, cut, window),
            None => out.null(),
        }
        out.key(b"channel");
        out.unsigned(host.channel);
        out.key(b"connected");
        out.boolean(host.connected);
        out.key(b"awaiting_logins");
        out.boolean(host.awaiting_logins);
        programs(&host.programs, out, cut, window);
        out.key(b"cut");
        out.boolean(host.cut);
        window.cut |= host.cut;
        out.object_end();
    }
    out.array_end();
}

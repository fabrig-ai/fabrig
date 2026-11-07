//! A native client's ask for an approval (client/README.md, 7.1).
use crate::{Domain, Limits, Request};
use jig_client::ApprovalCode;
use skein_lib::{Env, Queue, Wall};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Asking,
    Asked { code: ApprovalCode, expires: Wall },
    NotAsked(jig_client::NotAsked),
}
pub(crate) fn ask(root: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    if root.client.link().state != jig_client::Link::SignedOut || root.asked == Some(State::Asking) {
        return;
    }
    root.asked = Some(State::Asking);
    crate::approve::changed(root);
    crate::route::client(root, env, jig_client::Event::AskApproval, out);
}
pub(crate) fn answered(root: &mut Domain, env: &Env<Limits>, answer: jig_client::Approval, out: &mut Queue<Request>) {
    if root.asked != Some(State::Asking) {
        return;
    }
    match answer {
        jig_client::Approval::Asked { secret, code, expires } => {
            root.asked = Some(State::Asked { code, expires });
            crate::approve::changed(root);
            if !root.unlinked {
                crate::route::client(
                    root,
                    env,
                    jig_client::Event::SignIn { with: jig_client::With::Approval { secret } },
                    out,
                );
            }
        }
        jig_client::Approval::NotAsked(why) => {
            root.asked = Some(State::NotAsked(why));
            crate::approve::changed(root);
        }
        jig_client::Approval::Read { .. } | jig_client::Approval::Decided { .. } => {}
    }
}
pub(crate) fn progress(root: &mut Domain) {
    if root.asked.is_some() && root.client.link().state != jig_client::Link::SignedOut {
        root.asked = None;
        crate::approve::changed(root);
    }
}

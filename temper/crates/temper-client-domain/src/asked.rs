//! A native client's ask for an approval (client/README.md, 4.1).
use crate::{Domain, Limits, Request};
use jig_client::ApprovalCode;
use skein_lib::{Env, Queue, Secret, Wall};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Asking,
    Asked { code: ApprovalCode, expires: Wall, port: Option<u16> },
    NotAsked(jig_client::NotAsked),
}
pub(crate) fn ask(root: &mut Domain, env: &Env<Limits>, port: Option<u16>, out: &mut Queue<Request>) {
    if root.client.link().state != jig_client::Link::SignedOut || root.asked == Some(State::Asking) {
        return;
    }
    root.asked_secret = None;
    root.asked_port = port;
    root.asked = Some(State::Asking);
    crate::approve::changed(root);
    crate::route::client(root, env, jig_client::Event::AskApproval { port }, out);
}
pub(crate) fn answered(root: &mut Domain, env: &Env<Limits>, answer: jig_client::Approval, out: &mut Queue<Request>) {
    if root.asked != Some(State::Asking) {
        return;
    }
    match answer {
        jig_client::Approval::Asked { secret, code, expires } => {
            let port = root.asked_port.take();
            root.asked = Some(State::Asked { code, expires, port });
            crate::approve::changed(root);
            if !root.unlinked {
                if port.is_some() {
                    root.asked_secret = Some(secret.clone());
                }
                crate::route::client(
                    root,
                    env,
                    jig_client::Event::SignIn { with: jig_client::With::Approval { secret, grant: None } },
                    out,
                );
            }
        }
        jig_client::Approval::NotAsked(why) => {
            root.asked_port = None;
            root.asked = Some(State::NotAsked(why));
            crate::approve::changed(root);
        }
        jig_client::Approval::Read { .. } | jig_client::Approval::Decided { .. } => {}
    }
}
pub(crate) fn granted(root: &mut Domain, env: &Env<Limits>, grant: Secret, out: &mut Queue<Request>) {
    if root.unlinked || root.client.link().state != jig_client::Link::SignedOut {
        return;
    }
    match root.asked {
        Some(State::Asked { port: Some(_), .. }) => {}
        None | Some(State::Asking | State::NotAsked(_) | State::Asked { port: None, .. }) => return,
    }
    if let Some(secret) = &root.asked_secret {
        let secret = secret.clone();
        crate::route::client(
            root,
            env,
            jig_client::Event::SignIn { with: jig_client::With::Approval { secret, grant: Some(grant) } },
            out,
        );
    }
}
pub(crate) fn progress(root: &mut Domain) {
    if root.asked.is_some() && root.client.link().state != jig_client::Link::SignedOut {
        root.asked_secret = None;
        root.asked_port = None;
        root.asked = None;
        crate::approve::changed(root);
    }
}

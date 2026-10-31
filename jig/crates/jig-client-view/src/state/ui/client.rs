//! The UI reads the application-held client through its lent reference (view.md, 18.1).
use crate::{Config, Request, Ui, state::intents};
use jig_client::{Client, Progress, Readout, Told, contract};
pub(crate) mod outcome;
use skein_lib::{Env, Queue, Token};
pub(crate) fn told(
    domain: &mut Ui,
    env: &Env<Config>,
    client: &Client,
    told: Told,
    salt: u64,
    out: &mut Queue<Request>,
) {
    match told {
        Told::Changed { handle } => {
            super::changed_handle(domain, env, client, handle, out);
        }
        Told::Pending { key } => {
            let Some(pending) = client.pending(key) else { return };
            let progress = match pending.progress {
                Progress::InFlight => intents::Progress::InFlight,
                Progress::Retrying { why: jig_client::Retry::Full { held, room_at } } => {
                    intents::Progress::Full { held, room_at }
                }
                Progress::Retrying { .. } => intents::Progress::Retrying,
                Progress::Parked | Progress::NotServed => intents::Progress::Parked,
                Progress::Unknown => intents::Progress::Unknown,
            };
            intents::progress(domain, Token::new(key.number ^ salt), progress, out, client);
        }
        Told::Answered { key, outcome } => outcome::answered(domain, env, client, key, outcome, salt, out),
        Told::Kept { key, .. } => {
            let Some(pending) = client.pending(key) else { return };
            if domain.kept_requests.insert(key, pending.party).is_err() {
                crate::state::page::routing::refuse(domain, contract::Limit::Requests);
            }
            super::refresh_kept_page(domain, env, out, client);
        }
        Told::Link { link: state } => {
            domain.set_link(state);
            match state {
                jig_client::Link::Starting => {
                    if domain.frame_watch.is_none() {
                        domain.frame_watch =
                            domain.hold_watch(contract::Subject::Me, crate::state::handles::Holder::Me, out);
                    }
                }
                jig_client::Link::Offline | jig_client::Link::Live => {}
                jig_client::Link::SignedOut => super::signed_out(domain, out, client),
                jig_client::Link::OutOfDate => {
                    crate::state::page::routing::leave_page(domain, out, client);
                    domain.blocked = true;
                    domain.confirmed = None;
                    domain.page = crate::state::page::Page::OutOfDate;
                    domain.leave_handles(true, out);
                    domain.changed();
                }
            }
            if client.link().sign_out_lost {
                crate::state::signin::sign_out_lost(domain);
            }
        }
        Told::Ended { handle, why } => {
            super::ended_handle(domain, handle, why, out);
        }
        Told::Page { handle, page } => {
            let read = Token::new(handle.number);
            let result = match page {
                Readout::Jig(result) => result,
                Readout::Limit => contract::ReadResult::None(contract::ReadNone::Busy),
                Readout::NotServed { .. } | Readout::Family { .. } => {
                    contract::ReadResult::None(contract::ReadNone::Refused)
                }
            };
            if crate::state::objects::historical_read(&result).is_some()
                || crate::state::objects::is_card_read(domain, read, client)
            {
                crate::state::objects::read_card(domain, env, read, result, out, client);
            } else {
                crate::state::page::routing::read_result(domain, env, read, result, out, client);
            }
        }
        Told::Text { handle } => {
            crate::state::activity::routing::text(domain, env, handle, out, client);
        }
        Told::Dropped { count } => {
            crate::tree::notices::kept::show(domain, crate::tree::notices::kept::Kind::Dropped { count });
        }
    }
}

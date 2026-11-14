//! Sign-in admission and durable credential completion.
use super::{Class, Credential, OneShot, Parsed, Shell, Signing, arguments, limits};
impl OneShot {
    pub(super) fn signing_start(&mut self) {
        use temper_client_tools::jig::call::{Value, Values};
        if self.class != Class::SignIn {
            return;
        }
        if self.signing.is_none() && self.directory.ready() && self.input.is_none() {
            if self.directory.link.is_none()
                && let Some(Parsed::Call { call, json }) = self.call.take()
            {
                match call.values.first() {
                    Some(Some(Values::One(Value::Stdin))) => {
                        if call.form == 1 {
                            self.fail(b"sign-in --service needs a file, not stdin");
                            return;
                        }
                        if self.stdin_ended.is_some() {
                            self.directory.link = Some(Box::new([]));
                        } else {
                            self.input = Some(Vec::new());
                            self.read_input();
                        }
                    }
                    Some(Some(Values::One(Value::File(path)))) => {
                        if call.form == 1 {
                            self.directory.load_service(path);
                        } else {
                            self.directory.load_link(path);
                        }
                    }
                    _ => {
                        self.fail(b"a sign-in file is required");
                        return;
                    }
                }
                self.service_sign_in = (call.form == 1).then_some(());
                let Parsed::Call { call, .. } = arguments::parse(&[b"me"]).expect("me call") else { unreachable!() };
                self.call = Some(Parsed::Call { call, json });
            }
            if let Some(bytes) = self.directory.link.take() {
                let start = if self.service_sign_in.is_some() { Signing::service } else { Signing::new };
                match start(
                    &bytes,
                    self.directory.parent(),
                    self.directory.leaf(),
                    self.startup,
                    &mut self.submit,
                    self.connections,
                    self.env.limits,
                ) {
                    Ok(signing) => {
                        self.directory.address = signing.address.as_ref().map(|a| a[..a.len() - 1].into());
                        self.signing = Some(signing);
                        self.until = Some(self.env.now.saturating_add(limits::DEADLINE));
                    }
                    Err(error) => self.fail(error),
                }
            }
        }
        let Some(signing) = &mut self.signing else { return };
        if signing.error {
            self.fail(b"the sign-in files could not be read");
            return;
        }
        if signing.service.is_none() && signing.held.is_some() {
            if self.directory.party_loaded.is_some() {
                let party = self.directory.party.map_or_else(String::new, |n| format!(" as party {n}"));
                self.fail(format!("DIRECTORY is signed in{party}; this link is not spent: sign out first, temper ctl DIRECTORY sign-out, or sign in to another directory").as_bytes());
            }
            return;
        }
        if signing.with.is_some()
            && let Some((held, records)) = signing.shell.as_mut().and_then(Shell::loaded)
        {
            if signing.service.is_none() && held.is_some() {
                signing.held = held;
                self.directory.load_party();
                return;
            }
            let secret = signing.with.take().expect("sign-in secret");
            let with = if signing.service.is_some() {
                jig_client::With::Service(secret)
            } else {
                jig_client::With::Link { secret, held: None }
            };
            signing.held.clone_from(&held);
            let mut kept = skein_lib::List::with_capacity(u32::try_from(records.len()).expect("bounded records"));
            for record in records {
                kept.push(record).expect("record room");
            }
            self.root_event(jig_client::Event::Start { kept });
            self.root_event(jig_client::Event::SignIn { with });
        }
    }
    pub(super) fn signing_retry(&mut self) {
        let Some(signing) = &mut self.signing else { return };
        if signing.retry.is_some_and(|at| self.env.now >= at) && self.until.is_some_and(|until| self.env.now < until) {
            signing.retry = None;
            let secret = signing.link.as_ref().expect("sign-in secret").clone();
            let with = if signing.service.is_some() {
                jig_client::With::Service(secret)
            } else {
                jig_client::With::Link { secret, held: None }
            };
            self.root_event(jig_client::Event::SignIn { with });
        }
    }
    pub(super) fn signing_keep(&mut self) {
        if self.class != Class::SignIn {
            return;
        }
        let Some(signing) = &mut self.signing else { return };
        let Some(shell) = &mut signing.shell else { return };
        if signing.credential == Credential::New
            && let Some(credential) = self.root.client().link().credential
        {
            signing.link = None;
            signing.retry = None;
            signing.last = None;
            if Some(credential) == signing.held.as_ref() {
                signing.credential = Credential::Kept;
            } else {
                shell.keep(Some(credential.clone()));
                signing.credential = Credential::Writing;
            }
            self.directory.store_address(self.env.now, signing.address.take().expect("parsed address"));
        }
        if let Some(kept) = shell.kept() {
            if !kept {
                self.fail(b"the sign-in token could not be kept");
                return;
            }
            signing.credential = Credential::Kept;
        }
        if signing.credential == Credential::Kept
            && self.directory.stored
            && self.directory.ready()
            && self.directory.party_stored.is_none()
            && let Some(party) = self.root.client().link().party
        {
            self.directory.store_party(self.env.now, party);
        }
        if signing.credential == Credential::Kept
            && self.directory.party_stored.is_some()
            && let Some(text) = signing.printed.take()
        {
            self.output.push(false, text);
            self.finish(0, false);
        }
    }
}

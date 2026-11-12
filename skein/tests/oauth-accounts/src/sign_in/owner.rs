//! Sign-in owner setup and outside request/terminal decisions.
use super::{Client, Env, Fact, List, Story, accounts, bytes, registration};

fn visited(client: &mut Client, env: &Env<accounts::Limits>) {
    let story = super::owner(client).story;
    match story {
        Story::Restart => {
            if super::owner(client).visits.len() == 1 {
                client.ask(env, accounts::Request::Cancel { account: 0 });
                client.ask(env, accounts::Request::SignIn { account: 0, subject: None });
            }
        }
        Story::CloseWaiting | Story::CloseConfidential | Story::PasteClose => {
            close(client, env);
            assert!(!client.closed);
        }
        Story::Cancel => {
            client.ask(env, accounts::Request::Cancel { account: 0 });
            close(client, env);
        }
        Story::Abort => {
            close(client, env);
            client.ask(env, accounts::Request::Abort);
        }
        Story::Paste
        | Story::PasteThenListener
        | Story::ListenerThenPaste
        | Story::PasteTwice
        | Story::PasteWrong
        | Story::StateRefusals
        | Story::ListenerWrongState
        | Story::PasteExpired
        | Story::PasteCancelled
        | Story::Return(_)
        | Story::OpenIdAgain
        | Story::IssuedClientId
        | Story::AnotherSubject
        | Story::RequiredScopes
        | Story::Public
        | Story::Profile(_)
        | Story::VaryingTaken
        | Story::TwoPorts
        | Story::Confidential
        | Story::WrongPath
        | Story::WrongHost
        | Story::LongHead
        | Story::NotKept
        | Story::Timeout => {}
    }
}

pub(crate) fn start(client: &mut Client, env: &Env<accounts::Limits>) {
    let story = super::owner(client).story;
    let count = if story == Story::TwoPorts { 2 } else { 1 };
    let mut configured = List::with_capacity(count);
    for _ in 0..count {
        assert!(
            configured
                .push(accounts::Account::SignIn {
                    origins: List::with_capacity(0),
                    shared: false,
                    registration: registration(story),
                    endpoint: accounts::Endpoint {
                        address: client.address.expect("issuer address"),
                        transport: accounts::Transport::Plaintext
                    },
                    keeper: match &client.file_driver {
                        Some(driver) => accounts::Keeper::Private {
                            root: driver.root(),
                            directory: bytes::copy_of(b"secret"),
                            file: bytes::copy_of(b"record")
                        },
                        None => accounts::Keeper::Owner { kept: None },
                    }
                })
                .is_ok()
        );
    }
    client.component = Some(
        accounts::Component::new(
            configured,
            &client.limits,
            skein_lib::CryptoRng::new(&skein_world::crypto_seed(17), 0).expect("32-byte world seed"),
            b"oauth-accounts-world",
        )
        .expect("accounts"),
    );
    if story == Story::Return(super::Return::Before) {
        return_to(client, env, bytes::copy_of(b"https://application.example/stale"));
        client.ask(
            env,
            accounts::Request::Return {
                account: u32::MAX,
                location: bytes::copy_of(b"https://application.example/missing"),
            },
        );
    }
    for account in 0..count {
        client.ask(
            env,
            accounts::Request::SignIn {
                account,
                subject: (story == Story::AnotherSubject).then(|| bytes::copy_of(super::EXPECTED_SUBJECT)),
            },
        );
        set_return(client, env);
    }
}
fn set_return(client: &mut Client, env: &Env<accounts::Limits>) {
    let story = super::owner(client).story;
    match story {
        Story::Return(super::Return::Replace) => {
            return_to(client, env, bytes::copy_of(b"https://application.example/first"));
            return_to(client, env, super::return_location(story).expect("replacement"));
        }
        Story::Return(super::Return::Long) => {
            let mut location = b"https://application.example/".to_vec();
            location.resize(accounts::RETURN_BYTES + 1, b'x');
            return_to(client, env, location.into_boxed_slice());
        }
        Story::Return(super::Return::Space) => {
            return_to(client, env, bytes::copy_of(b"https://application.example/a b"));
        }
        Story::Return(super::Return::Scheme) => return_to(client, env, bytes::copy_of(b"/back")),
        Story::Return(super::Return::Empty) => return_to(client, env, Box::new([])),
        Story::Return(super::Return::Control) => {
            return_to(client, env, bytes::copy_of(b"https://application.example/\r\n"));
        }
        Story::Return(super::Return::NonAscii) => {
            return_to(client, env, bytes::copy_of(b"https://application.example/\xff"));
        }
        Story::Return(super::Return::After) => {
            return_to(client, env, bytes::copy_of(b"https://application.example/first"));
        }
        Story::Paste
        | Story::PasteThenListener
        | Story::ListenerThenPaste
        | Story::PasteTwice
        | Story::PasteWrong
        | Story::StateRefusals
        | Story::ListenerWrongState
        | Story::PasteExpired
        | Story::PasteCancelled
        | Story::PasteClose
        | Story::Return(
            super::Return::Back | super::Return::Before | super::Return::Bound | super::Return::WrongPath,
        )
        | Story::OpenIdAgain
        | Story::IssuedClientId
        | Story::AnotherSubject
        | Story::RequiredScopes
        | Story::Public
        | Story::Profile(_)
        | Story::Restart
        | Story::VaryingTaken
        | Story::TwoPorts
        | Story::Confidential
        | Story::WrongPath
        | Story::WrongHost
        | Story::LongHead
        | Story::CloseWaiting
        | Story::CloseConfidential
        | Story::Cancel
        | Story::Abort
        | Story::NotKept
        | Story::Timeout => {
            if let Some(location) = super::return_location(story) {
                return_to(client, env, location);
            }
        }
    }
}
fn return_to(client: &mut Client, env: &Env<accounts::Limits>, location: Box<[u8]>) {
    client.ask(env, accounts::Request::Return { account: 0, location });
}
pub(crate) fn restart(client: &mut Client, env: &Env<accounts::Limits>) {
    if super::owner(client).callback == super::paste::Callback::Finished
        && let Some((account, generation)) = super::owner_mut(client).pending_keep.take()
    {
        kept(client, env, account, generation, true);
    }
    if super::owner(client).again_at.is_some_and(|at| env.now >= at) {
        super::owner_mut(client).again_at = None;
        super::owner_mut(client).contracts[0] = super::Contracts::new();
        client.ask(env, accounts::Request::SignIn { account: 0, subject: Some(bytes::copy_of(super::SUBJECT)) });
    }
}
fn keep(client: &mut Client, env: &Env<accounts::Limits>, account: u32, record: skein_oauth::SavedToken) {
    let story = super::owner(client).story;
    let generation = record.generation;
    super::owner_mut(client).contracts[usize::try_from(account).expect("account index")].keep(generation);
    super::owner_mut(client).facts.push(Fact::Keep(generation));
    let yes = story != Story::NotKept;
    let size = if super::owner(client).large { 256 } else { 16 };
    assert_eq!(record.access_token.expose(), vec![if generation == 1 { b'a' } else { b'b' }; size]);
    if matches!(story, Story::Profile(_)) && generation == 2 {
        assert_eq!(record.id_token, super::owner(client).records.first().expect("sign-in record").id_token);
    }
    super::owner_mut(client).records.push(record);
    if story == Story::PasteThenListener && super::owner(client).callback != super::paste::Callback::Finished {
        super::owner_mut(client).pending_keep = Some((account, generation));
    } else {
        kept(client, env, account, generation, yes);
    }
}
fn kept(client: &mut Client, env: &Env<accounts::Limits>, account: u32, generation: u64, yes: bool) {
    super::owner_mut(client).contracts[usize::try_from(account).expect("account index")].kept(generation, yes);
    client.ask(
        env,
        accounts::Request::Kept {
            account,
            generation,
            keeping: if yes { accounts::Keeping::Kept } else { accounts::Keeping::NotKept },
        },
    );
}
fn signed_in(client: &mut Client, env: &Env<accounts::Limits>, account: u32, generation: u64, missing: Box<[u8]>) {
    let story = super::owner(client).story;
    super::owner_mut(client).contracts[usize::try_from(account).expect("account index")].signed_in(generation);
    super::owner_mut(client).facts.push(Fact::SignedIn(generation));
    assert_eq!(
        generation,
        if super::again(story) { skein_lib::len_u64(super::owner(client).records.len()) } else { 1 }
    );
    if story == Story::Return(super::Return::After) {
        return_to(client, env, bytes::copy_of(b"https://application.example/stale"));
    }
    if !missing.is_empty() {
        super::owner_mut(client).facts.push(Fact::Missing(missing));
    }
    if !client.closing {
        client.ask(env, accounts::Request::Grant { account });
    }
    if story == Story::Confidential {
        client.ask(
            env,
            accounts::Request::Redirected {
                account,
                uri: skein_lib::Secret::new(bytes::copy_of(b"https://service.example/callback?state=late&code=late")),
            },
        );
    }
}
pub(crate) fn observe(client: &mut Client, env: &Env<accounts::Limits>) {
    let story = super::owner(client).story;
    if let Some(event) = client.above.pop() {
        assert!(!client.closed, "Closed once and last");
        match event {
            accounts::Event::Visit { url, .. } => {
                super::owner_mut(client).visits.push(url.into_exposed());
                super::owner_mut(client).facts.push(Fact::Visit);
                visited(client, env);
            }
            accounts::Event::Taken { .. } => super::owner_mut(client).facts.push(Fact::Taken),
            accounts::Event::Keep { account, record } => {
                keep(client, env, account, record);
            }
            accounts::Event::SignedIn { account, generation, missing, subject, profile } => {
                if matches!(story, Story::OpenIdAgain | Story::IssuedClientId) {
                    assert_eq!(subject.as_deref(), Some(super::SUBJECT));
                }
                signed_in(client, env, account, generation, missing);
                let expected = if matches!(story, Story::Profile(_)) {
                    super::owner_mut(client).facts.push(Fact::Profile);
                    super::issuer::profile_claims(false)
                } else {
                    skein_oauth::ProfileClaims::default()
                };
                assert_eq!(profile, expected);
            }
            accounts::Event::Refreshed { account, generation, profile } => {
                let state = super::owner_mut(client);
                state.contracts[usize::try_from(account).expect("account index")].granted(generation);
                assert!(!state.facts.contains(&Fact::Granted(generation)), "profile before grant");
                assert_eq!(profile, super::issuer::profile_claims(true));
                state.facts.push(Fact::Refreshed(generation));
            }
            accounts::Event::Granted { account, generation, .. } => {
                super::owner_mut(client).contracts[usize::try_from(account).expect("account index")]
                    .granted(generation);
                super::owner_mut(client).facts.push(Fact::Granted(generation));
                if generation == 1 && super::again(story) {
                    client.ask(env, accounts::Request::Release { account });
                    super::owner_mut(client).again_at = Some(env.now.saturating_add(skein_lib::Duration::from_secs(1)));
                } else if generation == 1 && matches!(story, Story::Public | Story::Profile(_)) {
                    client.ask(env, accounts::Request::Rejected { account, generation });
                } else if story == Story::ListenerThenPaste {
                    // The browser receives its page before the person pastes.
                } else if story != Story::TwoPorts
                    || super::owner(client).facts.iter().filter(|fact| matches!(fact, Fact::Granted(_))).count() == 2
                {
                    close(client, env);
                }
            }
            accounts::Event::Failed { account, ends, failure } => {
                let expected =
                    if story == Story::RequiredScopes { accounts::Ends::Grant } else { accounts::Ends::SignIn };
                assert_eq!(ends, expected);
                if ends == accounts::Ends::SignIn {
                    super::owner_mut(client).contracts[usize::try_from(account).expect("account index")].failed();
                }
                super::owner_mut(client).facts.push(Fact::Failed(failure));
                if story == Story::Restart {
                    assert_eq!(failure, accounts::Failure::Exchange(skein_oauth::Failure::Cancelled));
                    super::owner_mut(client).contracts[usize::try_from(account).expect("account index")] =
                        super::Contracts::new();
                } else {
                    if let Some(uri) = super::owner_mut(client).expired_paste.take() {
                        client.ask(env, accounts::Request::Redirected { account: 0, uri: skein_lib::Secret::new(uri) });
                    }
                    close(client, env);
                }
            }
            accounts::Event::Refused { asked: accounts::Asked::Redirected, why, .. } => {
                super::owner_mut(client).facts.push(Fact::Refused(why));
                if story == Story::ListenerThenPaste {
                    close(client, env);
                }
            }
            accounts::Event::Closed => {
                for contracts in &mut super::owner_mut(client).contracts {
                    contracts.closed();
                }
                super::owner_mut(client).facts.push(Fact::Closed);
                client.closed = true;
                if let Some(driver) = &mut client.file_driver {
                    driver.close_root(env.now);
                }
            }
            accounts::Event::Removed { .. }
            | accounts::Event::Refused { .. }
            | accounts::Event::Lock { .. }
            | accounts::Event::Unlock { .. }
            | accounts::Event::SignedOut { .. }
            | accounts::Event::Expiring { .. } => {
                panic!("unexpected sign-in owner event")
            }
        }
    }
}

fn close(client: &mut Client, env: &Env<accounts::Limits>) {
    if !client.closing {
        client.closing = true;
        client.ask(env, accounts::Request::Close);
    }
}

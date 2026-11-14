//! Outside sign-in outcomes and issuer document order.
use super::{Client, Fact, ISSUED_CLIENT, Judge, Story, accounts, bytes, oauth, peer};
const SIGN_INS: usize = 2;
const FIRST_GENERATION: u64 = 1;
const SECOND_GENERATION: u64 = FIRST_GENERATION + 1;
// The cancelled sign-in's terminal, then its replacement's whole sign-in.
const RESTARTED: &[Fact] = &[
    Fact::Visit,
    Fact::Failed(accounts::Failure::Exchange(oauth::Failure::Cancelled)),
    Fact::Visit,
    Fact::Keep(1),
    Fact::SignedIn(1),
    Fact::Granted(1),
    Fact::Closed,
];

impl Judge {
    #[expect(clippy::too_many_lines, reason = "each sign-in story returns the first broken outside outcome's finding")]
    pub(super) fn client_closed(&self, client: &Client, issuer: &peer::Peer) -> Result<(), String> {
        let client = super::owner(client);
        if client.facts.last() != Some(&Fact::Closed) {
            return Err(format!(
                "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                client.facts.last(),
                Some(&Fact::Closed)
            ));
        }
        if self.private_failure {
            if !client.facts.iter().any(|fact| matches!(fact, Fact::Failed(accounts::Failure::Unloaded { .. }))) {
                return Err(String::from(
                    "assertion failed: client.facts.iter().any(|fact| matches!(fact, Fact::Failed(accounts::Failure::Unloaded { .. })))",
                ));
            }
            if !client.visits.is_empty() {
                return Err(String::from("unsafe loading fails before Visit"));
            }
        } else {
            match self.story {
                Story::Cancel | Story::Abort | Story::PasteCancelled => {
                    if !client.facts.contains(&Fact::Failed(accounts::Failure::Exchange(oauth::Failure::Cancelled))) {
                        return Err(String::from(
                            "assertion failed: client.facts.contains(&Fact::Failed(accounts::Failure::Exchange(oauth::Failure::Cancelled)))",
                        ));
                    }
                }
                Story::Timeout | Story::PasteExpired => {
                    if !client.facts.contains(&Fact::Failed(accounts::Failure::Exchange(oauth::Failure::TimedOut))) {
                        return Err(String::from(
                            "assertion failed: client.facts.contains(&Fact::Failed(accounts::Failure::Exchange(oauth::Failure::TimedOut)))",
                        ));
                    }
                }
                Story::NotKept => {
                    if !client.facts.contains(&Fact::Failed(accounts::Failure::NotKept)) {
                        return Err(String::from(
                            "assertion failed: client.facts.contains(&Fact::Failed(accounts::Failure::NotKept))",
                        ));
                    }
                    if client.facts.iter().any(|fact| matches!(fact, Fact::SignedIn(_) | Fact::Granted(_))) {
                        return Err(String::from(
                            "assertion failed: !client.facts.iter().any(|fact| matches!(fact, Fact::SignedIn(_) | Fact::Granted(_)))",
                        ));
                    }
                }
                Story::RequiredScopes => {
                    if client.facts
                        != vec![
                            Fact::Visit,
                            Fact::Keep(1),
                            Fact::SignedIn(1),
                            Fact::Missing(bytes::copy_of(b"write")),
                            Fact::Failed(accounts::Failure::NotGranted),
                            Fact::Closed,
                        ]
                    {
                        return Err(format!(
                            "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                            client.facts,
                            vec![
                                Fact::Visit,
                                Fact::Keep(1),
                                Fact::SignedIn(1),
                                Fact::Missing(bytes::copy_of(b"write")),
                                Fact::Failed(accounts::Failure::NotGranted),
                                Fact::Closed,
                            ]
                        ));
                    }
                }
                Story::Restart => {
                    if client.facts != RESTARTED {
                        return Err(format!(
                            "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                            client.facts, RESTARTED
                        ));
                    }
                }
                Story::Public | Story::Profile(_) => {
                    if !client.facts.contains(&Fact::Granted(2)) {
                        return Err(String::from("assertion failed: client.facts.contains(&Fact::Granted(2))"));
                    }
                    if client.records.len() != 2 {
                        return Err(format!(
                            "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                            client.records.len(),
                            2
                        ));
                    }
                }
                Story::AnotherSubject => {
                    if client.facts != vec![Fact::Visit, Fact::Failed(accounts::Failure::AnotherSubject), Fact::Closed]
                    {
                        return Err(format!(
                            "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                            client.facts,
                            vec![Fact::Visit, Fact::Failed(accounts::Failure::AnotherSubject), Fact::Closed]
                        ));
                    }
                    if !client.records.is_empty() {
                        return Err(format!(
                            "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                            client.records.len(),
                            0
                        ));
                    }
                }
                Story::OpenIdAgain | Story::Return(super::Return::After) => {
                    if client.facts.iter().filter(|fact| matches!(fact, Fact::Visit)).count() != SIGN_INS {
                        return Err(format!(
                            "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                            client.facts.iter().filter(|fact| matches!(fact, Fact::Visit)).count(),
                            SIGN_INS
                        ));
                    }
                    if !client.facts.contains(&Fact::SignedIn(SECOND_GENERATION)) {
                        return Err(String::from(
                            "assertion failed: client.facts.contains(&Fact::SignedIn(SECOND_GENERATION))",
                        ));
                    }
                    if self.story == Story::OpenIdAgain {
                        if !client.records.iter().all(|record| record.refresh_token.is_none()) {
                            return Err(String::from(
                                "assertion failed: client.records.iter().all(|record| record.refresh_token.is_none())",
                            ));
                        }
                        let posts = issuer
                            .observations()
                            .iter()
                            .filter(|event| matches!(event, peer::Observation::Post { .. }))
                            .count();
                        if posts != SIGN_INS {
                            return Err(format!(
                                "assertion `left == right` failed: only sign-in token requests\n  left: {posts:?}\n right: {SIGN_INS:?}"
                            ));
                        }
                    }
                    document_visits(issuer.observations(), SIGN_INS)?;
                }
                Story::IssuedClientId => {
                    if client
                        .records
                        .first()
                        .is_none_or(|record| record.issued_client_id.as_deref() != Some(ISSUED_CLIENT))
                    {
                        return Err(String::from(
                            "assertion failed: client .records .first() .is_some_and(|record| record.issued_client_id.as_deref() == Some(ISSUED_CLIENT))",
                        ));
                    }
                }
                Story::Confidential => {
                    if !client.facts.contains(&Fact::Taken) {
                        return Err(String::from("assertion failed: client.facts.contains(&Fact::Taken)"));
                    }
                    if !client.facts.contains(&Fact::SignedIn(1)) {
                        return Err(String::from("assertion failed: client.facts.contains(&Fact::SignedIn(1))"));
                    }
                    if !client.facts.contains(&Fact::Refused(accounts::Refusal::NotWaiting)) {
                        return Err(String::from(
                            "assertion failed: client.facts.contains(&Fact::Refused(accounts::Refusal::NotWaiting))",
                        ));
                    }
                }
                Story::Paste
                | Story::PasteThenListener
                | Story::ListenerThenPaste
                | Story::PasteTwice
                | Story::PasteWrong
                | Story::StateRefusals
                | Story::ListenerWrongState
                | Story::PasteClose
                | Story::Return(_)
                | Story::VaryingTaken
                | Story::TwoPorts
                | Story::WrongPath
                | Story::WrongHost
                | Story::LongHead
                | Story::CloseWaiting
                | Story::CloseConfidential => {
                    if !client.facts.contains(&Fact::SignedIn(1)) {
                        return Err(String::from("assertion failed: client.facts.contains(&Fact::SignedIn(1))"));
                    }
                    if self.story == Story::CloseConfidential && !client.facts.contains(&Fact::Taken) {
                        return Err(String::from("assertion failed: client.facts.contains(&Fact::Taken)"));
                    }
                }
            }
        }
        Ok(())
    }
}
fn document_visits(observations: &[peer::Observation], visits: usize) -> Result<(), String> {
    let targets: Vec<&[u8]> = observations
        .iter()
        .filter_map(|observation| match observation {
            peer::Observation::Document { target, .. } | peer::Observation::Authorize { target, .. } => {
                Some(target.split(|byte| *byte == b'?').next().expect("path"))
            }
            peer::Observation::Accepted { .. }
            | peer::Observation::Post { .. }
            | peer::Observation::Revoke { .. }
            | peer::Observation::Redirect { .. }
            | peer::Observation::Answered { .. }
            | peer::Observation::Closed { .. } => None,
        })
        .collect();
    if targets != [b"/.well-known/openid-configuration".as_slice(), b"/keys", b"/authorize"].repeat(visits) {
        return Err(format!(
            "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
            targets,
            [b"/.well-known/openid-configuration".as_slice(), b"/keys", b"/authorize"].repeat(visits)
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closed_client_returns_a_missing_confidential_outcome() {
        let mut client = super::super::client(Story::Confidential, false);
        let owner = super::super::owner_mut(&mut client);
        owner.facts = vec![Fact::Closed];
        let judge = Judge {
            delivered: 0,
            redirected: false,
            shutdown: false,
            wake: None,
            passed: false,
            story: Story::Confidential,
            private_failure: false,
        };
        let super::super::Process::Issuer(issuer) = super::super::issuer::issuer(Story::Confidential, false) else {
            unreachable!("the issuer fixture")
        };
        assert_eq!(
            judge.client_closed(&client, &issuer),
            Err(String::from("assertion failed: client.facts.contains(&Fact::Taken)"))
        );
        super::super::owner_mut(&mut client).facts =
            vec![Fact::Taken, Fact::SignedIn(1), Fact::Refused(accounts::Refusal::NotWaiting), Fact::Closed];
        assert_eq!(judge.client_closed(&client, &issuer), Ok(()));
    }
}

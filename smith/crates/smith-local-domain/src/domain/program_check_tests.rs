use super::*;
use crate::{CredentialFailure, Mode, Notice, Undelivered};

#[test]
fn a_program_failure_drops_the_headless_start() {
    let mut config = crate::tests::config();
    config.mode = Mode::Headless { prompt: b"work".as_slice().into() };
    let (mut domain, env, mut out) = crate::tests::awaiting_credential(config);
    step(
        &mut domain,
        &env,
        Event::NoCredential { account: 7, reason: CredentialFailure::ProgramNotSignedIn },
        &mut out,
    );
    let Some(Request::Notice { notice: Notice::NotDelivered { line, why: Undelivered::Dropped } }) = out.pop() else {
        panic!("dropped line")
    };
    assert_eq!(line, Some(Token::new(1)));
    let Some(Request::Notice { notice }) = out.pop() else { panic!("credential notice") };
    assert_eq!(
        notice,
        Notice::Failure(Failure::Credential { account: 7, reason: CredentialFailure::ProgramNotSignedIn })
    );
    let Some(Request::Exit { status: ExitStatus::Credentials { account: 7 } }) = out.pop() else {
        panic!("credential exit")
    };
    assert!(out.is_empty());
}

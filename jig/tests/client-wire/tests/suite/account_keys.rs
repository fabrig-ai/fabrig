//! Client v21 account changes and typed outcomes.
use jig_client_protocol_wire::*;
use skein_lib::{List, Reader, Writer};
#[test]
fn all_eight_account_changes_and_their_outcomes_round_trip() {
    let mut uses = List::with_capacity(16);
    for n in 0..16 {
        uses.push(if n % 2 == 0 {
            AccountUse::Party(AccountUseParty { party: u64::from(n) })
        } else {
            AccountUse::Project(AccountUseProject { project: n })
        })
        .expect("bounded account wire fixture");
    }
    for change in [
        AccountChange::AddKey(AccountChangeAddKey { provider: 3, join: Some(7), digest: [19; 32] }),
        AccountChange::ReplaceKey(AccountChangeReplaceKey { source: 9, digest: [23; 32] }),
        AccountChange::RemoveKey(AccountChangeRemoveKey { source: 9 }),
        AccountChange::Resume(AccountChangeResume { source: 9 }),
        AccountChange::Remove(AccountChangeRemove { account: 7 }),
        AccountChange::MoveAccount(AccountChangeMoveAccount { account: 7, place: 2 }),
        AccountChange::MoveSource(AccountChangeMoveSource { account: 7, source: 9, place: 2 }),
        AccountChange::SetUses(AccountChangeSetUses { account: 7, uses: Some(uses) }),
    ] {
        for holder in [AskAccountHolder::Own, AskAccountHolder::Deployment] {
            let ask = Ask::AccountChange(AskAccountChange { holder, change: change.clone() });
            let mut out = Writer::new(skein_lib::length::usize_of(
                ask.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"),
            ));
            ask.write(&jig_client_protocol_wire::CEILINGS, &mut out).expect("bounded account wire fixture");
            assert_eq!(Ask::decode(&CEILINGS, &mut Reader::new(&out.finish())), Ok(ask));
        }
    }
    for outcome in [
        DocumentOutcome::AccountKeyKept(OutcomeAccountKeyKept { source: u32::MAX }),
        DocumentOutcome::AccountChanged,
        DocumentOutcome::AccountRemoved(OutcomeAccountRemoved {
            confirmed: true,
            failed_hosts: skein_lib::List::with_capacity(0),
        }),
    ] {
        let mut out = Writer::new(skein_lib::length::usize_of(
            outcome.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"),
        ));
        outcome.write(&jig_client_protocol_wire::CEILINGS, &mut out).expect("bounded account wire fixture");
        assert_eq!(DocumentOutcome::decode(&CEILINGS, &mut Reader::new(&out.finish())), Ok(outcome));
    }
    let mut uses = List::with_capacity(17);
    for _ in 0..17 {
        uses.push(AccountUse::Party(AccountUseParty { party: 1 })).expect("bounded account wire fixture");
    }
    assert!((AccountChangeSetUses { account: 1, uses: Some(uses) }).encode(&CEILINGS).is_err());
    let mut out = Writer::new(skein_lib::length::usize_of(
        Refusal::Provider.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"),
    ));
    Refusal::Provider.write(&jig_client_protocol_wire::CEILINGS, &mut out).expect("bounded account wire fixture");
    assert_eq!(Refusal::decode(&CEILINGS, &mut Reader::new(&out.finish())), Ok(Refusal::Provider));
}

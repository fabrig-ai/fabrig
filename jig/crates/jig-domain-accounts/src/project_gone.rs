//! A deleted project's explicit account uses (domain/accounts.md, 4.2;
//! temper/docs/design/draft/playground-reset.md, 5).
use crate::{Domain, Request, Use};
use skein_lib::{List, Queue};

pub(crate) fn project_gone(domain: &mut Domain, project: u32, out: &mut Queue<Request>) {
    let mut changed = List::with_capacity(domain.accounts.len());
    for (&number, account) in &domain.accounts {
        if let Some(uses) = &account.uses
            && uses.contains(&Use::Project(project))
        {
            changed.push(number).expect("bounded accounts");
        }
    }
    for number in changed.into_boxed() {
        let account = domain.accounts.get_mut(&number).expect("collected account");
        let mut kept = List::with_capacity(crate::USES);
        for payer in account.uses.as_ref().expect("explicit project use") {
            if *payer != Use::Project(project) {
                kept.push(*payer).expect("bounded account uses");
            }
        }
        account.uses = Some(kept.into_boxed());
        out.push(Request::Save { record: crate::ordering::account_record(domain, number) });
    }
}

//! Adopt source descriptors without changing their public shape (protocol/hosts.md, 5.4).
use super::{Account, Origin, Source, lock, oauth};
use skein_io::file_layer;
use skein_lib::{List, Wall};

pub(super) struct Binding {
    pub account: Account,
    pub configured: oauth::Account,
}

pub(super) fn source(
    source: Source,
    files: &mut file_layer::FileIo,
    locks: &mut List<lock::Lock>,
    shared: bool,
) -> Result<Binding, crate::Error> {
    let (number, origin, configured) = match source {
        Source::Env { account, borrowed } => {
            (account, Origin::Env { initial: Some(borrowed) }, oauth::Account::HandedIn)
        }
        Source::SignIn { account, root, directory, file, registration, endpoint, interactive, .. } => {
            let root = files.adopt_root(root).expect("validated private source count fits");
            locks.push(lock::Lock::new(account, root, &directory, &file)).ok().expect("account lock capacity");
            (
                account,
                Origin::SignIn { root: Some(root), interactive, sign_in: false, signing: false },
                oauth::Account::SignIn {
                    registration,
                    endpoint,
                    origins: List::with_capacity(0),
                    shared,
                    keeper: oauth::Keeper::Private { root, directory, file },
                },
            )
        }
    };
    Ok(Binding {
        account: Account {
            number,
            source: origin,
            generation: 0,
            expires: Wall::EPOCH,
            startup: true,
            loading: false,
            wanted: false,
            subject: None,
            subject_failure: None,
            pending: None,
        },
        configured,
    })
}

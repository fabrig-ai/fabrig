//! Checks the admin once for Forgejo's plans (skein/docs/design/forge/setup/forgejo.md, 1.2 and 1.3).
use crate::forgejo::reconcile;
use crate::plan::{Event, Why};
use skein_forge::forgejo::document::User;
use skein_lib::Secret;

#[derive(Debug)]
pub(crate) enum Admin {
    Begin,
    Read,
    Check,
}
pub(crate) enum Move {
    Read,
    Token(Secret),
    Passed(User),
}
pub(crate) fn step(admin: &mut Admin, event: Event) -> Result<Move, Why> {
    match admin {
        Admin::Begin => {
            event.begin()?;
            *admin = Admin::Read;
            Ok(Move::Read)
        }
        Admin::Read => {
            let token = event.read()?.ok_or(Why::Missing)?;
            *admin = Admin::Check;
            Ok(Move::Token(token))
        }
        Admin::Check => {
            let user = reconcile::account(event)?;
            if !user.is_admin {
                return Err(Why::NotAdmin);
            }
            Ok(Move::Passed(user))
        }
    }
}

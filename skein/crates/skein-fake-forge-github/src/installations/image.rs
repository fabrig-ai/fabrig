//! Installation state kept on the wall clock (testing/fakes/github.md, 1.7).
use super::{Grant, Installation, Installations, Issued, Kind, LOGIN_BYTES, Reach, Scope, TOKEN_BYTES};
use crate::image::{
    Refusal,
    codec::{Encoder, count, data, flag, num, small},
};
use skein_lib::{List, Reader, Rng, Secret, Wall};
impl Installations {
    pub(crate) fn encode_image(&self, out: &mut Encoder) {
        out.num(self.rng.state());
        out.num(self.next);
        out.num(self.minted);
        out.num(self.now.as_nanos());
        out.num(u64::from(self.rows.len()));
        for row in self.rows.iter().flatten() {
            out.num(row.id);
            out.num(row.account);
            out.bytes(&row.login);
            out.flag(row.kind == Kind::Organization);
            match &row.reach {
                Reach::All => out.flag(false),
                Reach::Listed(ids) => {
                    out.flag(true);
                    out.num(u64::from(ids.len()));
                    for id in ids {
                        out.num(u64::from(*id));
                    }
                }
            }
            out.flag(row.suspended);
            out.num(row.created.as_nanos());
        }
        out.num(u64::from(self.tokens.len()));
        for token in self.tokens.iter().flatten() {
            out.bytes(token.secret.expose());
            out.num(token.grant.installation);
            out.flag(token.grant.scope == Scope::Contents);
            out.num(token.expires.as_nanos());
            out.num(token.mint);
        }
    }
    pub(crate) fn decode_image(&self, input: &mut Reader<'_>, limits: &super::Limits) -> Result<Self, Refusal> {
        let rng = Rng::new(num(input, "installation rng")?);
        let next = num(input, "next installation")?;
        let minted = num(input, "mint count")?;
        let now = Wall::from_nanos(num(input, "installation clock")?);
        let mut restored = Self::with_life(limits, 0, self.life);
        restored.rng = rng;
        restored.next = next;
        restored.minted = minted;
        restored.now = now;
        if next == 0 {
            return Err(Refusal::Malformed("next installation"));
        }
        let length = count(input, limits.installations, "installations")?;
        let mut prior = 0;
        for _ in 0..length {
            let id = num(input, "installation id")?;
            if id <= prior || id >= next {
                return Err(Refusal::Malformed("installation id"));
            }
            prior = id;
            let account = num(input, "installation account")?;
            let login = data(input, LOGIN_BYTES, "installation login")?;
            if login.is_empty() {
                return Err(Refusal::Malformed("installation login"));
            }
            let kind = if flag(input, "installation kind")? { Kind::Organization } else { Kind::User };
            let reach = if flag(input, "installation reach")? {
                let length = count(input, limits.listed, "installation reach")?;
                let mut ids = List::with_capacity(length);
                for _ in 0..length {
                    ids.push(small(input, "reach repository")?).expect("admitted reach");
                }
                Reach::Listed(ids)
            } else {
                Reach::All
            };
            let suspended = flag(input, "installation suspension")?;
            let created = Wall::from_nanos(num(input, "installation created")?);
            restored
                .rows
                .push(Some(Installation { id, account, login, kind, reach, suspended, created }))
                .expect("admitted installations");
        }
        let length = count(input, limits.tokens, "installation tokens")?;
        let mut prior = 0;
        for _ in 0..length {
            let secret = Secret::new(data(input, TOKEN_BYTES, "installation token")?);
            let installation = num(input, "token installation")?;
            if restored.get(installation).is_none() {
                return Err(Refusal::Malformed("token installation"));
            }
            let scope = if flag(input, "token scope")? { Scope::Contents } else { Scope::Full };
            let expires = Wall::from_nanos(num(input, "token expiry")?);
            let mint = num(input, "token mint")?;
            if mint <= prior || mint > minted || secret.len() != TOKEN_BYTES {
                return Err(Refusal::Malformed("token mint"));
            }
            prior = mint;
            restored
                .tokens
                .push(Some(Issued { secret, grant: Grant { installation, scope }, expires, mint }))
                .expect("admitted tokens");
        }
        Ok(restored)
    }
}

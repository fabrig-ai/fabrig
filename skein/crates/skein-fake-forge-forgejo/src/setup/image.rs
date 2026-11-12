//! The whole bounded setup store (testing/fakes/forge.md, section 4).
use super::{Account, Issued, Organization, RepositoryHook, Store, Subscription, Team, collaborators::Collaborator};
use crate::image::{
    Refusal,
    codec::{Encoder, count, data, flag, num, small, strings},
};
use skein_forge::forgejo::{
    document::{self, TeamSpec, Units},
    request::Grant,
};
use skein_lib::{List, Reader, Secret};
impl Store {
    pub(crate) fn encode_image(&self, out: &mut Encoder) {
        out.num(u64::from(self.accounts.len()));
        for row in &self.accounts {
            out.num(row.id);
            out.bytes(&row.login);
            out.flag(row.admin);
            out.bytes(&row.full_name);
        }
        out.num(u64::from(self.tokens.len()));
        for row in &self.tokens {
            out.num(row.id);
            out.num(row.user);
            out.bytes(&row.name);
            out.strings(&row.scopes);
            out.bytes(row.secret.expose());
        }
        out.num(u64::from(self.orgs.len()));
        for row in &self.orgs {
            out.num(row.id);
            out.bytes(&row.name);
            out.bytes(&row.owner);
        }
        out.num(u64::from(self.teams.len()));
        for row in &self.teams {
            out.num(row.id);
            out.bytes(&row.org);
            write_spec(out, &row.spec);
            out.num(u64::from(row.members.len()));
            for member in &row.members {
                out.num(*member);
            }
        }
        out.num(u64::from(self.hooks.len()));
        for row in &self.hooks {
            out.num(row.id);
            out.bytes(&row.org);
            out.bytes(&row.url);
            out.bytes(row.secret.expose());
            out.strings(&row.events);
            out.flag(row.active);
        }
        out.num(u64::from(self.private.len()));
        for row in &self.private {
            out.num(*row);
        }
        out.num(u64::from(self.collaborators.len()));
        for row in &self.collaborators {
            out.num(u64::from(row.repository));
            out.bytes(&row.login);
            out.num(match row.permission {
                Grant::Read => 0,
                Grant::Write => 1,
                Grant::Admin => 2,
            });
        }
        out.num(u64::from(self.repo_hooks.len()));
        for row in &self.repo_hooks {
            out.num(row.id);
            out.num(u64::from(row.repository));
            out.bytes(&row.url);
            out.bytes(row.secret.expose());
            out.strings(&row.events);
            out.flag(row.active);
        }
        out.num(u64::from(self.repository_high));
        out.num(self.next_repo_hook);
        out.num(self.next_org);
        out.num(self.next_team);
        out.num(self.next_hook);
        out.num(self.next_account);
        out.num(self.next_token);
    }
    #[expect(
        clippy::too_many_lines,
        reason = "the flat store schema stages every field and counter before replacement"
    )]
    pub(crate) fn decode_image(&self, input: &mut Reader<'_>) -> Result<Self, Refusal> {
        let cap = self.accounts.capacity();
        let string = self.string_bytes;
        let secret = self.secret_bytes.max(40);
        let length = count(input, cap, "accounts")?;
        let mut accounts = List::with_capacity(cap);
        for _ in 0..length {
            accounts
                .push(Account {
                    id: num(input, "accounts id")?,
                    login: data(input, string, "accounts login")?,
                    admin: flag(input, "accounts admin")?,
                    full_name: data(input, string, "accounts full_name")?,
                })
                .expect("admitted accounts");
        }
        let length = count(input, cap, "tokens")?;
        let mut tokens = List::with_capacity(cap);
        for _ in 0..length {
            tokens
                .push(Issued {
                    id: num(input, "tokens id")?,
                    user: num(input, "tokens user")?,
                    name: data(input, string, "tokens name")?,
                    scopes: strings(input, cap, string, "tokens scopes")?,
                    secret: Secret::new(data(input, secret, "tokens secret")?),
                })
                .expect("admitted tokens");
        }
        let length = count(input, cap, "orgs")?;
        let mut orgs = List::with_capacity(cap);
        for _ in 0..length {
            orgs.push(Organization {
                id: num(input, "orgs id")?,
                name: data(input, string, "orgs name")?,
                owner: data(input, string, "orgs owner")?,
            })
            .expect("admitted orgs");
        }
        let length = count(input, cap, "teams")?;
        let mut teams = List::with_capacity(cap);
        for _ in 0..length {
            teams
                .push(Team {
                    id: num(input, "teams id")?,
                    org: data(input, string, "teams org")?,
                    spec: read_spec(input, string)?,
                    members: read_members(input, cap)?,
                })
                .expect("admitted teams");
        }
        let length = count(input, cap, "hooks")?;
        let mut hooks = List::with_capacity(cap);
        for _ in 0..length {
            hooks
                .push(Subscription {
                    id: num(input, "hooks id")?,
                    org: data(input, string, "hooks org")?,
                    url: data(input, string, "hooks url")?,
                    secret: Secret::new(data(input, secret, "hooks secret")?),
                    events: strings(input, cap, string, "hooks events")?,
                    active: flag(input, "hooks active")?,
                })
                .expect("admitted hooks");
        }
        let length = count(input, cap, "private")?;
        let mut private = List::with_capacity(cap);
        for _ in 0..length {
            private.push(num(input, "private")?).expect("admitted private");
        }
        let length = count(input, cap, "collaborators")?;
        let mut collaborators = List::with_capacity(cap);
        for _ in 0..length {
            collaborators
                .push(Collaborator {
                    repository: small(input, "collaborators repository")?,
                    login: data(input, string, "collaborators login")?,
                    permission: match num(input, "collaborators permission")? {
                        0 => Grant::Read,
                        1 => Grant::Write,
                        2 => Grant::Admin,
                        _ => return Err(Refusal::Malformed("collaborators permission")),
                    },
                })
                .expect("admitted collaborators");
        }
        let length = count(input, cap, "repo_hooks")?;
        let mut repo_hooks = List::with_capacity(cap);
        for _ in 0..length {
            repo_hooks
                .push(RepositoryHook {
                    id: num(input, "repo_hooks id")?,
                    repository: small(input, "repo_hooks repository")?,
                    url: data(input, string, "repo_hooks url")?,
                    secret: Secret::new(data(input, secret, "repo_hooks secret")?),
                    events: strings(input, cap, string, "repo_hooks events")?,
                    active: flag(input, "repo_hooks active")?,
                })
                .expect("admitted repo_hooks");
        }
        let repository_high = small(input, "repository_high")?;
        let next_repo_hook = num(input, "next_repo_hook")?;
        let next_org = num(input, "next_org")?;
        let next_team = num(input, "next_team")?;
        let next_hook = num(input, "next_hook")?;
        let next_account = num(input, "next_account")?;
        let next_token = num(input, "next_token")?;
        if next_account == 0 {
            return Err(Refusal::Malformed("next_account"));
        }
        for (index, row) in accounts.iter().enumerate() {
            if row.id == 0 || row.id >= next_account {
                return Err(Refusal::Malformed("accounts id"));
            }
            for prior in accounts.iter().take(index) {
                if prior.id == row.id {
                    return Err(Refusal::Malformed("accounts duplicate id"));
                }
            }
        }
        if next_token == 0 {
            return Err(Refusal::Malformed("next_token"));
        }
        for (index, row) in tokens.iter().enumerate() {
            if row.id == 0 || row.id >= next_token {
                return Err(Refusal::Malformed("tokens id"));
            }
            for prior in tokens.iter().take(index) {
                if prior.id == row.id {
                    return Err(Refusal::Malformed("tokens duplicate id"));
                }
            }
        }
        if next_org == 0 {
            return Err(Refusal::Malformed("next_org"));
        }
        for (index, row) in orgs.iter().enumerate() {
            if row.id == 0 || row.id >= next_org {
                return Err(Refusal::Malformed("orgs id"));
            }
            for prior in orgs.iter().take(index) {
                if prior.id == row.id {
                    return Err(Refusal::Malformed("orgs duplicate id"));
                }
            }
        }
        if next_team == 0 {
            return Err(Refusal::Malformed("next_team"));
        }
        for (index, row) in teams.iter().enumerate() {
            if row.id == 0 || row.id >= next_team {
                return Err(Refusal::Malformed("teams id"));
            }
            for prior in teams.iter().take(index) {
                if prior.id == row.id {
                    return Err(Refusal::Malformed("teams duplicate id"));
                }
            }
        }
        if next_hook == 0 {
            return Err(Refusal::Malformed("next_hook"));
        }
        for (index, row) in hooks.iter().enumerate() {
            if row.id == 0 || row.id >= next_hook {
                return Err(Refusal::Malformed("hooks id"));
            }
            for prior in hooks.iter().take(index) {
                if prior.id == row.id {
                    return Err(Refusal::Malformed("hooks duplicate id"));
                }
            }
        }
        if next_repo_hook == 0 {
            return Err(Refusal::Malformed("next_repo_hook"));
        }
        for (index, row) in repo_hooks.iter().enumerate() {
            if row.id == 0 || row.id >= next_repo_hook {
                return Err(Refusal::Malformed("repo_hooks id"));
            }
            for prior in repo_hooks.iter().take(index) {
                if prior.id == row.id {
                    return Err(Refusal::Malformed("repo_hooks duplicate id"));
                }
            }
        }
        Ok(Self {
            accounts,
            tokens,
            orgs,
            teams,
            hooks,
            private,
            collaborators,
            repo_hooks,
            repository_high,
            next_repo_hook,
            next_org,
            next_team,
            next_hook,
            next_account,
            next_token,
            string_bytes: self.string_bytes,
            secret_bytes: self.secret_bytes,
        })
    }
}
fn write_spec(out: &mut Encoder, spec: &TeamSpec) {
    out.bytes(&spec.name);
    out.bytes(document::access_text(spec.permission));
    out.flag(spec.includes_all_repositories);
    for unit in [spec.units.code, spec.units.issues, spec.units.pulls, spec.units.actions] {
        out.bytes(document::access_text(unit));
    }
}
fn access(input: &mut Reader<'_>) -> Result<document::Access, Refusal> {
    document::access(&data(input, 5, "team access")?).ok_or(Refusal::Malformed("team access"))
}
fn read_spec(input: &mut Reader<'_>, string: usize) -> Result<TeamSpec, Refusal> {
    Ok(TeamSpec {
        name: data(input, string, "team name")?,
        permission: access(input)?,
        includes_all_repositories: flag(input, "team all repositories")?,
        units: Units { code: access(input)?, issues: access(input)?, pulls: access(input)?, actions: access(input)? },
    })
}
fn read_members(input: &mut Reader<'_>, cap: u32) -> Result<List<u64>, Refusal> {
    let length = count(input, cap, "team members")?;
    let mut members = List::with_capacity(cap);
    for _ in 0..length {
        members.push(num(input, "team member")?).expect("admitted members");
    }
    Ok(members)
}

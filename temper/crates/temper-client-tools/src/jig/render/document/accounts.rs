//! Area 5's common, bounded account projection; no selection or catalogue inference.
use super::{Cut, Limits, Text};
use crate::jig::digest::Digest;
use alloc::boxed::Box;
use jig_client::contract as c;
use skein_json::writer::Encoder;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Accounts<'a> {
    pub party: Option<&'a c::OwnerAccounts>,
    pub deployment: Option<&'a c::OwnerDeployment>,
    pub models: Option<(&'a c::Me, Option<u32>)>,
    pub outcome: Option<(bool, &'a [u8])>,
    pub current: bool,
}
impl Accounts<'_> {
    pub fn digest(self, limits: &Limits) -> Digest {
        Digest::of(&self.bytes(limits, false))
    }
    fn bytes(self, limits: &Limits, printed: bool) -> Box<[u8]> {
        let bounds = skein_json::writer::Limits { depth: 32, length: u32::MAX };
        let mut measure = Encoder::measure(&bounds);
        encode(self, limits, printed, &mut measure);
        let mut out = Encoder::write(measure.measured().expect("bounded account projection"), &bounds);
        encode(self, limits, printed, &mut out);
        out.finish()
    }
}
// Reserve 1 KiB for the envelope and 128 B for its digest/wait outcome.
// Each of the two readiness lists has its own capacity (contract rows) × 128 B.
// Remaining bytes split between account/catalogue structure (1 KiB per row)
// and escaped text (six bytes per input byte).
pub(super) struct Window {
    pub(super) left: u32,
    pub(super) cut: bool,
}
impl Window {
    pub(super) fn take(&mut self) -> bool {
        if let Some(left) = self.left.checked_sub(1) {
            self.left = left;
            true
        } else {
            self.cut = true;
            false
        }
    }
}
pub(super) fn encode(value: Accounts<'_>, limits: &Limits, printed: bool, out: &mut Encoder) {
    let room = limits.text.saturating_sub(128);
    let readiness = super::offers::readiness_rows(limits);
    let room = room.saturating_sub(1024).saturating_sub(readiness.saturating_mul(2).saturating_mul(128));
    let mut window = Window { left: limits.rows.min(room / 2048), cut: false };
    let mut cut =
        Cut { room: usize::try_from(room / 12).expect("text fits"), left: 0, turn: None, subject: false, digest: None };
    out.object_start();
    if let Some((me, project)) = value.models {
        models(me, project, out, &mut cut, &mut window);
    }
    out.key(b"current");
    out.boolean(value.current);
    out.key(b"accounts");
    match value.party {
        None => out.null(),
        Some(party) => party_fields(party, value.deployment, limits, out, &mut cut, &mut window),
    }
    number(b"bytes_left", cut.left, out);
    out.key(b"cut");
    out.boolean(window.cut || cut.left > 0);
    if printed {
        out.key(b"digest");
        out.string(&value.digest(limits).hex());
        if let Some((met, why)) = value.outcome {
            if !met {
                out.key(b"why");
                out.string(why);
            }
            out.key(b"met");
            out.boolean(met);
        }
    }
    out.object_end();
}
fn party_fields(
    party: &c::OwnerAccounts,
    extra: Option<&c::OwnerDeployment>,
    limits: &Limits,
    out: &mut Encoder,
    cut: &mut Cut,
    window: &mut Window,
) {
    out.object_start();
    for (key, flag) in [(b"usable".as_slice(), party.usable), (b"settled", party.settled)] {
        out.key(key);
        out.boolean(flag);
    }
    out.key(b"party");
    out.array_start();
    for row in &party.accounts {
        if !window.take() {
            break;
        }
        out.object_start();
        line(
            row.number,
            row.place,
            row.provider,
            row.provider_name.as_ref(),
            row.manage_usage.as_deref(),
            row.plan,
            &row.display,
            row.holder,
            row.uses.as_ref(),
            &row.sources,
            row.cut,
            out,
            cut,
            window,
        );
        // The party document supplies no account activity lists.
        out.key(b"runs");
        out.null();
        out.key(b"waiting");
        out.null();
        out.object_end();
    }
    out.array_end();
    out.key(b"signing");
    out.array_start();
    for signing in &party.signing {
        if !window.take() {
            break;
        }
        out.object_start();
        number(b"source", u64::from(signing.source), out);
        number(b"provider", u64::from(signing.provider), out);
        out.key(b"holder");
        holder(signing.holder, out);
        out.object_end();
    }
    out.array_end();
    out.key(b"last_end");
    match &party.last_end {
        None => out.null(),
        Some(last) => {
            out.object_start();
            number(b"source", u64::from(last.source), out);
            out.key(b"end");
            end(last.end, out);
            out.object_end();
        }
    }
    super::account_programs::hosts(&party.program_hosts, out, cut, window);
    super::offers::readiness(&party.readiness, party.readiness_cut, limits, out, window);
    deployment(extra, out, cut, window);
    out.key(b"deployment_facts");
    match extra {
        None => out.null(),
        Some(extra) => {
            out.object_start();
            super::offers::fields(&extra.providers, extra.providers_cut, extra.key_bytes, out, cut, window);
            super::offers::readiness(&extra.readiness, extra.readiness_cut, limits, out, window);
            super::account_programs::deployment_hosts(extra, out, cut, window);
            out.object_end();
        }
    }
    window.cut |= party.cut;
    out.key(b"cut");
    out.boolean(window.cut || cut.left > 0);
    out.object_end();
}
fn deployment(deployment: Option<&c::OwnerDeployment>, out: &mut Encoder, cut: &mut Cut, window: &mut Window) {
    out.key(b"deployment");
    out.array_start();
    if let Some(deployment) = deployment {
        window.cut |= deployment.cut;
        for row in &deployment.accounts {
            if !window.take() {
                break;
            }
            out.object_start();
            line(
                row.number,
                row.place,
                row.provider,
                row.provider_name.as_ref(),
                row.manage_usage.as_deref(),
                row.plan,
                &row.display,
                row.holder,
                row.uses.as_ref(),
                &row.sources,
                row.cut,
                out,
                cut,
                window,
            );
            optional(b"generation", row.generation, out);
            out.key(b"state");
            state(row.state, out);
            flags(row.usable, row.expiring, row.attention, row.spent_until, out);
            for (key, chips) in [(b"runs".as_slice(), &row.runs), (b"waiting", &row.waiting)] {
                out.key(key);
                out.array_start();
                for chip in chips {
                    if !window.take() {
                        break;
                    }
                    out.unsigned(chip.task);
                }
                out.array_end();
            }
            out.object_end();
        }
    }
    out.array_end();
}
#[expect(clippy::too_many_arguments, reason = "party and deployment share these supplied fields")]
fn line(
    number_: u32,
    place: u32,
    supplied_provider: Option<u32>,
    provider_name: Option<&c::DocumentText>,
    manage_usage: Option<&[u8]>,
    plan: bool,
    display: &c::DocumentText,
    held: c::OwnerAccountHolder,
    uses: Option<&skein_lib::List<c::OwnerAccountUse>>,
    sources: &skein_lib::List<c::OwnerAccountSource>,
    lost: bool,
    out: &mut Encoder,
    cut: &mut Cut,
    window: &mut Window,
) {
    number(b"number", u64::from(number_), out);
    number(b"place", u64::from(place), out);
    provider(supplied_provider, out);
    window.cut |= display.cut;
    out.key(b"provider_name");
    match provider_name {
        Some(name) => {
            super::encode_document_text(name, out, cut);
            window.cut |= name.cut;
        }
        None => out.null(),
    }
    out.key(b"manage_usage");
    match manage_usage {
        Some(uri) => super::offers::whole(uri, out, cut, window),
        None => out.null(),
    }

    out.key(b"plan");
    out.boolean(plan);
    out.key(b"display");
    super::encode_document_text(display, out, cut);
    out.key(b"holder");
    holder(held, out);
    out.key(b"uses");
    match uses {
        None => out.null(),
        Some(uses) => {
            out.array_start();
            for payer in uses {
                if !window.take() {
                    break;
                }
                out.object_start();
                match payer {
                    c::OwnerAccountUse::Party(n) => number(b"party", *n, out),
                    c::OwnerAccountUse::Project(n) => number(b"project", u64::from(*n), out),
                }
                out.object_end();
            }
            out.array_end();
        }
    }
    out.key(b"sources");
    out.array_start();
    for source in sources {
        if !window.take() {
            break;
        }
        out.object_start();
        number(b"number", u64::from(source.number), out);
        number(b"place", u64::from(source.place), out);
        out.key(b"name");
        match &source.name {
            Some(name) => super::offers::whole(name, out, cut, window),
            None => out.null(),
        }
        out.key(b"kind");
        out.string(match source.kind {
            c::OwnerAccountKind::SignIn => b"sign_in",
            c::OwnerAccountKind::Key => b"key",
            c::OwnerAccountKind::Program => b"program",
            c::OwnerAccountKind::None => b"none",
        });
        out.key(b"state");
        state(source.state, out);
        flags(source.usable, source.expiring, source.attention, source.spent_until, out);
        out.object_end();
    }
    out.array_end();
    window.cut |= lost;
    out.key(b"cut");
    out.boolean(lost || window.cut);
}
fn flags(usable: bool, expiring: bool, attention: bool, until: Option<skein_lib::Wall>, out: &mut Encoder) {
    for (key, flag) in [(b"usable".as_slice(), usable), (b"expiring", expiring), (b"attention", attention)] {
        out.key(key);
        out.boolean(flag);
    }
    wall(b"spent_until", until, out);
}
fn number(key: &[u8], value: u64, out: &mut Encoder) {
    out.key(key);
    out.unsigned(value);
}
fn optional(key: &[u8], value: Option<u64>, out: &mut Encoder) {
    out.key(key);
    match value {
        Some(n) => out.unsigned(n),
        None => out.null(),
    }
}
fn holder(value: c::OwnerAccountHolder, out: &mut Encoder) {
    out.object_start();
    match value {
        c::OwnerAccountHolder::Party(n) => number(b"party", n, out),
        c::OwnerAccountHolder::Deployment => {
            out.key(b"deployment");
            out.boolean(true);
        }
    }
    out.object_end();
}
pub(super) fn unloaded(at: c::OwnerAccountPlace, why: c::OwnerAccountUnread, out: &mut Encoder) {
    out.key(b"at");
    out.string(match at {
        c::OwnerAccountPlace::Directory => b"directory",
        c::OwnerAccountPlace::File => b"file",
    });
    out.key(b"why");
    out.string(match why {
        c::OwnerAccountUnread::Unreadable => b"unreadable",
        c::OwnerAccountUnread::Refused => b"refused",
        c::OwnerAccountUnread::Failed => b"failed",
        c::OwnerAccountUnread::Stalled => b"stalled",
        c::OwnerAccountUnread::TooLarge => b"too_large",
    });
}
fn state(value: c::OwnerAccountState, out: &mut Encoder) {
    use c::OwnerAccountState as S;
    out.object_start();
    out.key(b"kind");
    out.string(match value {
        S::Moving => b"moving",
        S::Asking => b"asking",
        S::Granted => b"granted",
        S::Later { .. } => b"later",
        S::SignInAgain => b"sign_in_again",
        S::Misconfigured => b"misconfigured",
        S::Unloaded { .. } => b"unloaded",
        S::Cancelled => b"cancelled",
        S::Closing => b"closing",
    });
    match value {
        S::Later { until } => wall(b"until", until, out),
        S::Unloaded { at, why } => unloaded(at, why, out),
        S::Moving | S::Asking | S::Granted | S::SignInAgain | S::Misconfigured | S::Cancelled | S::Closing => {}
    }
    out.object_end();
}
fn end(value: c::OwnerAccountEnd, out: &mut Encoder) {
    use c::{OwnerAccountEnd as E, OwnerAccountFailure as F, OwnerAccountRefusal as R};
    out.object_start();
    match value {
        E::Made { account, declined } => made(b"made", account, declined, out),
        E::Renewed { account, declined } => made(b"renewed", account, declined, out),
        E::Failed { failure } => {
            out.key(b"failed");
            out.string(match failure {
                F::Later { .. } => b"later",
                F::SignInAgain => b"sign_in_again",
                F::MissingScope { .. } => b"missing_scope",
                F::Misconfigured => b"misconfigured",
                F::Unloaded { .. } => b"unloaded",
                F::Cancelled => b"cancelled",
            });
            match failure {
                F::Later { after } => {
                    out.key(b"after");
                    match after {
                        Some(span) => out.unsigned(span.as_nanos()),
                        None => out.null(),
                    }
                }
                F::MissingScope { scope } => number(b"scope", u64::from(scope), out),
                F::Unloaded { at, why } => unloaded(at, why, out),
                F::SignInAgain | F::Misconfigured | F::Cancelled => {}
            }
        }
        E::Refused { refusal } => {
            out.key(b"refused");
            out.string(match refusal {
                R::NotSignedIn => b"not_signed_in",
                R::NotOurs => b"not_ours",
                R::Cancelled => b"cancelled",
                R::Moving => b"moving",
                R::NotLimited => b"not_limited",
                R::Uses => b"uses",
                R::NotKept { .. } => b"not_kept",
                R::Limit { .. } => b"limit",
                R::Held { .. } => b"held",
                R::Provider { .. } => b"provider",
                R::Subject => b"subject",
                R::Unknown => b"unknown",
                R::Numbers => b"numbers",
                R::Kind { .. } => b"kind",
                R::Closing => b"closing",
                R::NotStarted => b"not_started",
            });
            match refusal {
                R::Held { provider } | R::Provider { provider } => number(b"provider", u64::from(provider), out),
                R::NotKept { why } => {
                    out.key(b"why");
                    out.string(match why {
                        c::OwnerAccountKeyFailure::TooLong => b"too_long",
                        c::OwnerAccountKeyFailure::NotStored => b"not_stored",
                    });
                }
                R::Limit { bound } => {
                    out.key(b"bound");
                    out.string(match bound {
                        c::OwnerAccountBound::Accounts => b"accounts",
                        c::OwnerAccountBound::HolderAccounts => b"holder_accounts",
                        c::OwnerAccountBound::Sources => b"sources",
                        c::OwnerAccountBound::SignIns => b"sign_ins",
                    });
                }
                R::Kind { kind } => {
                    out.key(b"kind");
                    out.string(match kind {
                        c::OwnerAccountKind::SignIn => b"sign_in",
                        c::OwnerAccountKind::Key => b"key",
                        c::OwnerAccountKind::Program => b"program",
                        c::OwnerAccountKind::None => b"none",
                    });
                }
                R::NotSignedIn
                | R::NotOurs
                | R::Cancelled
                | R::Moving
                | R::NotLimited
                | R::Uses
                | R::Subject
                | R::Unknown
                | R::Numbers
                | R::Closing
                | R::NotStarted => {}
            }
        }
    }
    out.object_end();
}
fn models(me: &c::Me, project: Option<u32>, out: &mut Encoder, cut: &mut Cut, window: &mut Window) {
    window.cut |= me.cut;
    out.key(b"models");
    out.array_start();
    for held in &me.projects {
        if let Some(project) = project
            && project != held.project
        {
            continue;
        }
        if !window.take() {
            break;
        }
        out.object_start();
        number(b"project", u64::from(held.project), out);
        out.key(b"providers");
        out.array_start();
        for (index, model) in held.models.as_slice().iter().enumerate() {
            if window.left == 0 {
                window.cut = true;
                break;
            }
            let mut seen = false;
            for previous in held.models.as_slice().get(..index).expect("preceding models") {
                seen |= previous.provider == model.provider;
            }
            if seen {
                continue;
            }
            if !window.take() {
                break;
            }
            out.object_start();
            provider(model.provider, out);
            out.key(b"provider_name");
            let mut name = None;
            for offer in &me.providers {
                if Some(offer.provider) == model.provider {
                    name = Some(&offer.name);
                    break;
                }
            }
            match name {
                Some(name) => {
                    super::encode_document_text(name, out, cut);
                    window.cut |= name.cut;
                }
                None => out.null(),
            }
            out.key(b"models");
            out.array_start();
            for item in &held.models {
                if item.provider != model.provider {
                    continue;
                }
                if !window.take() {
                    break;
                }
                out.object_start();
                out.key(b"name");
                cut.words(item.name.bytes(), out);
                number(b"window", u64::from(item.window), out);
                number(b"output", u64::from(item.output), out);
                out.key(b"description");
                match &item.description {
                    Some(description) => cut.words(description.bytes(), out),
                    None => out.null(),
                }
                out.key(b"prices");
                out.object_start();
                number(b"input", item.prices.input, out);
                number(b"cached", item.prices.cached, out);
                // jig prices cache writes at input when no separate price is configured.
                number(b"cache_write", item.prices.cache_write.unwrap_or(item.prices.input), out);
                number(b"output", item.prices.output, out);
                number(b"unit", u64::from(item.prices.unit), out);
                out.object_end();
                out.key(b"efforts");
                out.array_start();
                for effort in &item.efforts {
                    if !window.take() {
                        break;
                    }
                    cut.words(effort.bytes(), out);
                }
                out.array_end();
                out.object_end();
            }
            out.array_end();
            out.object_end();
        }
        out.array_end();
        out.object_end();
    }
    out.array_end();
}
pub(super) fn text(value: Accounts<'_>, limits: &Limits, out: &mut Text) {
    // Keep every printed identity and cut identical to JSON, followed by the
    // usual text digest and wait status. No second account interpretation.
    super::account_lines::text(&value.bytes(limits, false), out);
    out.put(b"\ndigest: ");
    out.put(if value.models.is_some() { b"models" } else { b"accounts" });
    out.put(b" ");
    out.put(&value.digest(limits).hex());
    out.put(b"\n");
    if let Some((met, why)) = value.outcome {
        if met {
            out.put(b"met\n");
        } else {
            out.put(b"not met: ");
            out.put(why);
            out.put(b"\n");
        }
    }
}

fn made(kind: &[u8], account: u32, declined: bool, out: &mut Encoder) {
    out.key(b"kind");
    out.string(kind);
    number(b"account", u64::from(account), out);
    out.key(b"declined");
    out.boolean(declined);
}
fn provider(value: Option<u32>, out: &mut Encoder) {
    out.key(b"provider");
    match value {
        Some(n) => out.unsigned(u64::from(n)),
        None => out.null(),
    }
}
fn wall(key: &[u8], value: Option<skein_lib::Wall>, out: &mut Encoder) {
    out.key(key);
    match value {
        Some(wall) => out.unsigned(wall.as_nanos()),
        None => out.null(),
    }
}

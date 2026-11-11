//! Stop asking and Keep asking on a need's reason (temper/docs/design/client/pages/README.md, Needs you).
use super::{
    Pages,
    needs::{self, Need, Toasted, Why},
};
use crate::{
    Domain, Limits, Request,
    forge::{
        dismissed::{self, PersonFact},
        ways,
    },
    owner::Owner,
};
use alloc::boxed::Box;
use jig_client::{Key, Outcome, Told, contract as c};
use skein_lib::{Decimal, Env, List, Queue, Token, Wall, bytes};
use temper_protocol_reaches::{Classes, Kind, Offer, Reach, Reason, Subject, narrowed, offers};

/// Seven days of Keep asking, in wall-clock nanoseconds (client/pages/README.md).
const KEEP_NS: u64 = 604_800_000_000_000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Act {
    Open { object: c::ObjectKey },
    Words { words: Box<[u8]> },
    Apply { offer: Offer },
    Propose { offer: Offer },
    Keep { object: c::ObjectKey },
}
#[derive(Debug)]
pub struct Open {
    object: c::ObjectKey,
    project: u32,
    reach: Reach,
    words: Box<[u8]>,
    out: Option<Out>,
    refusal: Option<c::Refusal>,
}
#[derive(Clone, Copy, Debug)]
enum Out {
    Apply(Token),
    Propose(Key),
}
#[derive(Debug)]
pub(super) struct Keeping {
    key: Option<Key>,
    project: u32,
    subject: Subject,
    at: Wall,
}
impl Open {
    #[must_use]
    pub const fn object(&self) -> c::ObjectKey {
        self.object
    }
    #[must_use]
    pub fn words(&self) -> &[u8] {
        &self.words
    }
    #[must_use]
    pub const fn busy(&self) -> bool {
        self.out.is_some()
    }
    #[must_use]
    pub const fn refused(&self) -> Option<&c::Refusal> {
        self.refusal.as_ref()
    }
}
impl Pages {
    #[must_use]
    pub const fn stop_asking(&self) -> Option<&Open> {
        self.stop_asking.as_ref()
    }
}
fn subject(reach: Reach) -> Option<Subject> {
    match reach.reason {
        Reason::Kind => Some(Subject::Kind(reach.kind)),
        Reason::Set(number) => Some(Subject::Set(number)),
        Reason::Always => None,
    }
}
#[must_use]
pub fn offered(root: &Domain, need: Need<'_>, wall: Wall) -> Option<Box<[Offer]>> {
    let Why::Reach { project, reach } = needs::why(root, need)? else { return None };
    let subject = subject(reach)?;
    if !root.forge.dismissals_delivered() {
        return None;
    }
    for pending in &root.pages.keep_asking {
        if pending.project == project
            && pending.subject == subject
            && (pending.key.is_some() || wall.as_nanos().saturating_sub(pending.at.as_nanos()) < KEEP_NS)
        {
            return None;
        }
    }
    if let Some(at) = root.forge.keep_asking(project, subject)
        && wall.as_nanos().saturating_sub(at.as_nanos()) < KEEP_NS
    {
        return None;
    }
    let offered = offers(root.forge.classes(project)?, &reach);
    if offered.is_empty() { None } else { Some(offered) }
}
#[must_use]
pub fn owner(root: &Domain, project: u32) -> bool {
    let Some(me) = root.pages.me(&root.client) else { return false };
    for row in &me.projects {
        if row.project == project {
            return row.role == c::HeldRole::Owner;
        }
    }
    false
}
fn open(root: &mut Domain, object: c::ObjectKey, wall: Wall) {
    if let Some(open) = &root.pages.stop_asking
        && open.object == object
    {
        root.pages.stop_asking = None;
        return;
    }
    root.pages.stop_asking = None;
    let Some(inbox) = root.pages.inbox() else { return };
    for need in &root.pages.needs().needs(inbox, root.forge.approvals()) {
        if need.object() != object || offered(root, *need, wall).is_none() {
            continue;
        }
        let Some(Why::Reach { project, reach }) = needs::why(root, *need) else { return };
        root.pages.stop_asking = Some(Open { object, project, reach, words: Box::default(), out: None, refusal: None });
        return;
    }
}
pub(crate) fn act(root: &mut Domain, env: &Env<Limits>, act: Act, out: &mut Queue<Request>) {
    match act {
        Act::Open { object } => open(root, object, env.wall),
        Act::Words { words } => {
            if let Some(open) = &mut root.pages.stop_asking
                && !open.busy()
            {
                open.words = words;
                open.refusal = None;
            }
        }
        Act::Apply { offer } => change(root, env, offer, true, out),
        Act::Propose { offer } => change(root, env, offer, false, out),
        Act::Keep { object } => keep(root, env, object, out),
    }
    root.pages.shown = root.pages.shown.checked_add(1).expect("pages shown fits");
}
fn change(root: &mut Domain, env: &Env<Limits>, offer: Offer, apply: bool, out: &mut Queue<Request>) {
    let Some(open) = &root.pages.stop_asking else { return };
    let project = open.project;
    if open.busy() || root.forge.classes_busy(project) || apply != owner(root, project) {
        return;
    }
    let Some(against) = root.forge.settings_newest() else { return };
    let Some(classes) = root.forge.classes(project) else { return };
    let Ok(table) = narrowed(classes, &open.reach, offer, &open.words) else {
        root.pages.stop_asking.as_mut().expect("open offers").refusal = Some(c::Refusal::Malformed {
            at: 0,
            why: c::Malformed::Settings(c::MalformedSettings {
                setting: c::Setting::Application { kind: 8, scope: project, number: 1 },
                fault: c::SettingFault::Shape,
                detail: None,
            }),
        });
        return;
    };
    if apply {
        let mut child = Queue::uncapped();
        ways::act(
            &mut root.forge,
            env.limits.forge,
            ways::Act::Set { project, asked: ways::Asked::Classes(table) },
            &mut child,
        );
        if let Some(request) = root.forge.classes_request(project) {
            let open = root.pages.stop_asking.as_mut().expect("open offers");
            open.out = Some(Out::Apply(request));
            open.refusal = None;
        }
        crate::route::route_forge(root, env, &mut child, out);
    } else {
        let Some(reason) = proposal_reason(root, open, classes, env.limits.client.domain.text) else { return };
        let Some(item) = ways::setting_item(project, &ways::Asked::Classes(table), env.limits.forge) else { return };
        let mut items = List::with_capacity(1);
        items.push(item).expect("one classes item");
        let key = next(&mut root.pages);
        let open = root.pages.stop_asking.as_mut().expect("open offers");
        open.out = Some(Out::Propose(key));
        open.refusal = None;
        crate::route::client(
            root,
            env,
            jig_client::Event::Send {
                key,
                name: None,
                request: jig_client::Ask::Jig(Box::new(c::Ask::Propose {
                    project: 0,
                    action: c::ProposeAction::Settings(c::ProposeSettings { against, items }),
                    reason,
                })),
            },
            out,
        );
    }
}
fn next(pages: &mut Pages) -> Key {
    pages.serial = pages.serial.checked_add(1).expect("page keys fit");
    Key { asker: Owner::Pages.asker(), number: pages.serial }
}
fn keep(root: &mut Domain, env: &Env<Limits>, object: c::ObjectKey, out: &mut Queue<Request>) {
    if let Some(open) = &root.pages.stop_asking
        && open.busy()
    {
        return;
    }
    let Some(inbox) = root.pages.inbox() else { return };
    let mut why = None;
    for need in &root.pages.needs().needs(inbox, root.forge.approvals()) {
        if need.object() == object && offered(root, *need, env.wall).is_some() {
            why = needs::why(root, *need);
            break;
        }
    }
    let Some(Why::Reach { project, reach }) = why else { return };
    let Some(subject) = subject(reach) else { return };
    let Some(request) = dismissed::ask(PersonFact::KeepAsking { project, subject, at: env.wall }, &env.limits.forge)
    else {
        return;
    };
    let key = next(&mut root.pages);
    // Acknowledgements may precede the facts document; keep their suppression until
    // that document catches up or the seven days pass.
    for index in (0..root.pages.keep_asking.len()).rev() {
        let pending = root.pages.keep_asking.get(index).expect("bounded request");
        let caught_up = match root.forge.keep_asking(pending.project, pending.subject) {
            Some(at) => at >= pending.at,
            None => false,
        };
        if pending.key.is_none() && (env.wall.as_nanos().saturating_sub(pending.at.as_nanos()) >= KEEP_NS || caught_up)
        {
            root.pages.keep_asking.swap_remove(index).expect("acknowledged request");
        }
    }
    root.pages
        .keep_asking
        .push(Keeping { key: Some(key), project, subject, at: env.wall })
        .expect("session requests grow");
    root.pages.stop_asking = None;
    crate::route::client(root, env, jig_client::Event::Send { key, name: None, request }, out);
}
/// The proposal's durable reason uses the same subject plurals as the view's shared object
/// (client/pages/README.md, Stop asking).
fn proposal_reason(root: &Domain, open: &Open, classes: &Classes, bound: u32) -> Option<c::Text> {
    let me = root.pages.me(&root.client)?;
    let mut project = None;
    for row in &me.projects {
        if row.project == open.project {
            project = Some(row.name.bytes());
        }
    }
    let project = match project {
        Some(name) => Box::from(name),
        None => bytes::concat(&[b"Project ", Decimal::of(u64::from(open.project)).as_bytes()]),
    };
    let subject = subject(open.reach)?;
    let object = match subject {
        Subject::Kind(kind) => Box::from(match kind {
            Kind::Wording => b"wording choices".as_slice(),
            Kind::Retry => b"retries and splits",
            Kind::Implementation => b"implementation choices",
            Kind::Tests => b"test-only changes",
            Kind::Dependency => b"new dependencies",
            Kind::Departure => b"departures from a landed design",
            Kind::Money => b"spending past a budget",
            Kind::Irreversible => b"irreversible changes",
            Kind::Rules => b"changes to what reaches you",
            Kind::Other => b"other decisions",
        }),
        Subject::Set(number) => bytes::concat(&[b"changes to ", &classes.set(number)?.name]),
    };
    let object = if let Some(index) = open.reach.under.checked_sub(1)
        && let Some(item) = classes.items.get(usize::from(index))
        && !item.words.is_empty()
    {
        bytes::concat(&[&object, " under “".as_bytes(), &item.words, "”".as_bytes()])
    } else {
        object
    };
    c::Text::new(
        bytes::concat(&[
            b"Stop asking in ",
            &project,
            b": let agents decide ",
            &object,
            b", then tell whoever they would have asked.",
        ]),
        bound,
    )
    .ok()
}
/// Called after the single settings sender has reconciled a reply, including a stale retry.
pub(crate) fn applied(root: &mut Domain, env: &Env<Limits>, told: &Told) {
    let Some(open) = &root.pages.stop_asking else { return };
    let Some(Out::Apply(request)) = open.out else { return };
    let Told::Answered { key, outcome } = told else { return };
    if key.asker != Owner::Forge.asker() || key.number != request.raw() {
        return;
    }
    if let Some(next) = root.forge.classes_request(open.project) {
        root.pages.stop_asking.as_mut().expect("open offers").out = Some(Out::Apply(next));
        return;
    }
    if let Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::SettingsChanged(_)))) = outcome {
        root.pages.stop_asking = None;
        root.pages.needs.say(Toasted::StopApplied, env.now);
    } else {
        refuse(root.pages.stop_asking.as_mut().expect("open offers"), outcome);
    }
    root.pages.shown = root.pages.shown.checked_add(1).expect("pages shown fits");
}
fn refuse(open: &mut Open, outcome: &Outcome) {
    open.out = None;
    open.refusal = Some(match outcome {
        Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused { refusal, .. })) => refusal.clone(),
        Outcome::Answered(_)
        | Outcome::Unknown
        | Outcome::NotKept
        | Outcome::Limit
        | Outcome::Unconfirmed
        | Outcome::Held
        | Outcome::Unsent
        | Outcome::Full { .. }
        | Outcome::Lost => c::Refusal::NotWaiting,
    });
}
pub(crate) fn told(root: &mut Domain, env: &Env<Limits>, told: &Told) -> bool {
    let (key, outcome) = match told {
        Told::Answered { key, outcome } => (*key, outcome),
        Told::Pending { key } => {
            if let Some(pending) = root.client.pending(*key)
                && pending.progress == jig_client::Progress::Unknown
            {
                (*key, &Outcome::Unknown)
            } else {
                return false;
            }
        }
        Told::Link { .. }
        | Told::Changed { .. }
        | Told::Ended { .. }
        | Told::Page { .. }
        | Told::Text { .. }
        | Told::Kept { .. }
        | Told::Dropped { .. } => return false,
    };
    for index in 0..root.pages.keep_asking.len() {
        if root.pages.keep_asking.get(index).expect("bounded request").key == Some(key) {
            let toast = match dismissed::answer(outcome, env.limits.forge) {
                dismissed::Answer::Dismissed => {
                    root.pages.keep_asking.get_mut(index).expect("matched request").key = None;
                    Some(Toasted::KeptAsking)
                }
                dismissed::Answer::Unknown => {
                    root.pages.keep_asking.swap_remove(index).expect("matched request");
                    None
                }
                dismissed::Answer::Refused => {
                    root.pages.keep_asking.swap_remove(index).expect("matched request");
                    Some(Toasted::KeepAskingRefused)
                }
            };
            if let Some(toast) = toast {
                root.pages.needs.say(toast, env.now);
            }
            root.pages.shown = root.pages.shown.checked_add(1).expect("pages shown fits");
            return true;
        }
    }
    if let Some(open) = &mut root.pages.stop_asking
        && let Some(Out::Propose(pending)) = open.out
        && key == pending
    {
        if let Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(
            c::DocumentOutcome::Proposed { .. },
        )))) = outcome
        {
            root.pages.stop_asking = None;
            root.pages.needs.say(Toasted::StopProposed, env.now);
        } else {
            refuse(open, outcome);
        }
        root.pages.shown = root.pages.shown.checked_add(1).expect("pages shown fits");
        return true;
    }
    false
}

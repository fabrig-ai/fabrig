//! Read a project's follows (temper/docs/design/domain/forge/follows.md, 7).
use crate::{
    jig::{
        self,
        call::{Value, number, value},
        render::{self, Text},
        runner::{Called, Out},
        table::{Keyed, Kind, Occurs::Required, Tool, flag, positional},
    },
    root::Limits,
};
use alloc::{boxed::Box, collections::BTreeMap};
use jig_client::{Client, Event, Handle, Latest, State, Told, Watchable, Why};
use skein_lib::{Env, Queue};
use temper_client_domain::{
    forge::{self, follows},
    owner::Owner,
};
use temper_client_protocol_wire::{self as wire, v1 as w};
mod acts;
mod print;
pub use acts::{HOLD, MERGE, SKIP, START};
pub const LIST: Tool = Tool {
    name: b"follow list",
    line: b"List a project's follows, their lag and parity",
    prints: b"One line per follow: source and target branches, lag and parity; `no follows` when empty; then one line per dependency",
    result: b"follows",
    forms: &[&[flag(b"project", b"PROJECT", Kind::Project { empty: false }, Required, b"The project")]],
    keyed: Keyed::Never,
    area: 6,
    notes: b"`follow show --project PROJECT FOLLOW` reads one whole.\n",
};
pub const SHOW: Tool = Tool {
    name: b"follow show",
    line: b"Read one follow whole",
    prints: b"The setting, task, lag oldest first, skips and parity",
    result: b"follow",
    forms: &[&[
        flag(b"project", b"PROJECT", Kind::Project { empty: false }, Required, b"The project"),
        positional(
            b"follow",
            b"FOLLOW",
            Kind::NumberRange { min: 1, max: 4_294_967_295 },
            Required,
            b"The follow's number",
        ),
    ]],
    keyed: Keyed::Never,
    area: 6,
    notes: b"The lag and parity are the forge's retained facts.\n",
};
#[derive(Debug)]
pub(crate) struct Follows {
    calls: BTreeMap<u32, Watching>,
    acts: acts::Acts,
}
#[derive(Debug)]
struct Watching {
    called: Called,
    follow: Option<u32>,
    project: Option<u32>,
    panel: Option<Handle>,
}
impl Follows {
    pub(crate) fn new() -> Self {
        Self { calls: BTreeMap::new(), acts: acts::Acts::new() }
    }
    pub(crate) fn kept_json(&mut self, json: bool) {
        self.acts.json = json;
    }
    pub(crate) fn calls(&self) -> usize {
        self.calls.len().saturating_add(self.acts.calls())
    }
}
pub(crate) fn owns(call: &jig::call::Call) -> bool {
    let name = crate::table::TOOLS.get(usize::from(call.tool)).expect("parsed tool").name;
    name == LIST.name || name == SHOW.name || acts::owns(call)
}
pub(crate) fn call(state: &mut Follows, client: &Client, env: &Env<Limits>, called: Called, out: &mut Queue<Out>) {
    if acts::owns(&called.call) {
        acts::call(&mut state.acts, client, env, called, out);
        return;
    }
    if called.value.is_some() || jig::argv::check(crate::table::TOOLS, &called.call, &env.limits.tools).is_err() {
        out.push(Out::Printed {
            number: called.number,
            printed: render::failure(b"invalid follow call", called.json, 1),
        });
        return;
    }
    let follow = match number(&called.call, 1) {
        Some(number) => Some(u32::try_from(number).expect("checked follow")),
        None => None,
    };
    state.calls.insert(called.call_number, Watching { called, follow, project: None, panel: None });
    ready(state, client, env, out);
}
pub(crate) fn subject(project: u32, limits: &Limits) -> Watchable {
    follows::watch(project, &limits.domain.forge).expect("checked follows watch")
}
fn complete(state: &mut Follows, number: u32, printed: jig::Printed, out: &mut Queue<Out>) {
    let watch = state.calls.remove(&number).expect("active follow read");
    if let Some(handle) = watch.panel {
        out.push(Out::Client { event: Event::Unwatch { handle } });
    }
    out.push(Out::Printed { number: watch.called.number, printed });
}
pub(crate) fn ready(state: &mut Follows, client: &Client, env: &Env<Limits>, out: &mut Queue<Out>) {
    acts::ready(&mut state.acts, client, env, out);
    let Some(me) = crate::forge::me(client) else { return };
    for _ in 0..state.calls.len() {
        let mut next = None;
        for (&number, watch) in &state.calls {
            if watch.panel.is_none() {
                next = Some(number);
                break;
            }
        }
        let Some(number) = next else { break };
        let watch = state.calls.get_mut(&number).expect("active follow read");
        watch.project = match value(&watch.called.call, 0) {
            Some(Value::Project(jig::call::Project::Number(number))) => Some(*number),
            Some(Value::Project(jig::call::Project::Name(name))) => {
                let mut found = None;
                for project in &me.projects {
                    if project.name.bytes() == name.as_ref() {
                        found = Some(project.project);
                        break;
                    }
                }
                found
            }
            Some(_) | None => None,
        };
        if let Some(project) = watch.project {
            let handle = Handle { asker: Owner::Forge.asker(), number: (u64::from(number) << 32_u32) | 1 };
            watch.panel = Some(handle);
            out.push(Out::Client { event: Event::Watch { handle, subject: subject(project, &env.limits) } });
        } else {
            let printed = render::failure(b"unknown project", watch.called.json, 1);
            complete(state, number, printed, out);
        }
    }
}
fn missing(watch: &Watching) -> jig::Printed {
    let words = skein_lib::bytes::concat(&[
        b"no follow ",
        skein_lib::Decimal::of(u64::from(watch.follow.expect("show"))).as_bytes(),
    ]);
    render::failure(&words, watch.called.json, 2)
}
pub(crate) fn told(state: &mut Follows, client: &Client, env: &Env<Limits>, told: &Told, out: &mut Queue<Out>) {
    acts::told(&mut state.acts, env, told, out);
    let handle = match told {
        Told::Changed { handle } | Told::Ended { handle, .. } => *handle,
        Told::Link { .. }
        | Told::Pending { .. }
        | Told::Answered { .. }
        | Told::Kept { .. }
        | Told::Dropped { .. }
        | Told::Text { .. }
        | Told::Page { .. } => return,
    };
    let number = u32::try_from(handle.number >> 32_u32).expect("call number");
    let Some(watch) = state.calls.get(&number) else { return };
    if watch.panel != Some(handle) {
        return;
    }
    let projects = match crate::forge::me(client) {
        Some(me) => me.projects.as_slice(),
        None => &[],
    };
    let printed = match told {
        Told::Changed { .. } => {
            let Some(doc) = client.document(handle) else { return };
            if (doc.state == State::Opening || doc.state == State::Behind) && doc.latest.is_none() {
                return;
            }
            match doc.latest {
                Some(Latest::Family(record))
                    if record.family == wire::FORGE && record.version == wire::CLIENT_VERSION =>
                {
                    let limits = forge::wire_limits(&env.limits.domain.forge).expect("checked limits");
                    match wire::decode_follows_panel(&record.bytes, &limits) {
                        Some(panel) if Some(panel.project()) == watch.project => {
                            let mut row = None;
                            if let Some(number) = watch.follow {
                                for follow in panel.follows() {
                                    if follow.number() == number {
                                        row = Some(follow);
                                        break;
                                    }
                                }
                            }
                            if watch.follow.is_some() && row.is_none() {
                                missing(watch)
                            } else {
                                print::print(&panel, row, projects, watch.called.json, &env.limits)
                            }
                        }
                        Some(_) | None => render::failure(b"unexpected follows document", watch.called.json, 1),
                    }
                }
                Some(Latest::Family(_) | Latest::Jig(_)) | None => {
                    render::failure(b"unexpected follows document", watch.called.json, 1)
                }
            }
        }
        Told::Ended { why, .. } => match why {
            Why::Unknown | Why::SubjectEnded => {
                if watch.follow.is_some() {
                    missing(watch)
                } else {
                    let panel = w::FollowsPanel::new(
                        &w::CEILINGS,
                        w::FollowsPanelParts {
                            project: watch.project.expect("project"),
                            follows: skein_lib::List::with_capacity(0),
                            dependencies: skein_lib::List::with_capacity(0),
                        },
                    )
                    .expect("empty follows");
                    print::print(&panel, None, projects, watch.called.json, &env.limits)
                }
            }
            Why::NotAllowed => render::failure(b"not_allowed", watch.called.json, 2),
            Why::SignedOut => render::failure(b"signed out", watch.called.json, 1),
            Why::NotServed => render::failure(b"version not served", watch.called.json, 1),
            Why::Limit => render::failure(b"watch table limit", watch.called.json, 1),
        },
        Told::Link { .. }
        | Told::Pending { .. }
        | Told::Answered { .. }
        | Told::Kept { .. }
        | Told::Dropped { .. }
        | Told::Text { .. }
        | Told::Page { .. } => return,
    };
    complete(state, number, printed, out);
}

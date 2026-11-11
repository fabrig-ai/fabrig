//! The shared decided region (temper/docs/design/client/pages/README.md, Decided for you).
use crate::{
    frame::Bindings,
    look,
    words::{Shell, TimeForm, Times, Word, Words},
};
use skein_ui::{Builder, Role, Run, State, phrases::Hole};
use temper_client_domain::{
    Domain,
    pages::{
        Act,
        decided::{DecisionLine, DecisionTask, Shown},
    },
};

pub(crate) fn main(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    project: Option<u32>,
    shown: &Shown,
    times: &mut Times,
) {
    times.next = [times.next, Some(shown.expires)].into_iter().flatten().min();
    let heading = w.say(
        if shown.more { Word::DecidedCountMore } else { Word::HomeDecidedCount },
        &[Hole::Number(u64::from(shown.showing))],
    );
    b.open(Role::Region);
    let mut needs = false;
    if let temper_client_domain::Address::Inbox { .. } = root.pages().address() {
        needs = true;
    }
    b.name(&heading);
    b.open(Role::Slot);
    b.style(if needs { look::DECIDED_HEAD } else { look::HOME_HEAD });
    text(b, Role::Heading { level: 2 }, &heading, None);
    if let Some(project) = project {
        b.open(Role::Button);
        b.name(&w.say(Word::AwayLimits, &[]));
        b.bind(bindings.insert(Act::Away(temper_client_domain::pages::away::Act::Limits { project })));
        b.close();
    } else if needs {
        text(b, Role::Text, &w.say(Word::NeedsDecidedLifetime, &[]), None);
    }
    b.close();
    b.open(Role::Slot);
    b.style(look::DECIDED);
    for row in &shown.rows {
        let name = if project.is_none() {
            Some(
                super::project_name(root.pages().me(root.client()), row.project)
                    .unwrap_or(w.say(Word::Project, &[Hole::Number(u64::from(row.project))])),
            )
        } else {
            None
        };
        decision(b, Some(root), w, bindings, row, times, name.as_deref(), root.pages().overturn_busy());
    }
    b.close();
    b.close();
}
#[expect(clippy::too_many_arguments, reason = "shared rows carry optional live facts and a project label")]
pub(crate) fn decision(
    b: &mut Builder,
    root: Option<&Domain>,
    w: &Words,
    bindings: &mut Bindings,
    row: &DecisionLine,
    times: &mut Times,
    name: Option<&[u8]>,
    busy: bool,
) {
    b.open(Role::Slot);
    b.style(look::DECISION);
    if row.overturned {
        b.style(look::DECISION_OVERTURNED);
    }
    b.open(if w.shell == Shell::Terminal { Role::Paragraph } else { Role::Slot });
    let fallback = w.say(Word::Task, &[Hole::Number(row.task)]);
    let title = row.title.as_deref().unwrap_or(&fallback);
    let at = times.say(w, TimeForm::Clock, row.at);
    let title = crate::accounts::words::join(&[title, &w.say(Word::ToolSeparator, &[]), &at]);
    let title = match name {
        Some(name) => crate::accounts::words::join(&[name, &w.say(Word::ToolSeparator, &[]), &title]),
        None => title,
    };
    text(b, Role::Text, &title, None);
    if w.shell == Shell::Terminal {
        b.text(Run::Plain, b" ");
    }
    text(b, Role::Text, &row.question, None);
    if w.shell == Shell::Terminal {
        b.text(Run::Plain, b" ");
    }
    text(b, Role::Text, &row.took, Some(look::DECISION_TOOK));
    if w.shell == Shell::Terminal {
        b.text(Run::Plain, b" ");
    }
    if let Some(reach) = &row.reach
        && let Some(root) = root
    {
        let allowed = crate::shared::allowed(w, reach, root.forge().classes(row.project));
        if !allowed.is_empty() {
            text(b, Role::Text, &allowed, Some(look::DECISION_ALLOWED));
            if w.shell == Shell::Terminal {
                b.text(Run::Plain, b" ");
            }
        }
    }
    text(b, Role::Text, &row.why, Some(look::DECISION_WHY));
    let late = match row.state {
        DecisionTask::Done(at) => {
            Some(w.say(Word::HomeDecisionLanded, &[Hole::Name(times.say(w, TimeForm::Clock, at))]))
        }
        DecisionTask::Ended(at) => {
            Some(w.say(Word::HomeDecisionEnded, &[Hole::Name(times.say(w, TimeForm::Clock, at))]))
        }
        DecisionTask::Live | DecisionTask::Unknown => {
            if row.overturned {
                Some(w.say(Word::HomeDecisionOverturned, &[]))
            } else {
                None
            }
        }
    };
    if w.shell == Shell::Browser {
        b.close();
    }
    if let Some(late) = late {
        if w.shell == Shell::Terminal {
            b.text(Run::Plain, b" ");
        }
        text(b, Role::Text, &late, Some(look::DECISION_LATE));
    }
    if row.state == DecisionTask::Live && !row.overturned {
        button(
            b,
            bindings,
            &w.say(Word::Overturn, &[]),
            Act::Away(temper_client_domain::pages::away::Act::Overturn { project: row.project, number: row.number }),
            look::HOME_SMALL,
            busy,
        );
    }
    if w.shell == Shell::Terminal {
        b.close();
    }
    b.close();
}
fn text(b: &mut Builder, role: Role, words: &[u8], style: Option<skein_ui::Style>) {
    b.open(role);
    if let Some(style) = style {
        b.style(style);
    }
    b.name(words);
    b.text(Run::Plain, words);
    b.close();
}
fn button(b: &mut Builder, bindings: &mut Bindings, label: &[u8], act: Act, style: skein_ui::Style, disabled: bool) {
    b.open(Role::Button);
    b.style(style);
    b.name(label);
    if disabled {
        b.state(State::Disabled);
    }
    b.bind(bindings.insert(act));
    b.close();
}

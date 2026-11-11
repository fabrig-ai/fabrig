//! One placement, shared by host and policy (protocol/setup/box.md, 3).
use alloc::boxed::Box;
use skein_lib::List;
use skein_lib::{Decimal, bytes};
use temper_forge_setup::{FORGE_ARGUMENT, Stage, Step, Steps, machine::BoxKind};
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Asked {
    pub forge: u16,
    pub recorded: bool,
    pub stage: Stage,
    pub kind: Option<BoxKind>,
    pub remover: bool,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Placed {
    pub step: Step,
    pub asked: Option<Asked>,
    pub label: Option<Box<[u8]>>,
}
#[must_use]
pub fn flatten(steps: Steps) -> Box<[Placed]> {
    let mut count = u32::try_from(steps.always.len()).expect("bounded steps");
    for group in &steps.asked {
        count = count.checked_add(u32::from(group.remover.is_some())).expect("remover slot");
        count = count.checked_add(u32::try_from(group.steps.len()).expect("bounded group")).expect("bounded steps");
    }
    let mut out = List::with_capacity(count);
    // Slots move ownership without copying commands; groups with the same
    // insertion point retain their setup order.
    let mut groups = List::with_capacity(u32::try_from(steps.asked.len()).expect("bounded groups"));
    for group in steps.asked {
        groups.push(Some(group)).expect("group capacity");
    }
    let always_len = steps.always.len();
    let mut always = steps.always.into_iter();
    for before in 0..=always_len {
        for index in 0..groups.len() {
            let group = groups.get_mut(index).expect("group slot");
            if let Some(present) = group
                && usize::try_from(present.before).expect("index fits") == before
            {
                let group = group.take().expect("present group");
                let mut label = Some(group.label);
                for (stage, step) in group.steps {
                    out.push(Placed {
                        step,
                        asked: Some(Asked {
                            forge: group.forge,
                            recorded: group.recorded,
                            stage,
                            kind: group.kind,
                            remover: false,
                        }),
                        label: label.take(),
                    })
                    .expect("step capacity");
                }
                if let Some(step) = group.remover {
                    out.push(Placed {
                        step,
                        asked: Some(Asked {
                            forge: group.forge,
                            recorded: group.recorded,
                            stage: Stage::Run,
                            kind: group.kind,
                            remover: true,
                        }),
                        label: None,
                    })
                    .expect("remover capacity");
                }
            }
        }
        if let Some(step) = always.next() {
            out.push(Placed { step, asked: None, label: None }).expect("step capacity");
        }
    }
    for group in &groups {
        assert!(group.is_none(), "group's before is within always");
    }
    out.into_boxed()
}
/// Labels are supplied opaque bytes, owned once at the group's first step.
#[must_use]
pub fn label(steps: &[Placed], forge: u16) -> Option<&[u8]> {
    for placed in steps {
        if let Some(asked) = placed.asked
            && asked.forge == forge
            && let Some(label) = &placed.label
        {
            return Some(label);
        }
    }
    None
}

/// A kind's slot names its bound forge in its arguments and output (temper/docs/design/protocol/setup/box.md, 1).
#[must_use]
pub fn bound(template: &Step, forge: u16) -> Step {
    let digits = Decimal::of(u64::from(forge));
    let mut args = List::with_capacity(u32::try_from(template.arguments().len()).expect("bounded arguments"));
    for argument in template.arguments() {
        args.push(if argument.as_ref() == FORGE_ARGUMENT {
            bytes::copy_of(digits.as_bytes())
        } else {
            argument.clone()
        })
        .expect("argument capacity");
    }
    let mut step = Step::new(
        crate::lines::joined(&[template.name(), b"-", digits.as_bytes()]),
        args.into_boxed(),
        template.kind(),
    )
    .listening(Box::from(template.listeners()));
    if template.ignores_ending() {
        step = step.ending_ignored();
    }
    if let Some(address) = template.ready() {
        step = step.with_ready(address);
    }
    step.set_started_notice(bytes::copy_of(template.started_notice()));
    step
}

/// A group's opaque label substitutes only its forge placeholder (temper/docs/design/protocol/setup/box.md, 3).
#[must_use]
pub fn bound_label(template: &[u8], forge: u16) -> Box<[u8]> {
    let digits = Decimal::of(u64::from(forge));
    let mut parts = List::uncapped();
    let mut rest = template;
    for _ in 0..template.len() {
        let Some(at) = bytes::find(rest, FORGE_ARGUMENT) else { break };
        parts.push(rest.get(..at).expect("placeholder prefix")).expect("label parts");
        parts.push(digits.as_bytes()).expect("label parts");
        rest = rest.get(at.checked_add(FORGE_ARGUMENT.len()).expect("placeholder end")..).expect("label suffix");
    }
    parts.push(rest).expect("label parts");
    crate::lines::joined(parts.as_slice())
}

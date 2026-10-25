//! One placement, shared by host and policy (protocol/setup.md, 7.3).
use alloc::boxed::Box;
use skein_lib::List;
use temper_forge_setup::{Stage, Step, Steps};
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Asked {
    pub forge: u16,
    pub recorded: bool,
    pub stage: Stage,
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
                        asked: Some(Asked { forge: group.forge, recorded: group.recorded, stage }),
                        label: label.take(),
                    })
                    .expect("step capacity");
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

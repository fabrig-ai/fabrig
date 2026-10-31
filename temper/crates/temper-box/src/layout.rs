//! One placement, shared by host and policy (protocol/setup.md, 7.3).
//! Runtime slots: temper/docs/design/draft/playground-reset.md, 3.
use alloc::boxed::Box;
use skein_lib::List;
use temper_forge_setup::{Group, Kind, Stage, Step, Steps};
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Asked {
    pub forge: u16,
    pub recorded: bool,
    pub stage: Stage,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Role {
    Always,
    Group,
    Removal,
    Vacant,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Placed {
    pub role: Role,
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
                        role: Role::Group,
                        step,
                        asked: Some(Asked { forge: group.forge, recorded: group.recorded, stage }),
                        label: label.take(),
                    })
                    .expect("step capacity");
                }
            }
        }
        if let Some(step) = always.next() {
            out.push(Placed { role: Role::Always, step, asked: None, label: None }).expect("step capacity");
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

/// Runtime groups occupy fixed slots just before the last always step (serve).
/// Each slot's last position holds its removal step.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Slots {
    pub first: usize,
    pub count: u32,
    pub width: u32,
}
impl Slots {
    pub const NONE: Self = Self { first: 0, count: 0, width: 0 };
    #[must_use]
    pub fn slot(&self, position: usize) -> Option<u32> {
        let offset = position.checked_sub(self.first)?;
        let width = usize::try_from(self.width).expect("slot width");
        if width == 0 {
            return None;
        }
        let slot = u32::try_from(offset.checked_div(width)?).ok()?;
        (slot < self.count).then_some(slot)
    }
    pub(crate) fn range(&self, slot: u32) -> core::ops::Range<usize> {
        assert!(slot < self.count, "slot exists");
        let offset = slot.checked_mul(self.width).expect("slot offset");
        let first = self.first.checked_add(usize::try_from(offset).expect("slot offset")).expect("slot first");
        first..first.checked_add(usize::try_from(self.width).expect("slot width")).expect("slot end")
    }
}
fn vacant() -> Placed {
    Placed { role: Role::Vacant, step: Step::new(Box::new([]), Box::new([]), Kind::Once), asked: None, label: None }
}
/// Places the always steps, the slots, then the last always step.
/// Given groups ignore their `before`; each must fit before its removal position.
#[must_use]
pub fn slotted(steps: Steps, groups: Box<[(Group, Step)]>, count: u32, width: u32) -> (Box<[Placed]>, Slots) {
    assert!(steps.asked.is_empty(), "slotted groups are supplied with removal steps");
    assert!(count == 0 || width > 0, "a slot holds a removal step");
    assert!(groups.len() <= usize::try_from(count).expect("slot count"), "groups fit slots");
    let first = steps.always.len().checked_sub(1).expect("last always step is serve");
    let positions = count.checked_mul(width).expect("slot positions");
    let capacity = u32::try_from(steps.always.len()).expect("always steps").checked_add(positions).expect("positions");
    let slots = Slots { first, count, width };
    let mut placed = List::with_capacity(capacity);
    for (index, step) in steps.always.into_iter().enumerate() {
        if index == first {
            for _ in 0..positions {
                placed.push(vacant()).expect("slot capacity");
            }
        }
        placed.push(Placed { role: Role::Always, step, asked: None, label: None }).expect("always capacity");
    }
    let mut placed = placed.into_boxed();
    for (slot, (group, remove)) in groups.into_iter().enumerate() {
        fill(&mut placed, &slots, u32::try_from(slot).expect("slot index"), group, remove);
    }
    (placed, slots)
}
/// Writes a group and its removal into a vacant slot.
pub fn fill(placed: &mut [Placed], slots: &Slots, slot: u32, group: Group, remove: Step) {
    let range = slots.range(slot);
    assert!(group.steps.len() < usize::try_from(slots.width).expect("slot width"), "group fits slot");
    for position in range.clone() {
        assert_eq!(placed.get(position).expect("slot position").role, Role::Vacant, "only a vacant slot is filled");
    }
    assert_eq!(remove.kind(), Kind::Once, "removal is once");
    let mut label = Some(group.label);
    let asked = Asked { forge: group.forge, recorded: group.recorded, stage: Stage::Run };
    for (offset, (stage, step)) in group.steps.into_iter().enumerate() {
        *placed.get_mut(range.start.checked_add(offset).expect("group position")).expect("group position") =
            Placed { role: Role::Group, step, asked: Some(Asked { stage, ..asked }), label: label.take() };
    }
    *placed.get_mut(range.end.checked_sub(1).expect("removal position")).expect("removal position") =
        Placed { role: Role::Removal, step: remove, asked: Some(asked), label };
}
/// Returns all positions in a slot to vacant placeholders.
pub fn clear(placed: &mut [Placed], slots: &Slots, slot: u32) {
    for position in slots.range(slot) {
        *placed.get_mut(position).expect("slot position") = vacant();
    }
}

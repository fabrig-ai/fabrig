//! A held row's judgement, without a map census (protocol/store/layouts.md, 5.3).
use super::{conversion, derive, frame_type, history_type};
use crate::registry::{Layouts, Row, Text, texts, variant};
use crate::{JobLayout, OwnerSets, Unplanned};
use alloc::boxed::Box;
use skein_codec::{
    layout::{Underived, Why, Workspace},
    schema::Span,
};
use skein_lib::List;

/// How a held layout reaches the binary. Jobs are taken as pending.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeldLayout {
    Planned,
    Converted { job: &'static str },
}

/// Judge a layout taken as held, using the open's row planner and log-held rule.
/// `kept` supplies the row's root, variant, text and held texts; all jobs in
/// `sets` are taken as pending. No census retirement exception is available.
/// Missing or malformed texts are refused, never covered by a written job.
pub fn judge_held_layout(
    sets: &OwnerSets,
    family: &str,
    number: u16,
    kept: &JobLayout,
) -> Result<HeldLayout, Unplanned> {
    let mut declaration = None;
    for set in sets.sets {
        for found in set.families {
            if found.name == family {
                for ty in found.types {
                    if ty.number == number {
                        declaration = Some(ty);
                    }
                }
            }
        }
    }
    let frame = frame_type(sets, family, number);
    let ty = match frame {
        Some((_, ty)) if kept.held.is_empty() => ty,
        Some(_) => return Err(Unplanned::Text),
        None => *declaration.ok_or(Unplanned::Undeclared)?,
    };
    let mut layouts = Layouts::new();
    let digest = insert(&mut layouts, &kept.text)?;
    let mut held = List::uncapped();
    for text in &kept.held {
        held.push(insert(&mut layouts, text)?).expect("uncapped held texts");
    }
    let Ok(mut binary) = texts::Binary::new(sets, &mut layouts) else { return Err(Unplanned::Text) };
    let fingerprint = binary
        .layout(&layouts, digest, kept.root.as_bytes(), variant(kept.variant.as_deref()), &held)
        .ok_or(Unplanned::Text)?;
    let row = Row {
        value: crate::registry_v1::Layout {
            root: Box::from(kept.root.as_bytes()),
            variant: match &kept.variant {
                Some(name) => Some(Box::from(name.as_bytes())),
                None => None,
            },
            layout: fingerprint,
            text: digest,
            held,
        },
        stored: true,
        retire: false,
    };
    let current = binary.text(family).ok_or(Unplanned::Text)?;
    let held =
        if frame.is_some() { List::uncapped() } else { binary.held(&mut layouts, current).ok_or(Unplanned::Text)? };
    let why = match derive(&mut binary, &layouts, &row, current, &ty, &held, None) {
        Ok(None) => return Ok(HeldLayout::Planned),
        Ok(Some(plan)) if plan.keeps() || Workspace::new().fit(&plan) => return Ok(HeldLayout::Planned),
        Ok(Some(_)) => {
            Unplanned::Underived(Underived { why: Why::Deep, decl: Span { start: 0, end: 0 }, part: None, old: false })
        }
        Err(why) => why,
    };
    if let Unplanned::Underived(_) = why
        && !history_type(sets, family, number)
    {
        for set in sets.sets {
            for job in set.jobs {
                if let Some(from) = conversion(job, family, number) {
                    let takes = match from {
                        crate::Layouts::One(from) => from == fingerprint,
                        crate::Layouts::Unplanned => true,
                    };
                    if takes {
                        return Ok(HeldLayout::Converted { job: job.name });
                    }
                }
            }
        }
    }
    Err(why)
}

fn insert(layouts: &mut Layouts, bytes: &[u8]) -> Result<super::super::Digest, Unplanned> {
    let digest = texts::digest(bytes);
    layouts
        .texts
        .insert(digest, Text { bytes: Box::from(bytes), schema: None, stored: true, current: false })
        .expect("uncapped texts");
    texts::parse(layouts, digest).ok_or(Unplanned::Text)?;
    Ok(digest)
}

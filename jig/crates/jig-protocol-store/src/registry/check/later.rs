//! Check later map layouts before the open can write (protocol/store/layouts.md, 5.6).
use crate::registry::{Layouts, ReadWork, RowKey, head};
use crate::{OpenRefusal, Store, Unkept, Unplanned, pages, restore};
use alloc::boxed::Box;
use skein_codec::layout::Misconverted;
use skein_lib::{List, Map};

#[derive(Debug)]
struct Layout {
    key: RowKey,
    values: Option<u64>,
    unkept: Option<Unkept>,
}

/// Keep the candidates by type so the walk reads each type's ranges once.
pub(in crate::registry) struct Check {
    types: Map<u32, Map<u16, Layout>>,
}
impl Check {
    pub(super) fn new() -> Self {
        Self { types: Map::uncapped() }
    }

    pub(super) fn add(&mut self, index: u32, key: &RowKey, values: Option<u64>) {
        if !self.types.contains_key(&index) {
            self.types.insert(index, Map::uncapped()).expect("uncapped check types");
        }
        self.types
            .get_mut(&index)
            .expect("type inserted")
            .insert(key.2, Layout { key: key.clone(), values, unkept: None })
            .expect("uncapped checked layouts");
    }

    pub(in crate::registry) fn run(
        self,
        store: &mut Store,
        layouts: &Layouts,
        work: &mut ReadWork,
    ) -> Result<(), OpenRefusal> {
        if self.types.is_empty() {
            return Ok(());
        }
        let mut rows = 0_u64;
        for space in store.stores.store.census().keyspaces() {
            rows = rows.checked_add(space.rows).ok_or(OpenRefusal::Failed)?;
        }
        let mut types = self.types;
        let mut refused: Option<(RowKey, Option<u64>, Unkept)> = None;
        for _ in 0..types.len() {
            let (&index, _) = types.first().expect("next checked type");
            let mut checks = types.remove(&index).expect("checked type");
            walk(store, layouts, index, &mut checks, work, rows)?;
            // Within a type the layout ids are ordered, but type indices are
            // not in registry key order.
            for (_, layout) in &checks {
                if let Some(unkept) = &layout.unkept {
                    if let Some((earlier, _, _)) = &refused
                        && *earlier <= layout.key
                    {
                        break;
                    }
                    refused = Some((layout.key.clone(), layout.values, unkept.clone()));
                    break;
                }
            }
        }
        if let Some(((family, value_type, id), values, unkept)) = refused {
            return Err(OpenRefusal::Layout { family, value_type, id, values, why: Unplanned::Unkept(unkept) });
        }
        Ok(())
    }
}

fn walk(
    store: &mut Store,
    layouts: &Layouts,
    index: u32,
    checks: &mut Map<u16, Layout>,
    work: &mut ReadWork,
    bound: u64,
) -> Result<(), OpenRefusal> {
    let mut ranges = List::uncapped();
    for (from, range) in &layouts.ranges {
        if range.ty == index {
            ranges.push(skein_kv::Range { start: from.clone(), end: Some(range.to.clone()) }).expect("uncapped ranges");
        }
    }
    let ty = layouts.types.get(index).expect("checked type");
    let mut after = None;
    for _ in 0..=bound {
        let page = match pages::reader::walk_stored(store, ranges.as_slice(), after.as_ref(), store.limits.store.ops) {
            Ok(page) => page,
            Err(failure) => {
                store.open_failed(restore::OpenFailure::Data(failure));
                return Err(OpenRefusal::Failed);
            }
        };
        if page.rows.is_empty() && page.next.is_some() {
            return Err(OpenRefusal::Failed);
        }
        for row in page.rows {
            let Some(id) = head(&row.row.value) else { continue };
            let Some(layout) = checks.get_mut(&id) else { continue };
            // A plan that keeps needs only a new head, as at the serving edge.
            let Some(plan) = ty.plans.get(&id) else {
                assert!(ty.keeping.as_slice().contains(&id), "each audited layout has a plan");
                continue;
            };
            match work.convert(plan, &row.row.value) {
                Ok(()) => {}
                Err(Misconverted::Refused(refusal)) => {
                    if let Some(unkept) = layout.unkept.as_mut() {
                        unkept.refused = unkept.refused.checked_add(1).ok_or(OpenRefusal::Failed)?;
                    } else {
                        let (why, declaration, part) = plan.refusal(refusal).expect("the plan names its refusal");
                        layout.unkept = Some(Unkept {
                            refused: 1,
                            why,
                            declaration: Box::from(declaration),
                            part: Box::from(part),
                        });
                    }
                }
                Err(
                    Misconverted::Short
                    | Misconverted::Tag
                    | Misconverted::Bool
                    | Misconverted::Bound
                    | Misconverted::Trailing
                    | Misconverted::Room,
                ) => {
                    let cause = restore::Cause::Decode(restore::DecodeFailure::Value);
                    let site = restore::site(store, &row.row.key);
                    store.open_failed(restore::OpenFailure::Data(restore::Failure::at(cause, site)));
                    return Err(OpenRefusal::Failed);
                }
            }
        }
        after = page.next;
        if after.is_none() {
            return Ok(());
        }
    }
    Err(OpenRefusal::Failed)
}

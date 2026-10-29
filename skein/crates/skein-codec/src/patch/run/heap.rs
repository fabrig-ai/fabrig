//! Heap admission includes each transient parser and each retained table (11.7).
use super::{Buffer, Capacity, Frame, List, Open, Patcher, Schema, Value, plan, schema};
use skein_lib::Stack;

impl Patcher {
    /// The old name of [`workspace_bound`]; it goes once every tree calls the new name (wave 3).
    #[must_use]
    pub fn worst_case_heap(capacity: Capacity) -> Option<u64> {
        Self::workspace_bound(capacity)
    }

    /// Bounds one patch operation.
    /// The run, two records, kept schemas and plans, including their scratch.
    #[must_use]
    pub fn workspace_bound(capacity: Capacity) -> Option<u64> {
        let mut total = Value::worst_case_heap(capacity.nodes)?;
        for bytes in [
            Buffer::worst_case(capacity.bytes)?,
            u64::from(capacity.bytes).checked_mul(2)?,
            Stack::<Frame>::worst_case(capacity.nodes)?,
            Stack::<Open>::worst_case(capacity.nodes)?,
            List::<Schema>::worst_case(capacity.versions)?,
            List::<plan::Plan>::worst_case(capacity.patches)?,
            schema::worst_case_heap(capacity.text)?.checked_mul(u64::from(capacity.versions))?,
            // plan reads the grammar even when no plan slot is available.
            plan::worst_case_heap(capacity.text)?.checked_mul(u64::from(capacity.patches.max(1)))?,
        ] {
            total = total.checked_add(bytes)?;
        }
        let mut tables = Value::worst_case_heap(capacity.nodes)?;
        for bytes in [
            Buffer::worst_case(capacity.bytes)?,
            Stack::<Frame>::worst_case(capacity.nodes)?,
            Stack::<Open>::worst_case(capacity.nodes)?,
        ] {
            tables = tables.checked_add(bytes)?;
        }
        total.checked_add(tables.checked_add(15)?.checked_div(16)?)
    }
}

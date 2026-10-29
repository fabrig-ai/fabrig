//! Heap admission includes each transient parser and each retained table (11.7).
use super::{Buffer, Capacity, Frame, List, Open, Patcher, Schema, Value, plan, schema};
use skein_lib::Stack;

impl Patcher {
    /// Bounds one patch operation.
    /// The run, two records, kept schemas and plans, including their scratch.
    #[must_use]
    pub fn workspace_bound(capacity: Capacity) -> Option<u64> {
        let mut total = Value::decoder_bound(capacity.nodes)?;
        for bytes in [
            Buffer::price(capacity.bytes)?,
            u64::from(capacity.bytes).checked_mul(2)?,
            Stack::<Frame>::price(capacity.nodes)?,
            Stack::<Open>::price(capacity.nodes)?,
            List::<Schema>::price(capacity.versions)?,
            List::<plan::Plan>::price(capacity.patches)?,
            schema::schema_bound(capacity.text)?.checked_mul(u64::from(capacity.versions))?,
            // plan reads the grammar even when no plan slot is available.
            plan::plan_bound(capacity.text)?.checked_mul(u64::from(capacity.patches.max(1)))?,
        ] {
            total = total.checked_add(bytes)?;
        }
        let mut tables = Value::decoder_bound(capacity.nodes)?;
        for bytes in [
            Buffer::price(capacity.bytes)?,
            Stack::<Frame>::price(capacity.nodes)?,
            Stack::<Open>::price(capacity.nodes)?,
        ] {
            tables = tables.checked_add(bytes)?;
        }
        total.checked_add(tables.checked_add(15)?.checked_div(16)?)
    }
}

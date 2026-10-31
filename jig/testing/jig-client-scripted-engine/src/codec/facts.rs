//! Ordered recursive fact tables (protocol/client.md, 3).
use super::{c, pieces, problem, values, w};
pub(super) struct Facts {
    rows: Vec<Option<w::Fact>>,
}
impl Facts {
    pub(super) fn chip(&mut self, value: &c::Chip, parent: Option<u32>, limits: &w::Limits) -> Result<u32, w::Problem> {
        let index = u32::try_from(self.rows.len()).map_err(|_| problem())?;
        self.rows.push(None);
        let row = w::Fact::Chip(w::ChipRow::new(
            limits,
            w::ChipRowParts { parent, value: values::wire_chip_fields(value, limits, self, index)? },
        )?);
        *self.rows.get_mut(usize::try_from(index).map_err(|_| problem())?).ok_or(problem())? = Some(row);
        Ok(index)
    }
    pub(super) fn chip_phase(
        &mut self,
        value: &c::ChipPhase,
        parent: Option<u32>,
        limits: &w::Limits,
    ) -> Result<u32, w::Problem> {
        let index = u32::try_from(self.rows.len()).map_err(|_| problem())?;
        self.rows.push(None);
        let row = w::Fact::ChipPhase(w::ChipPhaseRow::new(
            limits,
            w::ChipPhaseRowParts { parent, value: values::wire_chip_phase_owner_fields(value, limits, self, index)? },
        )?);
        *self.rows.get_mut(usize::try_from(index).map_err(|_| problem())?).ok_or(problem())? = Some(row);
        Ok(index)
    }
    pub(super) fn closing(
        &mut self,
        value: &c::Closing,
        parent: Option<u32>,
        limits: &w::Limits,
    ) -> Result<u32, w::Problem> {
        let index = u32::try_from(self.rows.len()).map_err(|_| problem())?;
        self.rows.push(None);
        let row = w::Fact::Closing(w::ClosingRow::new(
            limits,
            w::ClosingRowParts { parent, value: values::wire_closing_fields(value, limits, self, index)? },
        )?);
        *self.rows.get_mut(usize::try_from(index).map_err(|_| problem())?).ok_or(problem())? = Some(row);
        Ok(index)
    }
    pub(super) fn ending(
        &mut self,
        value: &c::Ending,
        parent: Option<u32>,
        limits: &w::Limits,
    ) -> Result<u32, w::Problem> {
        let index = u32::try_from(self.rows.len()).map_err(|_| problem())?;
        self.rows.push(None);
        let row = w::Fact::Ending(w::EndingRow::new(
            limits,
            w::EndingRowParts { parent, value: values::wire_ending_fields(value, limits, self, index)? },
        )?);
        *self.rows.get_mut(usize::try_from(index).map_err(|_| problem())?).ok_or(problem())? = Some(row);
        Ok(index)
    }
    pub(super) fn held_runtime(
        &mut self,
        value: &c::HeldRuntime,
        parent: Option<u32>,
        limits: &w::Limits,
    ) -> Result<u32, w::Problem> {
        let index = u32::try_from(self.rows.len()).map_err(|_| problem())?;
        self.rows.push(None);
        let row = w::Fact::HeldRuntime(w::HeldRuntimeRow::new(
            limits,
            w::HeldRuntimeRowParts { parent, value: values::wire_held_runtime_fields(value, limits, self, index)? },
        )?);
        *self.rows.get_mut(usize::try_from(index).map_err(|_| problem())?).ok_or(problem())? = Some(row);
        Ok(index)
    }
    pub(super) fn hold_reason(
        &mut self,
        value: &c::HoldReason,
        parent: Option<u32>,
        limits: &w::Limits,
    ) -> Result<u32, w::Problem> {
        let index = u32::try_from(self.rows.len()).map_err(|_| problem())?;
        self.rows.push(None);
        let row = w::Fact::HoldReason(w::HoldReasonRow::new(
            limits,
            w::HoldReasonRowParts { parent, value: values::wire_hold_reason_fields(value, limits, self, index)? },
        )?);
        *self.rows.get_mut(usize::try_from(index).map_err(|_| problem())?).ok_or(problem())? = Some(row);
        Ok(index)
    }
    pub(super) fn owner_chip_details(
        &mut self,
        value: &c::OwnerChipDetails,
        parent: Option<u32>,
        limits: &w::Limits,
    ) -> Result<u32, w::Problem> {
        let index = u32::try_from(self.rows.len()).map_err(|_| problem())?;
        self.rows.push(None);
        let row = w::Fact::OwnerChipDetails(Box::new(w::OwnerChipDetailsRow::new(
            limits,
            w::OwnerChipDetailsRowParts {
                parent,
                value: values::wire_owner_chip_details_fields(value, limits, self, index)?,
            },
        )?));
        *self.rows.get_mut(usize::try_from(index).map_err(|_| problem())?).ok_or(problem())? = Some(row);
        Ok(index)
    }
    pub(super) fn owner_chip_ending(
        &mut self,
        value: &c::OwnerChipEnding,
        parent: Option<u32>,
        limits: &w::Limits,
    ) -> Result<u32, w::Problem> {
        let index = u32::try_from(self.rows.len()).map_err(|_| problem())?;
        self.rows.push(None);
        let row = w::Fact::OwnerChipEnding(w::OwnerChipEndingRow::new(
            limits,
            w::OwnerChipEndingRowParts {
                parent,
                value: values::wire_owner_chip_ending_fields(value, limits, self, index)?,
            },
        )?);
        *self.rows.get_mut(usize::try_from(index).map_err(|_| problem())?).ok_or(problem())? = Some(row);
        Ok(index)
    }
    pub(super) fn result_value(
        &mut self,
        value: &c::ResultValue,
        parent: Option<u32>,
        limits: &w::Limits,
    ) -> Result<u32, w::Problem> {
        let index = u32::try_from(self.rows.len()).map_err(|_| problem())?;
        self.rows.push(None);
        let row = w::Fact::ResultValue(w::ResultValueRow::new(
            limits,
            w::ResultValueRowParts { parent, value: values::wire_result_value_fields(value, limits, self, index)? },
        )?);
        *self.rows.get_mut(usize::try_from(index).map_err(|_| problem())?).ok_or(problem())? = Some(row);
        Ok(index)
    }
}
pub(super) fn chip(value: &c::Chip, limits: &w::Limits) -> Result<w::Chip, w::Problem> {
    let mut facts = Facts { rows: Vec::new() };
    facts.chip(value, None, limits)?;
    let rows = pieces(facts.rows.into_iter().map(|row| row.ok_or(problem())), w::Fact::measure, w::Fact::encode)?;
    w::Chip::new(limits, w::ChipParts { rows })
}
pub(super) fn closing(value: &c::Closing, limits: &w::Limits) -> Result<w::Closing, w::Problem> {
    let mut facts = Facts { rows: Vec::new() };
    facts.closing(value, None, limits)?;
    let rows = pieces(facts.rows.into_iter().map(|row| row.ok_or(problem())), w::Fact::measure, w::Fact::encode)?;
    w::Closing::new(limits, w::ClosingParts { rows })
}
pub(super) fn ending(value: &c::Ending, limits: &w::Limits) -> Result<w::Ending, w::Problem> {
    let mut facts = Facts { rows: Vec::new() };
    facts.ending(value, None, limits)?;
    let rows = pieces(facts.rows.into_iter().map(|row| row.ok_or(problem())), w::Fact::measure, w::Fact::encode)?;
    w::Ending::new(limits, w::EndingParts { rows })
}
pub(super) fn held_runtime(value: &c::HeldRuntime, limits: &w::Limits) -> Result<w::HeldRuntime, w::Problem> {
    let mut facts = Facts { rows: Vec::new() };
    facts.held_runtime(value, None, limits)?;
    let rows = pieces(facts.rows.into_iter().map(|row| row.ok_or(problem())), w::Fact::measure, w::Fact::encode)?;
    w::HeldRuntime::new(limits, w::HeldRuntimeParts { rows })
}
pub(super) fn hold_reason(value: &c::HoldReason, limits: &w::Limits) -> Result<w::HoldReason, w::Problem> {
    let mut facts = Facts { rows: Vec::new() };
    facts.hold_reason(value, None, limits)?;
    let rows = pieces(facts.rows.into_iter().map(|row| row.ok_or(problem())), w::Fact::measure, w::Fact::encode)?;
    w::HoldReason::new(limits, w::HoldReasonParts { rows })
}
pub(super) fn owner_chip_ending(
    value: &c::OwnerChipEnding,
    limits: &w::Limits,
) -> Result<w::OwnerChipEnding, w::Problem> {
    let mut facts = Facts { rows: Vec::new() };
    facts.owner_chip_ending(value, None, limits)?;
    let rows = pieces(facts.rows.into_iter().map(|row| row.ok_or(problem())), w::Fact::measure, w::Fact::encode)?;
    w::OwnerChipEnding::new(limits, w::OwnerChipEndingParts { rows })
}
pub(super) fn result_value(value: &c::ResultValue, limits: &w::Limits) -> Result<w::ResultValue, w::Problem> {
    let mut facts = Facts { rows: Vec::new() };
    facts.result_value(value, None, limits)?;
    let rows = pieces(facts.rows.into_iter().map(|row| row.ok_or(problem())), w::Fact::measure, w::Fact::encode)?;
    w::ResultValue::new(limits, w::ResultValueParts { rows })
}

//! Wire fixtures for process stories; edits rebuild validated records without
//! reaching into a service (smith/docs/design/testing.md, sections 4 and 5).
use skein_lib::Reader;
use smith_channel::Record;

/// Encode one measured schema record at its final length.
#[macro_export]
macro_rules! encoded {
    ($record:expr, $limits:expr) => {{
        use smith_channel::Record as _;
        $record.encode($limits).expect("bounded record")
    }};
    ($record:expr) => {{
        let record = &$record;
        let mut writer = skein_lib::Writer::new(usize::try_from(record.measure()).expect("record size"));
        record.encode(&mut writer).expect("measured record");
        writer.finish()
    }};
}

/// Frame one schema record under the selected channel kind.
#[macro_export]
macro_rules! frame {
    ($kind:expr, $record:expr) => {{
        let record = &$record;
        let bytes = $crate::encoded!(record, &smith_channel::CEILINGS);
        let length = u32::try_from(bytes.len()).expect("record size");
        let mut frame = skein_channel::frame_writer($kind, length).expect("record frame");
        frame.put(&bytes).expect("measured frame body");
        frame.finish().expect("filled frame")
    }};
}

/// Decode, edit and validate one charter using the process world's wire ceilings.
#[must_use]
pub fn edit_charter(bytes: &[u8], edit: impl FnOnce(&mut smith_charter::Charter)) -> Box<[u8]> {
    let mut parts =
        smith_charter::Charter::decode(&smith_charter::CEILINGS, &mut Reader::new(bytes)).expect("fixture charter");
    edit(&mut parts);
    let charter = parts;
    crate::encoded!(charter, &smith_charter::CEILINGS)
}

/// Set a scenario's turn and spend bounds while keeping its time policy.
pub fn budget(parts: &mut smith_charter::Charter, turns: u32, spend: u64) {
    let mut budget = parts.budget.clone();
    budget.turns = turns;
    budget.spend = spend;
    parts.budget = budget;
}

/// Decode, edit and validate the body of an ordinary Start frame.
#[must_use]
pub fn edit_start(frame: &skein_channel::Frame, edit: impl FnOnce(&mut smith_channel::Start)) -> skein_channel::Frame {
    let body = frame.bytes().get(8..).expect("Start header");
    let mut parts =
        smith_channel::Start::decode(&smith_channel::CEILINGS, &mut Reader::new(body)).expect("fixture Start");
    edit(&mut parts);
    let start = parts;
    crate::frame!(0x0100, start)
}

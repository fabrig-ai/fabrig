//! History record format 1 (jig/docs/design/protocol/store/logs.md, 3).
use alloc::boxed::Box;
use skein_lib::{List, Reader, Writer};

pub type Row = (Box<[u8]>, Option<Box<[u8]>>);

/// Encode nonempty rows in strictly increasing key order.
#[must_use]
pub fn encode(rows: &[Row]) -> Option<Box<[u8]>> {
    if rows.is_empty() {
        return None;
    }
    let mut size = 1_usize;
    let mut previous: Option<&[u8]> = None;
    for (key, value) in rows {
        if let Some(previous) = previous
            && previous >= key.as_ref()
        {
            return None;
        }
        previous = Some(key);
        u16::try_from(key.len()).ok()?;
        size = size.checked_add(3)?.checked_add(key.len())?;
        if let Some(value) = value {
            u32::try_from(value.len()).ok()?;
            size = size.checked_add(4)?.checked_add(value.len())?;
        }
    }
    u32::try_from(size).ok()?;
    let mut writer = Writer::new(size);
    writer.put(&[1]).ok()?;
    for (key, value) in rows {
        writer.put(&u16::try_from(key.len()).ok()?.to_be_bytes()).ok()?;
        writer.put(key).ok()?;
        match value {
            Some(value) => {
                writer.put(&[1]).ok()?;
                writer.put(&u32::try_from(value.len()).ok()?.to_be_bytes()).ok()?;
                writer.put(value).ok()?;
            }
            None => writer.put(&[2]).ok()?,
        }
    }
    Some(writer.finish())
}

/// Decode one complete record, refusing unknown formats, kinds and unordered rows.
#[must_use]
pub fn decode(bytes: &[u8]) -> Option<Box<[Row]>> {
    let mut reader = Reader::new(bytes);
    if reader.u8()? != 1 || reader.is_empty() {
        return None;
    }
    let mut rows = List::uncapped();
    for _ in 0..reader.remaining() {
        if reader.is_empty() {
            break;
        }
        let length = u32::from(reader.u16()?);
        let key = Box::from(reader.bytes(length)?);
        if let Some((previous, _)) = rows.last()
            && previous >= &key
        {
            return None;
        }
        let value = match reader.u8()? {
            1 => {
                let length = reader.u32()?;
                Some(Box::from(reader.bytes(length)?))
            }
            2 => None,
            _ => return None,
        };
        rows.push((key, value)).expect("record bytes bound its row count");
    }
    Some(rows.into_boxed())
}

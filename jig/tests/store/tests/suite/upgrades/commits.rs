//! Independent SKVC fixture framing (skein/docs/design/data/kv.md, 3).
type Put = (Box<[u8]>, Box<[u8]>);
pub(crate) fn commit_keys(raw: &[u8]) -> Option<Vec<Put>> {
    let mut input = skein_lib::Reader::new(raw);
    if input.bytes(4)? != b"SKVC" {
        return None;
    }
    input.bytes(10)?; // format version and commit number
    let count = u32::from_be_bytes(input.bytes(4)?.try_into().ok()?);
    input.bytes(4)?; // body length
    let mut puts = Vec::new();
    for _ in 0..count {
        if input.bytes(1)? != [1] {
            return None;
        }
        let length = input.u32()?;
        let key = input.bytes(length)?.into();
        let length = input.u32()?;
        let value = input.bytes(length)?.into();
        puts.push((key, value));
    }
    input.bytes(4)?; // checksum
    if input.remaining() != 0 {
        return None;
    }
    Some(puts)
}

type Row = (Box<[u8]>, Box<[u8]>);

/// One source commit in skein's framed format (skein/docs/design/data/kv.md, 3).
pub(crate) fn source_commit(rows: &[Row], position: u64) -> Vec<u8> {
    let mut body = Vec::new();
    for (key, value) in rows {
        body.push(1);
        body.extend(u32::try_from(key.len()).expect("key length").to_be_bytes());
        body.extend(key.as_ref());
        body.extend(u32::try_from(value.len()).expect("value length").to_be_bytes());
        body.extend(value.as_ref());
    }
    let mut frame = b"SKVC".to_vec();
    frame.extend(1_u16.to_be_bytes());
    frame.extend(position.to_be_bytes());
    frame.extend(u32::try_from(rows.len()).expect("row count").to_be_bytes());
    frame.extend(u32::try_from(body.len()).expect("body length").to_be_bytes());
    frame.extend(body);
    let mut crc = !0_u32;
    for byte in &frame {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ if crc & 1 == 0 { 0 } else { 0x82f6_3b78 };
        }
    }
    frame.extend((!crc).to_be_bytes());
    frame
}

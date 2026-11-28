//! Commit frames in a fixture's map log (skein/docs/design/data/kv.md, 3).
use jig_protocol_store::transcripts;
use skein_kv::Op;
use skein_lib::Reader;

/// The first log segment of the active generation's store.
#[must_use]
pub fn initial_path(active: &[u8]) -> String {
    let generation = std::str::from_utf8(active).expect("active generation text").trim();
    format!("generations/{generation}/store/log-00000000000000000001")
}

type Frames = Vec<(u64, Box<[Op]>)>;

/// Read complete, checksummed commits newer than `after` and return the consumed byte count.
#[must_use]
pub fn frames(bytes: &[u8], after: u64) -> (Frames, usize) {
    let mut reader = Reader::new(bytes);
    let mut frames = Vec::new();
    let mut used = 0;
    while reader.remaining() >= 26 {
        let start = bytes.len() - reader.remaining() as usize;
        assert_eq!(reader.bytes(4), Some(b"SKVC".as_slice()), "real KV frame magic");
        assert_eq!(reader.u16(), Some(1), "real KV frame version");
        let number = reader.u64().expect("frame number");
        let count = reader.u32().expect("frame writes");
        let body = reader.u32().expect("frame body length");
        if reader.remaining() < body + 4 {
            break;
        }
        let end = (bytes.len() - reader.remaining() as usize) + body as usize;
        let mut ops = Vec::new();
        for _ in 0..count {
            let kind = reader.u8().expect("write kind");
            let len = reader.u32().expect("key length");
            let key = reader.bytes(len).expect("key");
            match kind {
                1 => {
                    let len = reader.u32().expect("value length");
                    let value = reader.bytes(len).expect("value");
                    if number > after {
                        ops.push(Op::Put { key: key.into(), value: value.into() });
                    }
                }
                2 => {
                    if number > after {
                        ops.push(Op::Erase { key: key.into() });
                    }
                }
                _ => panic!("real KV write kind"),
            }
        }
        assert_eq!((bytes.len() - reader.remaining() as usize), end, "whole frame operations");
        let expected = transcripts::frame::crc32c(&bytes[start..end]);
        assert_eq!(reader.u32(), Some(expected), "real KV frame checksum");
        used = bytes.len() - reader.remaining() as usize;
        if number > after {
            frames.push((number, ops.into_boxed_slice()));
        }
    }
    (frames, used)
}

/// Encode one commit containing only map operations.
#[must_use]
pub fn commit(number: u64, ops: &[Op]) -> Box<[u8]> {
    let mut body = Vec::new();
    for op in ops {
        let (tag, key, value) = match op {
            Op::Put { key, value } => (1, key, Some(value)),
            Op::Erase { key } => (2, key, None),
            Op::Append { .. } | Op::Drop { .. } => unreachable!("only map operations in a fixture commit"),
        };
        body.push(tag);
        body.extend(u32::try_from(key.len()).expect("key length").to_be_bytes());
        body.extend_from_slice(key);
        if let Some(value) = value {
            body.extend(u32::try_from(value.len()).expect("value length").to_be_bytes());
            body.extend_from_slice(value);
        }
    }
    let mut frame = b"SKVC".to_vec();
    frame.extend(1_u16.to_be_bytes());
    frame.extend(number.to_be_bytes());
    frame.extend(u32::try_from(ops.len()).expect("operation count").to_be_bytes());
    frame.extend(u32::try_from(body.len()).expect("body length").to_be_bytes());
    frame.extend(body);
    frame.extend(transcripts::frame::crc32c(&frame).to_be_bytes());
    frame.into_boxed_slice()
}

//! One bounded frame, checked in stream order (temper/docs/design/client/box-binary.md, 3, step 4;
//! temper/docs/design/protocol/setup/owned.md, 6).
use ruzstd::decoding::{BlockDecodingStrategy, FrameDecoder, errors::FrameDecoderError};
use skein_lib::sha256::Sha256;
use std::{
    error::Error,
    io::{self, Read, Write},
};
use temper_fetch::fetch::Unpacked;

const MAGIC: [u8; 4] = [0x28, 0xb5, 0x2f, 0xfd];
const CHECKSUM: u8 = 0x04;
const RESERVED: u8 = 0x08;
const SINGLE_SEGMENT: u8 = 0x20;

/// How much of the magic is needed before reporting damage.
#[derive(Clone, Copy)]
pub(crate) enum Magic {
    // The box download treats any incomplete magic as truncated.
    Complete,
    // A pinned forge fetch reports a mismatching byte as soon as it is read.
    Prefix,
}

#[derive(Debug)]
pub(crate) enum UnpackError {
    Input(io::Error),
    Output(io::Error),
}

#[derive(Debug)]
enum DecodeError {
    Fault(Unpacked),
    Io(UnpackError),
}

fn reading(why: io::Error) -> DecodeError {
    if why.kind() == io::ErrorKind::UnexpectedEof {
        DecodeError::Fault(Unpacked::Truncated)
    } else {
        DecodeError::Io(UnpackError::Input(why))
    }
}

fn decoding(why: FrameDecoderError) -> DecodeError {
    // An I/O error in the decoder's source chain is from reading the archive.
    let mut source = why.source();
    while let Some(cause) = source {
        if let Some(io) = cause.downcast_ref::<io::Error>() {
            if io.kind() == io::ErrorKind::UnexpectedEof {
                return DecodeError::Fault(Unpacked::Truncated);
            }
            return DecodeError::Io(UnpackError::Input(io::Error::new(io.kind(), why)));
        }
        source = cause.source();
    }
    DecodeError::Fault(Unpacked::Damaged)
}

struct Header {
    bytes: [u8; 18], // magic, descriptor, window, dictionary ID and content size at their maxima
    len: usize,
    checksum: bool,
    size: Option<u64>,
    window: u64,
}

fn header(input: &mut impl Read, bound: u64, magic: Magic) -> Result<Header, DecodeError> {
    let mut bytes = [0; 18];
    match magic {
        Magic::Complete => {
            input.read_exact(&mut bytes[..4]).map_err(reading)?;
            if bytes[..4] != MAGIC {
                return Err(DecodeError::Fault(Unpacked::Damaged));
            }
        }
        Magic::Prefix => {
            for (byte, expected) in bytes[..4].iter_mut().zip(MAGIC) {
                input.read_exact(std::slice::from_mut(byte)).map_err(reading)?;
                if *byte != expected {
                    return Err(DecodeError::Fault(Unpacked::Damaged));
                }
            }
        }
    }
    input.read_exact(&mut bytes[4..5]).map_err(reading)?;
    let descriptor = bytes[4];
    if descriptor & RESERVED != 0 {
        return Err(DecodeError::Fault(Unpacked::Damaged));
    }
    let single = descriptor & SINGLE_SEGMENT != 0;
    let mut at = 5;
    let mut window = 0;
    if !single {
        input.read_exact(&mut bytes[at..=at]).map_err(reading)?;
        let value = bytes[at];
        at += 1;
        let base = 1_u64 << (10 + (value >> 3));
        window = base + (base / 8) * u64::from(value & 7);
        if window > bound {
            return Err(DecodeError::Fault(Unpacked::Damaged));
        }
    }
    // A zero dictionary ID means no dictionary. The site supplies none.
    let dict_len = [0, 1, 2, 4][usize::from(descriptor & 3)];
    let dict = &mut bytes[at..at + dict_len];
    input.read_exact(dict).map_err(reading)?;
    if dict.iter().any(|byte| *byte != 0) {
        return Err(DecodeError::Fault(Unpacked::Damaged));
    }
    at += dict_len;
    let size_len = [usize::from(single), 2, 4, 8][usize::from(descriptor >> 6)];
    let len = at + size_len;
    input.read_exact(&mut bytes[at..len]).map_err(reading)?;
    let mut size_bytes = [0; 8];
    size_bytes[..size_len].copy_from_slice(&bytes[at..len]);
    let size = (size_len != 0).then(|| u64::from_le_bytes(size_bytes) + if size_len == 2 { 256 } else { 0 });
    if size.is_some_and(|size| size > bound) {
        return Err(DecodeError::Fault(Unpacked::TooLong));
    }
    if single {
        window = size.expect("single-segment header carries a size");
    }
    // Let the caller check the last block's output bound before reading the checksum.
    bytes[4] &= !CHECKSUM;
    Ok(Header { bytes, len, checksum: descriptor & CHECKSUM != 0, size, window })
}

fn decode(input: &mut impl Read, output: &mut impl Write, bound: u64, magic: Magic) -> Result<Unpacked, DecodeError> {
    let header = header(input, bound, magic)?;
    let mut decoder = FrameDecoder::new();
    // Replay the admitted header. The optional checksum bytes delimit the
    // frame; the caller trusts the packed or unpacked SHA-256 instead.
    decoder.init(&header.bytes[..header.len]).map_err(decoding)?;

    let mut hash = Sha256::new();
    let mut total = 0;
    let mut buffer = [0; 16_384];
    loop {
        let last = decoder.decode_blocks(&mut *input, BlockDecodingStrategy::UptoBlocks(1)).map_err(decoding)?;
        let available = u64::try_from(decoder.can_collect()).expect("buffer length fits u64");
        // Before the last block, a nonempty drain leaves exactly the window behind.
        // With no drain, no bytes have been written yet and the buffered output
        // fits the admitted window.
        let retained = if !last && available != 0 { header.window } else { 0 };
        if available + retained > bound.saturating_sub(total) {
            return Err(DecodeError::Fault(Unpacked::TooLong));
        }
        loop {
            let count = decoder.read(&mut buffer).map_err(decoding_read)?;
            if count == 0 {
                break;
            }
            let bytes = &buffer[..count];
            output.write_all(bytes).map_err(|why| DecodeError::Io(UnpackError::Output(why)))?;
            hash.update(bytes);
            total += u64::try_from(count).expect("chunk fits u64");
        }
        if last {
            break;
        }
    }
    if header.size.is_some_and(|size| size != total) {
        return Err(DecodeError::Fault(Unpacked::Damaged));
    }
    if header.checksum {
        input.read_exact(&mut [0; 4]).map_err(reading)?;
    }
    match input.read_exact(&mut [0]) {
        Ok(()) => Ok(Unpacked::Trailing),
        Err(why) if why.kind() == io::ErrorKind::UnexpectedEof => {
            Ok(Unpacked::File { digest: hash.finish(), bytes: total })
        }
        Err(why) => Err(DecodeError::Io(UnpackError::Input(why))),
    }
}

fn decoding_read(_: io::Error) -> DecodeError {
    // FrameDecoder::read only drains the already decoded bytes into our buffer.
    DecodeError::Fault(Unpacked::Damaged)
}

pub(crate) fn decode_zstd(
    mut input: impl Read,
    output: &mut impl Write,
    bound: u64,
    magic: Magic,
) -> Result<Unpacked, UnpackError> {
    match decode(&mut input, output, bound, magic) {
        Ok(file) | Err(DecodeError::Fault(file)) => Ok(file),
        Err(DecodeError::Io(why)) => Err(why),
    }
}

//! One-file report ZIPs as served by Forgejo (skein/docs/design/forge.md, 5.2).
use alloc::boxed::Box;
use skein_fake_forge_domain::api::Error;
use skein_lib::bytes::copy_of;
/// Writes the one-file archive served by the fake, with Forgejo's signed descriptor.
/// The complete archive must fit the caller's existing raw-answer bound.
pub(crate) fn write(path: &[u8], bytes: &[u8], deflated: bool, bound: u32) -> Result<Box<[u8]>, Error> {
    let bound = usize::try_from(bound).expect("u32 fits");
    let name = u16::try_from(path.len()).or(Err(Error::TooLarge))?;
    if path.is_empty() || path.ends_with(b"/") || bytes.len() > bound {
        return Err(Error::TooLarge);
    }
    let inflated = u32::try_from(bytes.len()).or(Err(Error::TooLarge))?;
    let payload = if deflated {
        let length = skein_http::gzip::bound(bytes.len()).ok_or(Error::TooLarge)?;
        let mut gzip = skein_lib::bytes::zeroed(length);
        let length = skein_http::gzip::encode(bytes, &mut gzip).ok_or(Error::TooLarge)?;
        let end = length.checked_sub(8).ok_or(Error::TooLarge)?;
        copy_of(gzip.get(10..end).ok_or(Error::Refused)?)
    } else {
        copy_of(bytes)
    };
    let compressed = u32::try_from(payload.len()).or(Err(Error::TooLarge))?;
    // Local header, signed descriptor, central header, and end record.
    let length = 30usize
        .checked_add(16)
        .ok_or(Error::TooLarge)?
        .checked_add(46)
        .ok_or(Error::TooLarge)?
        .checked_add(22)
        .ok_or(Error::TooLarge)?
        .checked_add(path.len().checked_mul(2).ok_or(Error::TooLarge)?)
        .ok_or(Error::TooLarge)?
        .checked_add(payload.len())
        .ok_or(Error::TooLarge)?;
    if length > bound {
        return Err(Error::TooLarge);
    }
    let mut out = skein_lib::Writer::new(length);
    let method = if deflated { 8u16 } else { 0 };
    let crc = crc(bytes);
    out.put(&0x0403_4b50u32.to_le_bytes()).or(Err(Error::TooLarge))?;
    for value in [20u16, 8, method, 0, 0] {
        out.put(&value.to_le_bytes()).or(Err(Error::TooLarge))?;
    }
    for _ in 0u8..3 {
        out.put(&0u32.to_le_bytes()).or(Err(Error::TooLarge))?;
    }
    out.put(&name.to_le_bytes()).or(Err(Error::TooLarge))?;
    out.put(&0u16.to_le_bytes()).or(Err(Error::TooLarge))?;
    out.put(path).or(Err(Error::TooLarge))?;
    out.put(&payload).or(Err(Error::TooLarge))?;
    for value in [0x0807_4b50u32, crc, compressed, inflated] {
        out.put(&value.to_le_bytes()).or(Err(Error::TooLarge))?;
    }
    let central = u32::try_from(out.written()).or(Err(Error::TooLarge))?;
    out.put(&0x0201_4b50u32.to_le_bytes()).or(Err(Error::TooLarge))?;
    for value in [20u16, 20, 8, method, 0, 0] {
        out.put(&value.to_le_bytes()).or(Err(Error::TooLarge))?;
    }
    for value in [crc, compressed, inflated] {
        out.put(&value.to_le_bytes()).or(Err(Error::TooLarge))?;
    }
    for value in [name, 0u16, 0, 0, 0] {
        out.put(&value.to_le_bytes()).or(Err(Error::TooLarge))?;
    }
    for _ in 0u8..2 {
        out.put(&0u32.to_le_bytes()).or(Err(Error::TooLarge))?;
    }
    out.put(path).or(Err(Error::TooLarge))?;
    let size = u32::try_from(out.written()).or(Err(Error::TooLarge))?.checked_sub(central).ok_or(Error::TooLarge)?;
    out.put(&0x0605_4b50u32.to_le_bytes()).or(Err(Error::TooLarge))?;
    for value in [0u16, 0, 1, 1] {
        out.put(&value.to_le_bytes()).or(Err(Error::TooLarge))?;
    }
    for value in [size, central] {
        out.put(&value.to_le_bytes()).or(Err(Error::TooLarge))?;
    }
    out.put(&0u16.to_le_bytes()).or(Err(Error::TooLarge))?;
    Ok(out.finish())
}

fn crc(bytes: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb8_8320 & 0u32.wrapping_sub(crc & 1));
        }
    }
    !crc
}

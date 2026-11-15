//! ZIP CRC-32 (forge/forgejo.md, 1.2).

#[cfg(test)]
pub(super) use skein_deflate::decode as deflate;

/// The CRC-32 of the body (ZIP APPNOTE, 4.4.7), using the reflected polynomial.
#[must_use]
pub(super) fn crc(body: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for byte in body {
        crc ^= u32::from(*byte);
        for _bit in 0..8_u32 {
            crc = if crc & 1 == 1 { (crc >> 1_u32) ^ 0xEDB8_8320 } else { crc >> 1_u32 };
        }
    }
    !crc
}

use std::collections::BTreeMap;
use std::fmt;

/// The directory name: sixteen lowercase hexadecimal ASCII bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Digest(pub [u8; 16]);

impl fmt::Display for Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(f, "{}", char::from(byte))?;
        }
        Ok(())
    }
}

pub(crate) fn of(files: &BTreeMap<String, Vec<u8>>) -> Digest {
    let mut state = skein_lib::sha256::Sha256::new();
    for (path, bytes) in files {
        state.update(path.as_bytes());
        state.update(&[0]);
        state.update(&u64::try_from(bytes.len()).expect("file length fits u64").to_le_bytes());
        state.update(bytes);
    }
    let hash = state.finish();
    let mut name = [0; 16];
    for (pair, byte) in name.chunks_exact_mut(2).zip(hash) {
        let [high, low] = skein_lib::hex::pair(byte, skein_lib::hex::Case::Lower);
        pair[0] = high;
        pair[1] = low;
    }
    Digest(name)
}

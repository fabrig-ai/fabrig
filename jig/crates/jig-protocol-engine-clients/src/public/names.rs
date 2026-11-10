//! Domain-separated permutations (jig/docs/design/domain/people/public-reading.md, 8).
use skein_lib::{CryptoRng, Secret};

#[derive(Debug)]
pub struct Names {
    key: Secret,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Of {
    Project,
    Task,
}
impl Names {
    #[must_use]
    pub fn new(key: &Secret) -> Self {
        assert!(key.len() == 32, "public names use a 32-byte key");
        Self { key: key.clone() }
    }
    #[must_use]
    pub fn project(&self, n: u32) -> u64 {
        self.identifier(u64::from(n), Of::Project)
    }
    #[must_use]
    pub fn task(&self, n: u64) -> u64 {
        self.identifier(n, Of::Task)
    }
    fn identifier(&self, n: u64, of: Of) -> u64 {
        let (mut left, mut right) = halves(n);
        for round in 0..4 {
            let next = left ^ self.round(right, round, of);
            left = right;
            right = next;
        }
        join(left, right)
    }
    #[must_use]
    pub fn number(&self, id: u64, of: Of) -> u64 {
        let (mut left, mut right) = halves(id);
        for round in (0..4).rev() {
            let next = right ^ self.round(left, round, of);
            right = left;
            left = next;
        }
        join(left, right)
    }
    fn round(&self, half: u32, round: u64, of: Of) -> u32 {
        let domain = match of {
            Of::Project => 1_u64,
            Of::Task => 2_u64,
        };
        let stream = (domain << 40_u32) | (round << 32_u32) | u64::from(half);
        let mut rng = CryptoRng::new(&self.key, stream).expect("32-byte public names key");
        let mut bytes = [0; 4];
        rng.fill(&mut bytes);
        u32::from_le_bytes(bytes)
    }
}
fn halves(n: u64) -> (u32, u32) {
    (u32::try_from(n >> 32).expect("upper half"), u32::try_from(n & u64::from(u32::MAX)).expect("lower half"))
}
fn join(left: u32, right: u32) -> u64 {
    (u64::from(left) << 32) | u64::from(right)
}

//! Standalone worker disk admission (temper/docs/design/protocol/configuration/worker.md).
use skein_lib::{Duration, Time};
use temper_shell::directories::EnvironmentEntry;

pub const DISK_BUDGET: skein_shell::Variable = skein_shell::Variable {
    name: "TEMPER_WORKER_DISK_BUDGET",
    set: skein_shell::Set::Public,
    way: skein_shell::Way::Reads,
    form: "RESERVE_BYTES,BYTES_PER_PLACE",
    line: "Keep filesystem space for active work before admitting another checkout.",
    absent: "Use the configured places.",
    notes: "A standalone worker samples available bytes on its data filesystem. Each occupied place reserves a full allowance.",
};

#[derive(Debug)]
pub(super) struct Budget {
    reserve: u64,
    per_place: std::num::NonZeroU64,
    pub(super) next: Time,
    last: Option<u32>,
}

impl Budget {
    pub(super) fn read(environment: &[EnvironmentEntry]) -> Result<Option<Self>, String> {
        DISK_BUDGET.value(environment).map(Self::parse).transpose()
    }

    fn parse(bytes: &[u8]) -> Result<Self, String> {
        let invalid =
            || format!("{}: expected reserve bytes, then a comma and positive bytes per place", DISK_BUDGET.name);
        let (reserve, per_place) =
            std::str::from_utf8(bytes).ok().and_then(|v| v.split_once(',')).ok_or_else(invalid)?;
        Ok(Self {
            reserve: reserve.parse().map_err(|_| invalid())?,
            per_place: per_place.parse().map_err(|_| invalid())?,
            next: Time::ZERO,
            last: None,
        })
    }

    fn places(&self, available: u64) -> u32 {
        u32::try_from(available.saturating_sub(self.reserve) / self.per_place).unwrap_or(u32::MAX)
    }

    pub(super) fn sample(&mut self, path: &std::path::Path, now: Time) -> u32 {
        self.next = now.saturating_add(Duration::from_secs(2));
        let space = rustix::fs::statvfs(path).map(|s| s.f_bavail.saturating_mul(s.f_frsize));
        let places = self.places(space.unwrap_or(0));
        if self.last != Some(places) {
            match space {
                Ok(bytes) if places == 0 => eprintln!(
                    "worker disk admission: waiting for disk, {bytes} bytes available; need {} reserve plus {} per place",
                    self.reserve, self.per_place
                ),
                Ok(bytes) => {
                    eprintln!("worker disk admission: {bytes} bytes available, {places} places within reserve");
                }
                Err(error) => eprintln!("worker disk admission: waiting for disk: {error}"),
            }
            self.last = Some(places);
        }
        places
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disk_places_keep_headroom_and_close_when_the_filesystem_cannot_be_read() {
        let mut budget = Budget::parse(b"20,10").unwrap();
        for (bytes, places) in [(0, 0), (29, 0), (30, 1), (39, 1), (40, 2)] {
            assert_eq!(budget.places(bytes), places);
        }
        assert!(Budget::parse(b"20,0").is_err());
        let scratch = skein_scratch::Scratch::new("disk-probe");
        assert_eq!(budget.sample(&scratch.path().join("absent"), Time::ZERO), 0);
        assert!(budget.sample(scratch.path(), budget.next) > 0);
    }
}

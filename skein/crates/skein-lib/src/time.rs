//! Time is data (programming-model.md, section 9): nanoseconds on a monotonic
//! clock that the shell or the simulator reads, never a step; and wall time, a
//! separate type, for things about the world.

/// A point in time, in nanoseconds since an arbitrary origin.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Time(u64);

/// A span of time, in nanoseconds.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Duration(u64);

/// A point in wall-clock time, in nanoseconds since the Unix epoch, read by the
/// shell or the simulator beside [`Time`] (programming-model.md, section 9).
///
/// It is for things about the world: a certificate's validity, a timestamp a
/// peer will read. It is a type of its own, with no arithmetic on spans, so
/// it is never confused with a monotonic `Time` and never arms a deadline:
/// the wall clock can jump.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Wall(u64);

/// A date of the proleptic Gregorian calendar, in UTC (skein/docs/design/runtime/lib.md, 8).
/// Public fields may describe no date.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Civil {
    pub year: u32,
    pub month: u8,
    pub day: u8,
}

impl Civil {
    #[must_use]
    pub fn valid(self) -> bool {
        let leap = self.year.is_multiple_of(4) && (!self.year.is_multiple_of(100) || self.year.is_multiple_of(400));
        let days = match self.month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 if leap => 29,
            2 => 28,
            0 | 13..=u8::MAX => return false,
        };
        self.day >= 1 && self.day <= days
    }

    /// Days from the epoch, negative before it, or `None` for no date.
    #[must_use]
    pub fn days(self) -> Option<i64> {
        if !self.valid() {
            return None;
        }
        let year = i64::from(self.year).checked_sub(i64::from(self.month <= 2))?;
        let era = year.div_euclid(400);
        let era_year = year.rem_euclid(400);
        let month = i64::from(self.month);
        let march_month = if month > 2 { month.checked_sub(3)? } else { month.checked_add(9)? };
        let year_day = march_month
            .checked_mul(153)?
            .checked_add(2)?
            .div_euclid(5)
            .checked_add(i64::from(self.day))?
            .checked_sub(1)?;
        let era_day = era_year
            .checked_mul(365)?
            .checked_add(era_year.div_euclid(4))?
            .checked_sub(era_year.div_euclid(100))?
            .checked_add(year_day)?;
        era.checked_mul(146_097)?.checked_add(era_day)?.checked_sub(719_468)
    }

    /// Midnight, or `None` for no date or a midnight outside `Wall`.
    #[must_use]
    pub fn wall(self) -> Option<Wall> {
        let days = u64::try_from(self.days()?).ok()?;
        Some(Wall::from_nanos(days.checked_mul(86_400)?.checked_mul(1_000_000_000)?))
    }

    /// The English three-byte name, or `None` outside 1 through 12.
    #[must_use]
    pub fn month_name(month: u8) -> Option<&'static [u8]> {
        const MONTHS: [&[u8]; 12] =
            [b"Jan", b"Feb", b"Mar", b"Apr", b"May", b"Jun", b"Jul", b"Aug", b"Sep", b"Oct", b"Nov", b"Dec"];
        MONTHS.get(usize::from(month.checked_sub(1)?)).copied()
    }
}

impl Wall {
    /// The Unix epoch, 1970-01-01T00:00:00Z.
    pub const EPOCH: Wall = Wall(0);

    #[must_use]
    pub const fn from_nanos(nanos: u64) -> Wall {
        Wall(nanos)
    }

    #[must_use]
    pub const fn as_nanos(self) -> u64 {
        self.0
    }

    /// Whole seconds since the epoch, as certificates and HTTP dates count.
    #[must_use]
    pub const fn as_secs(self) -> u64 {
        self.0.div_euclid(1_000_000_000)
    }

    /// The UTC date holding this time. Every `Wall` has one.
    #[must_use]
    pub fn civil(self) -> Civil {
        // March-based Gregorian eras; Wall holds fewer than 214,000 days,
        // so every intermediate fits and no arithmetic saturates.
        let days = self.as_secs().div_euclid(86_400).saturating_add(719_468);
        let era = days.div_euclid(146_097);
        let era_day = days.rem_euclid(146_097);
        let era_year = era_day
            .saturating_sub(era_day.div_euclid(1_460))
            .saturating_add(era_day.div_euclid(36_524))
            .saturating_sub(era_day.div_euclid(146_096))
            .div_euclid(365);
        let year_day = era_day.saturating_sub(
            era_year
                .saturating_mul(365)
                .saturating_add(era_year.div_euclid(4))
                .saturating_sub(era_year.div_euclid(100)),
        );
        let march_month = year_day.saturating_mul(5).saturating_add(2).div_euclid(153);
        let day =
            year_day.saturating_sub(march_month.saturating_mul(153).saturating_add(2).div_euclid(5)).saturating_add(1);
        let month = if march_month < 10 { march_month.saturating_add(3) } else { march_month.saturating_sub(9) };
        let year = era.saturating_mul(400).saturating_add(era_year).saturating_add(u64::from(month <= 2));
        Civil {
            year: u32::try_from(year).expect("Wall's year is at most 2554"),
            month: u8::try_from(month).expect("calendar month is at most 12"),
            day: u8::try_from(day).expect("calendar day is at most 31"),
        }
    }
}

impl Time {
    /// The origin.
    pub const ZERO: Time = Time(0);

    #[must_use]
    pub const fn from_nanos(nanos: u64) -> Time {
        Time(nanos)
    }

    #[must_use]
    pub const fn as_nanos(self) -> u64 {
        self.0
    }

    /// `span` later, or `None` past the end of time.
    #[must_use]
    pub const fn checked_add(self, span: Duration) -> Option<Time> {
        match self.0.checked_add(span.0) {
            Some(nanos) => Some(Time(nanos)),
            None => None,
        }
    }

    /// `span` later, or the end of time: for deadlines so far out that
    /// "never" is the right reading.
    #[must_use]
    pub const fn saturating_add(self, span: Duration) -> Time {
        Time(self.0.saturating_add(span.0))
    }

    /// The span from `earlier` to this time, or zero if `earlier` is later.
    #[must_use]
    pub const fn saturating_since(self, earlier: Time) -> Duration {
        Duration(self.0.saturating_sub(earlier.0))
    }
}

/// The earlier of two optional deadlines, `None` only when both are: an
/// absent deadline never falls due. A step holding several deadlines folds
/// each into its next one: `next = earliest(next, Some(deadline))`.
#[must_use]
pub const fn earliest(left: Option<Time>, right: Option<Time>) -> Option<Time> {
    match left {
        None => right,
        Some(left_time) => match right {
            None => left,
            Some(right_time) => {
                if right_time.0 < left_time.0 {
                    right
                } else {
                    left
                }
            }
        },
    }
}

impl Duration {
    pub const ZERO: Duration = Duration(0);

    #[must_use]
    pub const fn from_nanos(nanos: u64) -> Duration {
        Duration(nanos)
    }

    /// Saturates at the longest span: a configuration value that large means "never".
    #[must_use]
    pub const fn from_millis(millis: u64) -> Duration {
        Duration(millis.saturating_mul(1_000_000))
    }

    /// Saturates at the longest span: a configuration value that large means "never".
    #[must_use]
    pub const fn from_secs(secs: u64) -> Duration {
        Duration(secs.saturating_mul(1_000_000_000))
    }

    #[must_use]
    pub const fn as_nanos(self) -> u64 {
        self.0
    }

    /// Whole milliseconds, discarding submillisecond nanoseconds (skein/docs/design/runtime/lib.md, 8).
    #[must_use]
    pub const fn as_millis(self) -> u64 {
        self.0.div_euclid(1_000_000)
    }

    #[must_use]
    pub const fn checked_add(self, other: Duration) -> Option<Duration> {
        match self.0.checked_add(other.0) {
            Some(nanos) => Some(Duration(nanos)),
            None => None,
        }
    }

    #[must_use]
    pub const fn saturating_add(self, other: Duration) -> Duration {
        Duration(self.0.saturating_add(other.0))
    }

    #[must_use]
    pub const fn saturating_mul(self, factor: u64) -> Duration {
        Duration(self.0.saturating_mul(factor))
    }
}

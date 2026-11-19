//! The executable's first dispatch (runtime/kernel.md, 6.6; runtime/shell.md, 6.4).
//!
//! Marker: `--skein-reach-v1`, followed by spec, report and ruleset descriptor numbers.
//! Spec: LE u32 argument/environment/descriptor counts, then length-prefixed program,
//! arguments and environment bytes, then LE u32 child descriptors (at most 64 KiB).
//! Report: five-byte records, a tag (0 Restricted, 1 Restrict, 2 Command) and LE i32 errno.
//! Service and helper are the same executable; these private layouts cannot skew.

/// Dispatches a reach helper before the service reads any configuration.
/// Returns immediately for an ordinary invocation; a helper never returns.
pub fn dispatch() {
    #[cfg(target_os = "linux")]
    crate::linux::helper::dispatch();
}

#[derive(Debug)]
pub(crate) struct Launcher {
    pub(crate) executable: Box<[u8]>,
    #[cfg(target_os = "linux")]
    abi: Option<Result<u32, skein_io::kernel::Unenforced>>,
}

impl Launcher {
    pub(crate) fn new() -> Self {
        Self {
            executable: Box::from(b"/proc/self/exe".as_slice()),
            #[cfg(target_os = "linux")]
            abi: None,
        }
    }

    pub(crate) fn check(&mut self) -> Result<(), skein_io::kernel::Unenforced> {
        #[cfg(target_os = "linux")]
        {
            crate::linux::process::can_restrict(self)
        }
        #[cfg(target_os = "macos")]
        {
            Err(skein_io::kernel::Unenforced::Unsupported)
        }
    }

    #[cfg(target_os = "linux")]
    pub(crate) fn admit(&mut self, needs: u32) -> Result<(), skein_io::kernel::Unenforced> {
        let answer = *self.abi.get_or_insert_with(crate::linux::helper::abi);
        admit_abi(answer, needs)
    }
}

#[cfg(target_os = "linux")]
pub(crate) const MARKER: &str = "--skein-reach-v1";
#[cfg(target_os = "linux")]
pub(crate) const CAPACITY: u64 = 65_536;

#[cfg(target_os = "linux")]
pub(crate) fn admit_abi(
    answer: Result<u32, skein_io::kernel::Unenforced>,
    needs: u32,
) -> Result<(), skein_io::kernel::Unenforced> {
    let found = answer?;
    if found < needs { Err(skein_io::kernel::Unenforced::Abi { found, needs }) } else { Ok(()) }
}

#[cfg(target_os = "linux")]
pub(crate) fn abi_error(errno: i32) -> skein_io::kernel::Unenforced {
    use skein_io::kernel::Unenforced;
    match errno {
        libc::ENOSYS => Unenforced::Absent,
        libc::EOPNOTSUPP => Unenforced::Disabled,
        _ => Unenforced::Restrict { errno },
    }
}

#[cfg(target_os = "linux")]
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Spec {
    pub(crate) program: Box<[u8]>,
    pub(crate) args: Box<[Box<[u8]>]>,
    pub(crate) env: Box<[Box<[u8]>]>,
    pub(crate) named: Box<[u32]>,
}

#[cfg(target_os = "linux")]
impl Spec {
    pub(crate) fn encode(&self) -> Result<Vec<u8>, i32> {
        let mut bytes = Vec::new();
        for count in [self.args.len(), self.env.len(), self.named.len()] {
            bytes.extend_from_slice(&u32::try_from(count).map_err(|_| libc::E2BIG)?.to_le_bytes());
        }
        for entry in std::iter::once(&self.program).chain(self.args.iter()).chain(self.env.iter()) {
            bytes.extend_from_slice(&u32::try_from(entry.len()).map_err(|_| libc::E2BIG)?.to_le_bytes());
            bytes.extend_from_slice(entry);
        }
        for descriptor in &self.named {
            bytes.extend_from_slice(&descriptor.to_le_bytes());
        }
        if skein_lib::len_u64(bytes.len()) > CAPACITY {
            return Err(libc::E2BIG);
        }
        Ok(bytes)
    }

    pub(crate) fn decode(mut bytes: &[u8]) -> Result<Self, i32> {
        fn number(bytes: &mut &[u8]) -> Result<u32, i32> {
            let (head, tail) = bytes.split_at_checked(4).ok_or(libc::EINVAL)?;
            *bytes = tail;
            Ok(u32::from_le_bytes(head.try_into().map_err(|_| libc::EINVAL)?))
        }
        fn entry(bytes: &mut &[u8]) -> Result<Box<[u8]>, i32> {
            let length = usize::try_from(number(bytes)?).map_err(|_| libc::EINVAL)?;
            let (head, tail) = bytes.split_at_checked(length).ok_or(libc::EINVAL)?;
            *bytes = tail;
            Ok(Box::from(head))
        }
        if skein_lib::len_u64(bytes.len()) > CAPACITY {
            return Err(libc::E2BIG);
        }
        let arguments = number(&mut bytes)?;
        let environment = number(&mut bytes)?;
        let descriptors = number(&mut bytes)?;
        let program = entry(&mut bytes)?;
        let args = (0..arguments).map(|_| entry(&mut bytes)).collect::<Result<_, _>>()?;
        let env = (0..environment).map(|_| entry(&mut bytes)).collect::<Result<_, _>>()?;
        let named = (0..descriptors).map(|_| number(&mut bytes)).collect::<Result<_, _>>()?;
        if !bytes.is_empty() {
            return Err(libc::EINVAL);
        }
        Ok(Self { program, args, env, named })
    }
}

#[cfg(target_os = "linux")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Record {
    Restricted,
    Restrict(i32),
    Command(i32),
}

#[cfg(target_os = "linux")]
impl Record {
    pub(crate) fn encode(self) -> [u8; 5] {
        let (tag, errno) = match self {
            Self::Restricted => (0, 0),
            Self::Restrict(errno) => (1, errno),
            Self::Command(errno) => (2, errno),
        };
        let [a, b, c, d] = errno.to_le_bytes();
        [tag, a, b, c, d]
    }

    pub(crate) fn decode(bytes: &[u8]) -> Option<Self> {
        let [tag, a, b, c, d] = <[u8; 5]>::try_from(bytes).ok()?;
        let errno = i32::from_le_bytes([a, b, c, d]);
        match tag {
            0 if errno == 0 => Some(Self::Restricted),
            1 => Some(Self::Restrict(errno)),
            2 => Some(Self::Command(errno)),
            _ => None,
        }
    }
}

/// Inclusive gaps, including all descriptors above the last named one.
#[cfg(target_os = "linux")]
pub(crate) fn gaps(named: &[u32], report: u32) -> Vec<(u32, u32)> {
    let mut keep = vec![0, 1, 2, report];
    keep.extend_from_slice(named);
    keep.sort_unstable();
    keep.dedup();
    let mut from = 0_u32;
    let mut gaps = Vec::new();
    for descriptor in keep {
        if descriptor > from {
            gaps.push((from, descriptor.checked_sub(1).expect("descriptor above from")));
        }
        let Some(next) = descriptor.checked_add(1) else { return gaps };
        from = next;
    }
    gaps.push((from, u32::MAX));
    gaps
}

#[cfg(target_os = "linux")]
pub(crate) fn rule_rights(rights: skein_io::kernel::Rights, directory: bool) -> u64 {
    u64::from(if directory { rights.0 } else { rights.0 & skein_io::kernel::Rights::FILE.0 })
}

#[cfg(all(test, target_os = "linux"))]
mod tests;

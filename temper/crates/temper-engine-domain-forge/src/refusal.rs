//! Refusal causes cross the in-process root boundary (temper/docs/design/domain/forge.md, 5.1).
use alloc::boxed::Box;
use skein_lib::List;

/// Why a resource operation cannot proceed; local pressure is retried by the connector.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RefusalCause {
    /// A configured limit cannot contain the requested working set.
    Limit { member: &'static str, configured: u64, needed: u64 },
    /// The resource has no adopted repository.
    Unadopted,
    /// Branch segments do not form a resource name.
    InvalidName,
    /// The client or forge returned this typed error.
    Backend(skein_forge::api::Error),
}

impl RefusalCause {
    /// Explain the cause and the act that makes a release useful.
    #[must_use]
    pub fn words(self) -> Box<[u8]> {
        let mut words = List::uncapped();
        match self {
            Self::Limit { member, configured, needed } => {
                put(&mut words, member.as_bytes());
                put(&mut words, b" = ");
                put(&mut words, skein_lib::Decimal::of(configured).as_bytes());
                put(&mut words, b"; required usage ");
                put(&mut words, skein_lib::Decimal::of(needed).as_bytes());
                put(&mut words, b"; reduce the named work or raise this limit in the deployment configuration, restart, then release");
            }
            Self::Unadopted => put(&mut words, b"repository is not adopted; adopt the repository, then release"),
            Self::InvalidName => {
                put(&mut words, b"invalid branch name; correct the task's resource name, then release");
            }
            Self::Backend(error) => {
                put(&mut words, crate::effect_words::error_words(error));
                put(&mut words, recovery(error));
            }
        }
        words.into_boxed()
    }
}

fn put(out: &mut List<u8>, bytes: &[u8]) {
    for byte in bytes {
        out.push(*byte).expect("internal refusal words");
    }
}

fn recovery(error: skein_forge::api::Error) -> &'static [u8] {
    use skein_forge::api::Error;
    match error {
        Error::Forbidden | Error::Protected | Error::Refused => b"; restore forge access or permission, then release",
        Error::NotConnected | Error::NotInstalled => b"; reconnect the forge or accept its installation, then release",
        Error::Missing | Error::MissingJob => b"; restore the missing target or correct its name, then release",
        Error::Busy => b"; waiting for local admission; retries automatically",
        Error::TooLarge => b"; reduce the response or operation size, then release",
        Error::Unavailable
        | Error::Timeout
        | Error::RateLimited { .. }
        | Error::Empty
        | Error::Full
        | Error::Exists
        | Error::NothingToMerge
        | Error::Closed
        | Error::Stale
        | Error::Conflict
        | Error::InvalidAnswer => b"; resolve the reported forge failure, then release to read afresh",
    }
}

pub(crate) fn read(
    out: &mut skein_lib::Queue<crate::Request>,
    task: u64,
    operation: &'static [u8],
    repository: skein_forge::api::Repository,
    read: &skein_forge::api::Read,
    error: skein_forge::api::Error,
) {
    use skein_forge::api::Read;
    let what = match read {
        Read::Branch { branch } | Read::PullFor { head: branch, .. } => crate::What::Branch(parts(branch)),
        Read::Pull { number } => crate::What::Pull(*number),
        Read::Locate { .. }
        | Read::Items { .. }
        | Read::Item { .. }
        | Read::Reviews { .. }
        | Read::Statuses { .. }
        | Read::Remarks { .. }
        | Read::Branches { .. }
        | Read::PullFiles { .. }
        | Read::Compare { .. }
        | Read::Checks { .. }
        | Read::Job { .. }
        | Read::File { .. }
        | Read::Protection { .. }
        | Read::Settings
        | Read::Collaborators { .. }
        | Read::Permission { .. }
        | Read::Runs { .. }
        | Read::Run { .. }
        | Read::Tree { .. }
        | Read::Artifact { .. }
        | Read::Commits { .. } => crate::What::Repository,
    };
    out.push(crate::Request::ResourceRefused {
        task,
        operation,
        resource: Some(crate::Name { forge: repository.forge, repository: repository.repository, what }),
        cause: RefusalCause::Backend(error),
    });
}

fn parts(branch: &[u8]) -> Box<[Box<[u8]>]> {
    let mut parts = List::uncapped();
    let mut start = 0;
    for (index, byte) in branch.iter().enumerate() {
        if *byte == b'/' {
            parts.push(Box::from(branch.get(start..index).expect("branch segment"))).expect("bounded segments");
            start = index.saturating_add(1);
        }
    }
    parts.push(Box::from(branch.get(start..).expect("last segment"))).expect("bounded segments");
    parts.into_boxed()
}

pub(crate) fn limit(
    out: &mut skein_lib::Queue<crate::Request>,
    task: u64,
    repository: skein_forge::api::Repository,
    member: &'static str,
    configured: u32,
    needed: u64,
) {
    out.push(crate::Request::ResourceRefused {
        task,
        operation: b"prepare landing",
        resource: Some(crate::Name {
            forge: repository.forge,
            repository: repository.repository,
            what: crate::What::Repository,
        }),
        cause: RefusalCause::Limit { member, configured: u64::from(configured), needed },
    });
}

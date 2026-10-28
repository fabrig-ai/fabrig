//! A private native shell adopts its own parent while DIRECTORY keeps the lock.
use crate::{directory, host::Startup};
use jig_client_shell_native::Shell;
use skein_io::kernel::{Complete, Done, Fd, Op, OpenHow, Submit};
use skein_lib::{Duration, Queue, Secret, Time, Token};
const FIRST: Duration = Duration::from_millis(100);
const MOST: Duration = Duration::from_secs(2);
const OPEN: Token = Token::new(1 << 59);
#[derive(PartialEq, Eq)]
pub(crate) enum Credential {
    New,
    Writing,
    Kept,
}
pub(crate) struct Signing {
    pub(crate) shell: Option<Shell>,
    pub(crate) with: Option<Secret>,
    pub(crate) link: Option<Secret>,
    pub(crate) held: Option<Secret>,
    pub(crate) retry: Option<Time>,
    pub(crate) last: Option<jig_client::NotSignedIn>,
    backoff: Duration,
    pub(crate) address: Option<Box<[u8]>>,
    pub(crate) credential: Credential,
    pub(crate) printed: Option<Box<[u8]>>,
    pub(crate) error: bool,
    pub(crate) exchange: Option<Token>,
    pub(crate) opening: bool,
    closing: bool,
    startup: Startup,
}
impl Signing {
    pub(crate) fn held(
        engine: skein_io::Addr,
        parent: Fd,
        leaf: Box<[u8]>,
        startup: Startup,
        out: &mut Queue<Submit>,
        connections: jig_client_shell_native::Limits,
        io: skein_io::Limits,
    ) -> Self {
        out.push(Submit {
            op: OPEN,
            kind: Op::Open { root: parent, path: b".".as_slice().into(), how: OpenHow::ReadNoFollow },
        });
        Self {
            shell: Some(Shell::new(jig_client_shell_native::Settings {
                engine,
                directory: leaf,
                limits: connections,
                io,
            })),
            with: None,
            link: None,
            held: None,
            retry: None,
            last: None,
            backoff: FIRST,
            address: None,
            credential: Credential::Kept,
            printed: None,
            error: false,
            exchange: None,
            opening: true,
            closing: false,
            startup,
        }
    }
    pub(crate) fn new(
        bytes: &[u8],
        parent: Fd,
        leaf: Box<[u8]>,
        startup: Startup,
        out: &mut Queue<Submit>,
        connections: jig_client_shell_native::Limits,
        io: skein_io::Limits,
    ) -> Result<Self, &'static [u8]> {
        let bytes = bytes.strip_suffix(b"\n").unwrap_or(bytes);
        let parsed = jig_client_protocol_http::printed(
            &jig_client_protocol_http::Mapping::new(b"/jig").expect("mapping"),
            bytes,
        )
        .map_err(|_| b"the sign-in link was refused".as_slice())?;
        let mut address = b"http://".to_vec();
        address.extend_from_slice(&parsed.host);
        address.push(b'\n');
        let engine = directory::address(&address)?;
        let mut shell =
            Shell::new(jig_client_shell_native::Settings { engine, directory: leaf, limits: connections, io });
        let with = shell.sign_in(bytes).map_err(|_| b"the sign-in link was refused".as_slice())?;
        out.push(Submit {
            op: OPEN,
            kind: Op::Open { root: parent, path: b".".as_slice().into(), how: OpenHow::ReadNoFollow },
        });
        Ok(Self {
            shell: Some(shell),
            link: Some(with.clone()),
            with: Some(with),
            held: None,
            retry: None,
            last: None,
            backoff: FIRST,
            address: Some(address.into_boxed_slice()),
            credential: Credential::New,
            printed: None,
            error: false,
            exchange: None,
            opening: true,
            closing: false,
            startup,
        })
    }
    pub(crate) fn complete(&mut self, complete: Complete) {
        if complete.op != OPEN {
            self.shell.as_mut().expect("native shell").completions().push(complete);
            return;
        }
        self.opening = false;
        if let Ok(Done::Fd(fd)) = complete.result {
            let shell = self.shell.as_mut().expect("native shell");
            shell.adopt(fd, self.startup.user, self.startup.seed);
            if self.closing {
                shell.close();
            }
        } else {
            self.shell = None;
            self.error = true;
        }
    }
    pub(crate) fn again(&mut self, now: Time, until: Time) {
        self.retry = Some(now.saturating_add(self.backoff).min(until));
        self.backoff = self.backoff.saturating_mul(2).min(MOST);
    }
    pub(crate) fn close(&mut self) {
        self.link = None;
        self.retry = None;
        self.closing = true;
        if !self.opening
            && let Some(shell) = &mut self.shell
        {
            shell.close();
        }
    }
    pub(crate) fn is_empty(&self) -> bool {
        !self.opening && self.shell.as_ref().is_none_or(Shell::is_empty)
    }
}

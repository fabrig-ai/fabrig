use super::*;
use skein_io::kernel::{Kind, Stat};
use skein_lib::{List, Secret};
use skein_shell::{Origin, Unread};
use std::{cell::RefCell, collections::BTreeMap, io, path::PathBuf};
use temper_config_shell::{FILE, FORMER, Facts, Shape};

const CONFIG: &str = "/fixture/config";
const DATA: &str = "/fixture/data";
const CACHE: &str = "/fixture/cache";
const CLAUDE: &str = "/fixture/bin/claude";
const DOCUMENT: &[u8] = b"listen = \"127.0.0.1:7100\"\n[programs]\nclaude = \"/fixture/bin/claude\"\n";
const USER: u32 = 42;

struct Fixture {
    contents: BTreeMap<PathBuf, Box<[u8]>>,
    reads: RefCell<Vec<PathBuf>>,
}

impl Fixture {
    fn file(&self, path: &Path) -> Option<&[u8]> {
        self.reads.borrow_mut().push(path.into());
        self.contents.get(path).map(AsRef::as_ref)
    }
    fn bounded(&self, path: &Path, bound: u32) -> Result<Box<[u8]>, Unread> {
        let bytes = self.file(path).ok_or(Unread::Absent)?;
        if skein_lib::len_u64(bytes.len()) > u64::from(bound) {
            return Err(Unread::TooLarge { max: bound });
        }
        Ok(bytes.into())
    }
    fn existing_root(&self, path: &Path) -> Result<(), String> {
        self.reads.borrow_mut().push(path.into());
        assert!([Path::new(DATA), Path::new(CACHE), &Path::new(DATA).join("programs")].contains(&path));
        let stat = Stat { kind: Kind::Directory, size: 0, mode: 0o700, owner: USER, links: 1 };
        temper_shell::directories::private_root_with(USER, Ok(stat))
    }
}
impl Host for Fixture {
    fn read(&self, directories: &Directories) -> Result<configuration::Read, String> {
        let config = crate::path(&directories.config);
        configuration::read_with(
            temper_config_shell::read_with(
                Shape::Standalone,
                || {
                    assert!(self.file(&config.join(FORMER)).is_none());
                    Err(io::ErrorKind::NotFound.into())
                },
                |bound| self.bounded(&config.join(FILE), bound),
                || Ok(Facts { login: "ada".into(), hostname: "fixture".into(), provider_key: false }),
            ),
            || {
                self.existing_root(crate::path(&directories.data))?;
                self.existing_root(crate::path(&directories.cache))
            },
        )
    }
    fn executable(&self, path: &Path) -> bool {
        assert_eq!(path, Path::new(CLAUDE));
        self.file(path).is_some()
    }
    fn address(&self) -> Result<SocketAddr, String> {
        Ok(crate::scripted::PREFERRED)
    }
    fn given(&self, directories: &Directories, address: SocketAddr) -> service::stock::Given {
        crate::scripted::given_with(directories, &service::stock::Given::Stock, address, |path, defaults| {
            let mut log = Vec::new();
            let values = temper_service_shell::providers::read_using(path, defaults, &mut log, |path| {
                Ok(self.file(path).map(<[u8]>::to_vec))
            });
            assert!(log.is_empty(), "provider copy admitted");
            values
        })
    }
    fn agent(
        &self,
        bytes: &[u8],
        directories: &Directories,
        environment: &[crate::EnvironmentEntry],
        programs: &temper_config_shell::Programs,
    ) -> Result<Option<crate::agent::Prepared>, String> {
        let mut log = Vec::new();
        let agent = crate::serve::configured_agent_with(
            bytes,
            directories,
            environment,
            &mut log,
            |path, bound| {
                let copy = self.file(path);
                assert!(copy.is_none_or(|bytes| skein_lib::len_u64(bytes.len()) <= bound));
                Ok(copy.map(Into::into))
            },
            || Ok(self.given(directories, crate::scripted::address(environment)?)),
            |part| {
                let bytes = crate::agent::figures::configuration(
                    part.figures.as_ref().expect("standalone worker figures"),
                    programs.claude.as_deref(),
                    crate::path(&directories.data),
                    &part.environment,
                )?;
                crate::agent::prepare_bytes_with(bytes, &mut io::sink(), |directory| self.existing_root(directory))
            },
        )?;
        assert!(agent.is_some(), "restoring a box checks the worker's agent");
        assert!(log.is_empty(), "setting copies admitted");
        Ok(agent)
    }
    fn files(&self, named: &mut service::startup::Named) -> Result<engine::Inputs, String> {
        engine::before_root_using(
            &mut named.files,
            &mut named.asked_files,
            named.sign_in.as_ref(),
            named.bundle.as_deref(),
            named.bundle_limits,
            false,
            |required, asked| {
                temper_service_shell::read_files_with_asked_using(required, asked, USER, |path, bound, private| {
                    assert_eq!(path, Path::new(DATA).join("worker.secret"));
                    assert!(private);
                    self.bounded(path, bound).map(Secret::new)
                })
            },
            |_, _| panic!("no configured bundle"),
        )
    }
    fn runtimes(&self, named: &service::startup::Named) -> Result<engine::Runtimes, String> {
        let (roots, _) = skein_tls::Config::from_der(&[skein_tls_kit::pki::ROOT.into()], &[]).expect("fixture trust");
        let github = engine::github_runtime(named.test_github.as_ref(), &roots)?;
        Ok(engine::Runtimes { roots, github })
    }
    fn accounts(&self, _: &jig_protocol_config::Engine) -> Result<temper_engine_service::AccountsRuntime, String> {
        Ok(temper_engine_service::AccountsRuntime { providers: List::with_capacity(0), trust: None })
    }
    fn seeds(&self) -> Result<service::settings::Seeds, String> {
        Ok(service::settings::Seeds {
            deployment: [1; 16],
            installation: [2; 16],
            seed: 3,
            crypto: Secret::new(Box::new([4; 32])),
            link_secret: None,
            local_secret: None,
        })
    }
}

#[test]
fn checkpoint_checks_hand_their_request_to_the_engine() {
    let directories = Directories {
        config: CONFIG.as_bytes().into(),
        data: DATA.as_bytes().into(),
        cache: CACHE.as_bytes().into(),
        origins: [Origin::Flag; 3],
    };
    let config = Path::new(CONFIG);
    let data = Path::new(DATA);
    let state = data.join("state");
    let checkpoint = state.join("checkpoints/00000000000000000001-00000000000000ff");
    let source = crate::fake_backends::bytes(&checkpoint);
    let providers = temper_service_shell::providers::render(&[]);
    let agents = temper_agent_fixtures::Setting { figures: service::stock::agents(), providers: Box::new([]) }.bytes();
    let host = Fixture {
        contents: BTreeMap::from([
            (config.join(FILE), DOCUMENT.into()),
            (CLAUDE.into(), b"inert fixture program".as_slice().into()),
            (data.join("worker.secret"), b"synthetic-upgrade-worker-secret".as_slice().into()),
            (data.join("providers.setting"), providers),
            (data.join("agents.setting"), agents),
            (checkpoint.join("fixture"), b"existing checkpoint data".as_slice().into()),
        ]),
        reads: RefCell::default(),
    };
    let before = host.contents.clone();
    for request in [Request::Check(source.clone()), Request::Workers(source)] {
        let mut handed = Vec::new();
        let code = run_with(
            &directories,
            &request,
            |dirs| prepare_with(dirs, &host),
            |binary, dirs, received, service, deployment| {
                assert_eq!(binary, "temper-box");
                assert_eq!(dirs.flags(), directories.flags());
                assert!(service.operations() > 0, "the real preflight builds an engine service");
                assert_eq!(host.contents, before, "preflight makes no write under existing --data");
                let received = match received {
                    Request::Check(source) => Request::Check(source.clone()),
                    Request::Workers(source) => Request::Workers(source.clone()),
                    Request::Back => Request::Back,
                };
                handed.push((received, deployment.to_vec()));
                Ok(17)
            },
        )
        .unwrap_or_else(|why| panic!("{request:?}: {why}"));
        assert_eq!(code, 17, "return the prepared run's status");
        assert_eq!(handed, [(request, crate::fake_backends::bytes(&state).into_vec())]);
        assert_eq!(
            *host.reads.borrow(),
            [
                config.join(FORMER),
                config.join(FILE),
                data.into(),
                CACHE.into(),
                CLAUDE.into(),
                data.join("providers.setting"),
                data.join("agents.setting"),
                data.join("providers.setting"),
                data.join("local.setting"),
                data.join("programs"),
                data.join("worker.secret"),
            ]
        );
        host.reads.borrow_mut().clear();
    }
}

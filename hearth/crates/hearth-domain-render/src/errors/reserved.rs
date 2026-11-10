//! Closed reserved rules (hearth/docs/design/protocol/format/tasks.md, 11; hearth/docs/design/domain/machine/boot.md, 3.2).
use alloc::boxed::Box;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Reserved {
    pub rule: Rule,
    pub what: Box<[u8]>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Rule {
    ActivatorKey,
    RootKey,
    RootWait,
    CmdlineLength { bytes: u32 },
    PasswordPreseed,
    TrialName,
    Overlay,
    TmpfilesSubvolume,
    TmpfilesWrite,
    TmpfilesPath,
    ContainerModule,
    ContainerParam,
    ActivatorFile,
    Dropin,
    Accounts,
    SecretPath,
    HearthState,
    PackagesLayer,
    ClassMount,
    DataRoot,
    Runtime,
    LogFile,
    InitramfsConf,
    SecretMode,
    HearthOwner,
    LiveNone,
    NeverLiveUnit,
    NeverLivePath,
    HearthContainer,
    HearthNull,
}

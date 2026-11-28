//! Jig versions, independent of application versions; client/domain/contract/README.md, 6.
//! Compatibility is supplied by the protocol stream hello before page requests leave.
/// The owner of versioned records (client/domain/contract/README.md, 6).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Owner {
    Jig,
    Family(u16),
}
/// Nothing was read, made or kept in this owner's version.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NotServed {
    pub owner: Owner,
}
/// The admission scope of an application's request or subject.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    Project(u32),
    Deployment,
    Own,
}
/// The bounded page carried by a family read or watch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Window {
    pub rows: u32,
    pub after: Option<super::Cursor>,
}

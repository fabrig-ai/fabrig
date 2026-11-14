//! Declaration maps (hearth/docs/design/protocol/format/tasks.md, 1).

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Map {
    Packages,
    Preseeds,
    Initramfs,
    Software,
    Files,
    Users,
    Groups,
    Presets,
    Tmpfiles,
    Data,
    Secrets,
    Identity,
    Cmdline,
    Live,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Fragment {
    Customize,
    Software,
}
impl Fragment {
    #[must_use]
    pub const fn name(self) -> &'static [u8] {
        match self {
            Self::Customize => b"customize.sh",
            Self::Software => b"software.sh",
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Name {
    Action,
    Age,
    Algorithm,
    Argument,
    Base,
    Class,
    Component,
    Copy,
    Count,
    Directory,
    From,
    Gecos,
    Gid,
    Group,
    Groups,
    Home,
    Kind,
    Link,
    Mode,
    Order,
    Owner,
    Param,
    Password,
    Path,
    Paths,
    PublicPath,
    Sha256,
    Shell,
    Strip,
    Subordinate,
    Template,
    Type,
    Uid,
    Unit,
    Unpack,
    Url,
    Value,
}
impl Name {
    #[must_use]
    pub const fn takes(self) -> crate::errors::Takes {
        match self {
            Self::Action
            | Self::Age
            | Self::Algorithm
            | Self::Argument
            | Self::Class
            | Self::Component
            | Self::Copy
            | Self::From
            | Self::Gecos
            | Self::Group
            | Self::Home
            | Self::Kind
            | Self::Link
            | Self::Mode
            | Self::Owner
            | Self::Param
            | Self::Password
            | Self::Path
            | Self::PublicPath
            | Self::Sha256
            | Self::Shell
            | Self::Template
            | Self::Type
            | Self::Unit
            | Self::Unpack
            | Self::Url
            | Self::Value => crate::errors::Takes::Text,
            Self::Base | Self::Count | Self::Gid | Self::Order | Self::Strip | Self::Uid => crate::errors::Takes::Int,
            Self::Directory => crate::errors::Takes::Bool,
            Self::Groups | Self::Paths => crate::errors::Takes::Strings,
            Self::Subordinate => crate::errors::Takes::Range,
        }
    }
    #[must_use]
    pub const fn key(self) -> &'static [u8] {
        match self {
            Self::Action => b"action",
            Self::Age => b"age",
            Self::Algorithm => b"algorithm",
            Self::Argument => b"argument",
            Self::Base => b"base",
            Self::Class => b"class",
            Self::Component => b"component",
            Self::Copy => b"copy",
            Self::Count => b"count",
            Self::Directory => b"directory",
            Self::From => b"from",
            Self::Gecos => b"gecos",
            Self::Gid => b"gid",
            Self::Group => b"group",
            Self::Groups => b"groups",
            Self::Home => b"home",
            Self::Kind => b"kind",
            Self::Link => b"link",
            Self::Mode => b"mode",
            Self::Order => b"order",
            Self::Owner => b"owner",
            Self::Param => b"param",
            Self::Password => b"password",
            Self::Path => b"path",
            Self::Paths => b"paths",
            Self::PublicPath => b"public_path",
            Self::Sha256 => b"sha256",
            Self::Shell => b"shell",
            Self::Strip => b"strip",
            Self::Subordinate => b"subordinate",
            Self::Template => b"template",
            Self::Type => b"type",
            Self::Uid => b"uid",
            Self::Unit => b"unit",
            Self::Unpack => b"unpack",
            Self::Url => b"url",
            Self::Value => b"value",
        }
    }
}

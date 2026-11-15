//! The listing and typed documents (hearth/docs/design/protocol/format/README.md, 2.1 and 3.4).
use alloc::boxed::Box;
pub mod declaration;
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Oid(pub Box<[u8]>);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Position {
    pub line: u32,
    pub column: u32,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Runs {
    Metal,
    Vm,
    Container,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Firmware {
    Efi,
    Bios,
}
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Mode {
    File,
    Executable,
    Link,
    Submodule,
}

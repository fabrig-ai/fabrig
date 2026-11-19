//! Composable field codecs for generated records (data/codec/README.md, 4).
//! Types fix the nesting depth; no peer selects a recursive call.

use crate::{Path, Problem, Reason, Record, read_count, read_len, text_is_valid};
use alloc::boxed::Box;
use skein_lib::{Duration, List, Reader, Secret, Writer};

mod sealed {
    use alloc::boxed::Box;

    pub trait Sealed<L, T> {}

    impl<L> Sealed<L, u8> for super::U8 {}
    impl<L> Sealed<L, u16> for super::U16 {}
    impl<L> Sealed<L, u32> for super::U32 {}
    impl<L> Sealed<L, u64> for super::U64 {}
    impl<L> Sealed<L, bool> for super::Boolean {}
    impl<L> Sealed<L, super::Duration> for super::Span {}
    impl<L, const N: usize> Sealed<L, [u8; N]> for super::Fixed<N> {}
    impl<L> Sealed<L, Box<[u8]>> for super::Bytes {}
    impl<L> Sealed<L, super::Secret> for super::Hidden {}
    impl<L> Sealed<L, Box<[u8]>> for super::Text {}
    impl<L, T: super::Record<Limits = L>> Sealed<L, T> for super::Nested {}
    impl<L, T, F: super::Field<L, T>> Sealed<L, super::List<T>> for super::Sequence<F> {}
    impl<L, T, F: super::Field<L, T>> Sealed<L, Option<T>> for super::Optional<F> {}
}

/// A shape's typed operations, shared by the generic composition helpers.
pub trait Field<L, T>: sealed::Sealed<L, T> {
    fn write(&self, value: &T, limits: &L, out: &mut Writer, path: Path<L>) -> Result<(), Problem<L>>;
    fn read(&self, limits: &L, reader: &mut Reader<'_>, path: Path<L>) -> Result<T, Problem<L>>;
}

/// Writes one field, checking its bounds before its contents.
pub fn write<L, T, F: Field<L, T>>(
    shape: F,
    value: &T,
    limits: &L,
    out: &mut Writer,
    path: Path<L>,
) -> Result<(), Problem<L>> {
    shape.write(value, limits, out, path)
}

/// Reads one field, checking lengths before allocation.
pub fn read<L, T, F: Field<L, T>>(
    shape: F,
    limits: &L,
    reader: &mut Reader<'_>,
    path: Path<L>,
) -> Result<T, Problem<L>> {
    shape.read(limits, reader, path)
}

/// Writes bytes within the measured output, naming the field on overflow.
pub fn put<L>(out: &mut Writer, bytes: &[u8], path: Path<L>) -> Result<(), Problem<L>> {
    match out.put(bytes) {
        Ok(()) => Ok(()),
        Err(skein_lib::Overflow) => Err(path.problem(Reason::Bound)),
    }
}

/// Reads a version prefix before any of the record's fields.
pub fn version<L>(reader: &mut Reader<'_>, expected: u16, path: Path<L>) -> Result<(), Problem<L>> {
    let found = reader.u16().ok_or(path.problem(Reason::Short))?;
    if found != expected {
        return Err(path.problem(Reason::Version));
    }
    Ok(())
}

/// A big-endian `u8` field.
#[derive(Clone, Copy, Debug)]
pub struct U8;
impl<L> Field<L, u8> for U8 {
    fn write(&self, value: &u8, _limits: &L, out: &mut Writer, path: Path<L>) -> Result<(), Problem<L>> {
        put(out, &value.to_be_bytes(), path)
    }
    fn read(&self, _limits: &L, reader: &mut Reader<'_>, path: Path<L>) -> Result<u8, Problem<L>> {
        reader.u8().ok_or(path.problem(Reason::Short))
    }
}

/// A big-endian `u16` field.
#[derive(Clone, Copy, Debug)]
pub struct U16;
impl<L> Field<L, u16> for U16 {
    fn write(&self, value: &u16, _limits: &L, out: &mut Writer, path: Path<L>) -> Result<(), Problem<L>> {
        put(out, &value.to_be_bytes(), path)
    }
    fn read(&self, _limits: &L, reader: &mut Reader<'_>, path: Path<L>) -> Result<u16, Problem<L>> {
        reader.u16().ok_or(path.problem(Reason::Short))
    }
}

/// A big-endian `u32` field.
#[derive(Clone, Copy, Debug)]
pub struct U32;
impl<L> Field<L, u32> for U32 {
    fn write(&self, value: &u32, _limits: &L, out: &mut Writer, path: Path<L>) -> Result<(), Problem<L>> {
        put(out, &value.to_be_bytes(), path)
    }
    fn read(&self, _limits: &L, reader: &mut Reader<'_>, path: Path<L>) -> Result<u32, Problem<L>> {
        reader.u32().ok_or(path.problem(Reason::Short))
    }
}

/// A big-endian `u64` field.
#[derive(Clone, Copy, Debug)]
pub struct U64;
impl<L> Field<L, u64> for U64 {
    fn write(&self, value: &u64, _limits: &L, out: &mut Writer, path: Path<L>) -> Result<(), Problem<L>> {
        put(out, &value.to_be_bytes(), path)
    }
    fn read(&self, _limits: &L, reader: &mut Reader<'_>, path: Path<L>) -> Result<u64, Problem<L>> {
        reader.u64().ok_or(path.problem(Reason::Short))
    }
}

/// A canonical boolean byte.
#[derive(Clone, Copy, Debug)]
pub struct Boolean;
impl<L> Field<L, bool> for Boolean {
    fn write(&self, value: &bool, _limits: &L, out: &mut Writer, path: Path<L>) -> Result<(), Problem<L>> {
        put(out, &[u8::from(*value)], path)
    }
    fn read(&self, _limits: &L, reader: &mut Reader<'_>, path: Path<L>) -> Result<bool, Problem<L>> {
        match reader.u8().ok_or(path.problem(Reason::Short))? {
            0 => Ok(false),
            1 => Ok(true),
            2..=u8::MAX => Err(path.problem(Reason::Bool)),
        }
    }
}

/// A nanosecond duration.
#[derive(Clone, Copy, Debug)]
pub struct Span;
impl<L> Field<L, Duration> for Span {
    fn write(&self, value: &Duration, _limits: &L, out: &mut Writer, path: Path<L>) -> Result<(), Problem<L>> {
        put(out, &value.as_nanos().to_be_bytes(), path)
    }
    fn read(&self, _limits: &L, reader: &mut Reader<'_>, path: Path<L>) -> Result<Duration, Problem<L>> {
        Ok(Duration::from_nanos(reader.u64().ok_or(path.problem(Reason::Short))?))
    }
}

/// A schema-sized byte array.
#[derive(Clone, Copy, Debug)]
pub struct Fixed<const N: usize>;
impl<L, const N: usize> Field<L, [u8; N]> for Fixed<N> {
    fn write(&self, value: &[u8; N], _limits: &L, out: &mut Writer, path: Path<L>) -> Result<(), Problem<L>> {
        put(out, value, path)
    }
    fn read(&self, _limits: &L, reader: &mut Reader<'_>, path: Path<L>) -> Result<[u8; N], Problem<L>> {
        let length = u32::try_from(N).expect("schema fixed width fits u32");
        Ok(reader.bytes(length).ok_or(path.problem(Reason::Short))?.try_into().expect("fixed width checked"))
    }
}

fn check_length<L>(length: usize, limit: u32, path: Path<L>) -> Result<u32, Problem<L>> {
    let Ok(length) = u32::try_from(length) else {
        return Err(path.problem(Reason::Bound));
    };
    if length > limit {
        return Err(path.problem(Reason::Bound));
    }
    Ok(length)
}

/// A byte string with a limit already clamped to its ceiling.
#[derive(Clone, Copy, Debug)]
pub struct Bytes {
    pub limit: u32,
}
impl<L> Field<L, Box<[u8]>> for Bytes {
    fn write(&self, value: &Box<[u8]>, _limits: &L, out: &mut Writer, path: Path<L>) -> Result<(), Problem<L>> {
        write_bytes(value, self.limit, out, path)
    }
    fn read(&self, _limits: &L, reader: &mut Reader<'_>, path: Path<L>) -> Result<Box<[u8]>, Problem<L>> {
        read_bytes(reader, self.limit, path)
    }
}

fn write_bytes<L>(value: &[u8], limit: u32, out: &mut Writer, path: Path<L>) -> Result<(), Problem<L>> {
    let length = check_length(value.len(), limit, path)?;
    put(out, &length.to_be_bytes(), path)?;
    put(out, value, path)
}

fn read_bytes<L>(reader: &mut Reader<'_>, limit: u32, path: Path<L>) -> Result<Box<[u8]>, Problem<L>> {
    let length = match read_len(reader, limit) {
        Ok(length) => length,
        Err(reason) => return Err(path.problem(reason)),
    };
    Ok(Box::from(reader.bytes(length).expect("read_len checked the available bytes")))
}

/// Secret bytes, exposed only while writing.
#[derive(Clone, Copy, Debug)]
pub struct Hidden {
    pub limit: u32,
}
impl<L> Field<L, Secret> for Hidden {
    fn write(&self, value: &Secret, _limits: &L, out: &mut Writer, path: Path<L>) -> Result<(), Problem<L>> {
        write_bytes(value.expose(), self.limit, out, path)
    }
    fn read(&self, _limits: &L, reader: &mut Reader<'_>, path: Path<L>) -> Result<Secret, Problem<L>> {
        Ok(Secret::new(read_bytes(reader, self.limit, path)?))
    }
}

/// UTF-8 bytes, checked after their length and before allocation.
#[derive(Clone, Copy, Debug)]
pub struct Text {
    pub limit: u32,
}
impl<L> Field<L, Box<[u8]>> for Text {
    fn write(&self, value: &Box<[u8]>, _limits: &L, out: &mut Writer, path: Path<L>) -> Result<(), Problem<L>> {
        let length = check_length(value.len(), self.limit, path)?;
        if !text_is_valid(value) {
            return Err(path.problem(Reason::Utf8));
        }
        put(out, &length.to_be_bytes(), path)?;
        put(out, value, path)
    }
    fn read(&self, _limits: &L, reader: &mut Reader<'_>, path: Path<L>) -> Result<Box<[u8]>, Problem<L>> {
        match crate::read_text(reader, self.limit) {
            Ok(value) => Ok(value),
            Err(reason) => Err(path.problem(reason)),
        }
    }
}

/// A list checked by its count, not its in-memory capacity.
#[derive(Clone, Copy, Debug)]
pub struct Sequence<F> {
    pub limit: u32,
    pub minimum: u32,
    pub item: F,
}
impl<L, T, F: Field<L, T>> Field<L, List<T>> for Sequence<F> {
    fn write(&self, value: &List<T>, limits: &L, out: &mut Writer, path: Path<L>) -> Result<(), Problem<L>> {
        if value.len() > self.limit {
            return Err(path.problem(Reason::Bound));
        }
        put(out, &value.len().to_be_bytes(), path)?;
        for item in value.as_slice() {
            self.item.write(item, limits, out, path)?;
        }
        Ok(())
    }
    fn read(&self, limits: &L, reader: &mut Reader<'_>, path: Path<L>) -> Result<List<T>, Problem<L>> {
        let count = match read_count(reader, self.limit) {
            Ok(count) => count,
            Err(reason) => return Err(path.problem(reason)),
        };
        if let Some(available) = reader.remaining().checked_div(self.minimum)
            && count > available
        {
            return Err(path.problem(Reason::Short));
        }
        let mut items = List::allocated(count);
        for _ in 0..count {
            let item = self.item.read(limits, reader, path)?;
            assert!(items.push(item).is_ok(), "count within capacity");
        }
        Ok(items)
    }
}

/// An option's tag and optional value share their enclosing field's path.
#[derive(Clone, Copy, Debug)]
pub struct Optional<F> {
    pub item: F,
}
impl<L, T, F: Field<L, T>> Field<L, Option<T>> for Optional<F> {
    fn write(&self, value: &Option<T>, limits: &L, out: &mut Writer, path: Path<L>) -> Result<(), Problem<L>> {
        put(out, &[u8::from(value.is_some())], path)?;
        if let Some(item) = value {
            self.item.write(item, limits, out, path)?;
        }
        Ok(())
    }
    fn read(&self, limits: &L, reader: &mut Reader<'_>, path: Path<L>) -> Result<Option<T>, Problem<L>> {
        match reader.u8().ok_or(path.problem(Reason::Short))? {
            0 => Ok(None),
            1 => Ok(Some(self.item.read(limits, reader, path)?)),
            2..=u8::MAX => Err(path.problem(Reason::Tag)),
        }
    }
}

/// A nested declaration uses its own field paths, without an outer tail check.
#[derive(Clone, Copy, Debug)]
pub struct Nested;
impl<L, T: Record<Limits = L>> Field<L, T> for Nested {
    fn write(&self, value: &T, limits: &L, out: &mut Writer, _path: Path<L>) -> Result<(), Problem<L>> {
        value.write(limits, out)
    }
    fn read(&self, limits: &L, reader: &mut Reader<'_>, _path: Path<L>) -> Result<T, Problem<L>> {
        T::read(limits, reader)
    }
}

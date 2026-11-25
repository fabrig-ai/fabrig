//! Schema type comparison for layout derivation (skein/docs/design/data/codec/layouts.md, 2).

use super::add;
use crate::schema::{DeclIx, Held, Schema, Type, TypeIx};
use skein_lib::List;

#[cfg(test)]
mod tests;

/// How the source type compares with the target, level by level.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Comparison {
    Equal,
    Widen { integer: bool },
    Wrap,
    Retype,
}

pub(crate) fn payload(matching: &List<Option<DeclIx>>, old: Option<DeclIx>, new: Option<DeclIx>) -> bool {
    match old {
        Some(old) => match new {
            Some(new) => *matching.get(new.index()).expect("new declaration index") == Some(old),
            None => false,
        },
        None => new.is_none(),
    }
}
pub(crate) fn held(matching: &List<Option<DeclIx>>, old: &Schema, new: &Schema, a: Held, b: Held) -> Comparison {
    match a {
        Held::Opaque => match b {
            Held::Opaque => Comparison::Equal,
            Held::Record { .. } | Held::Pieces(_) => Comparison::Widen { integer: false },
        },
        Held::Record { family: af, name: an } => match b {
            Held::Opaque => Comparison::Widen { integer: false },
            Held::Pieces(_) => Comparison::Retype,
            Held::Record { family: bf, name: bn } => {
                if old.spelled(af) == new.spelled(bf) && old.spelled(an) == new.spelled(bn) {
                    Comparison::Equal
                } else {
                    Comparison::Retype
                }
            }
        },
        Held::Pieces(a) => match b {
            Held::Opaque => Comparison::Widen { integer: false },
            Held::Record { .. } => Comparison::Retype,
            Held::Pieces(b) => {
                if payload(matching, Some(a), Some(b)) {
                    Comparison::Equal
                } else {
                    Comparison::Retype
                }
            }
        },
    }
}
pub(crate) fn width(ty: Type) -> Option<u32> {
    match ty {
        Type::U8 => Some(1),
        Type::U16 => Some(2),
        Type::U32 => Some(4),
        Type::U64 => Some(8),
        Type::Bool
        | Type::Duration
        | Type::Fixed(_)
        | Type::Secret { .. }
        | Type::Text(_)
        | Type::Bytes { .. }
        | Type::List { .. }
        | Type::Option(_)
        | Type::Named(_) => None,
    }
}
#[expect(
    clippy::too_many_lines,
    reason = "the leaf comparison exhaustively lists schema type pairs (data/codec/diff.md, 3)"
)]
pub(crate) fn leaf(matching: &List<Option<DeclIx>>, old: &Schema, new: &Schema, a: Type, b: Type) -> Comparison {
    let equal = match a {
        Type::Named(_) | Type::Bytes { .. } | Type::Secret { .. } | Type::List { .. } | Type::Option(_) => false,
        Type::U8 | Type::U16 | Type::U32 | Type::U64 | Type::Bool | Type::Duration | Type::Fixed(_) | Type::Text(_) => {
            a == b
        }
    };
    if equal {
        return Comparison::Equal;
    }
    if let Some(a) = width(a)
        && let Some(b) = width(b)
    {
        return if b > a { Comparison::Widen { integer: true } } else { Comparison::Retype };
    }
    match a {
        Type::Named(a) => match b {
            Type::Named(b) => {
                if payload(matching, Some(a), Some(b)) {
                    Comparison::Equal
                } else {
                    Comparison::Retype
                }
            }
            Type::U8
            | Type::U16
            | Type::U32
            | Type::U64
            | Type::Bool
            | Type::Duration
            | Type::Fixed(_)
            | Type::Secret { .. }
            | Type::Text(_)
            | Type::Bytes { .. }
            | Type::List { .. }
            | Type::Option(_) => Comparison::Retype,
        },
        Type::Text(a) => match b {
            Type::Text(b) | Type::Secret { bound: b, .. } | Type::Bytes { bound: b, .. } => {
                if b >= a {
                    Comparison::Widen { integer: false }
                } else {
                    Comparison::Retype
                }
            }
            Type::U8
            | Type::U16
            | Type::U32
            | Type::U64
            | Type::Bool
            | Type::Duration
            | Type::Fixed(_)
            | Type::List { .. }
            | Type::Option(_)
            | Type::Named(_) => Comparison::Retype,
        },
        Type::Secret { bound: a, held: ah } => match b {
            Type::Bytes { bound: b, .. } => {
                if b >= a {
                    Comparison::Widen { integer: false }
                } else {
                    Comparison::Retype
                }
            }
            Type::Secret { bound: b, held: bh } => {
                if b < a {
                    return Comparison::Retype;
                }
                match held(matching, old, new, ah, bh) {
                    Comparison::Equal => {
                        if b > a {
                            Comparison::Widen { integer: false }
                        } else {
                            Comparison::Equal
                        }
                    }
                    other @ (Comparison::Widen { .. } | Comparison::Wrap | Comparison::Retype) => other,
                }
            }
            Type::U8
            | Type::U16
            | Type::U32
            | Type::U64
            | Type::Bool
            | Type::Duration
            | Type::Fixed(_)
            | Type::Text(_)
            | Type::List { .. }
            | Type::Option(_)
            | Type::Named(_) => Comparison::Retype,
        },
        Type::Bytes { bound: a, held: ah } => match b {
            Type::Secret { bound: b, .. } => {
                if b >= a {
                    Comparison::Widen { integer: false }
                } else {
                    Comparison::Retype
                }
            }
            Type::Bytes { bound: b, held: bh } => {
                if b < a {
                    return Comparison::Retype;
                }
                match held(matching, old, new, ah, bh) {
                    Comparison::Equal => {
                        if b > a {
                            Comparison::Widen { integer: false }
                        } else {
                            Comparison::Equal
                        }
                    }
                    other @ (Comparison::Widen { .. } | Comparison::Wrap | Comparison::Retype) => other,
                }
            }
            Type::U8
            | Type::U16
            | Type::U32
            | Type::U64
            | Type::Bool
            | Type::Duration
            | Type::Fixed(_)
            | Type::Text(_)
            | Type::List { .. }
            | Type::Option(_)
            | Type::Named(_) => Comparison::Retype,
        },
        Type::U8
        | Type::U16
        | Type::U32
        | Type::U64
        | Type::Bool
        | Type::Duration
        | Type::Fixed(_)
        | Type::List { .. }
        | Type::Option(_) => Comparison::Retype,
    }
}

/// Compare types iteratively using new-to-old declaration matches.
pub(crate) fn compare(
    matching: &List<Option<DeclIx>>,
    old: &Schema,
    new: &Schema,
    mut a: TypeIx,
    mut b: TypeIx,
) -> Comparison {
    let mut widening = false;
    let mut integer = false;
    let mut wrapped = false;
    let turns = add(
        u32::try_from(old.text().len()).expect("schema ceiling"),
        u32::try_from(new.text().len()).expect("schema ceiling"),
    );
    for _ in 0..turns {
        let av = old.ty(a);
        let bv = new.ty(b);
        match bv {
            Type::Option(inner) => {
                b = inner;
                match av {
                    Type::Option(inner) => a = inner,
                    Type::U8
                    | Type::U16
                    | Type::U32
                    | Type::U64
                    | Type::Bool
                    | Type::Duration
                    | Type::Fixed(_)
                    | Type::Secret { .. }
                    | Type::Text(_)
                    | Type::Bytes { .. }
                    | Type::List { .. }
                    | Type::Named(_) => wrapped = true,
                }
                continue;
            }
            Type::List { bound: bb, item: bi } => match av {
                Type::List { bound: ab, item: ai } => {
                    if bb < ab {
                        return Comparison::Retype;
                    }
                    widening |= bb > ab;
                    a = ai;
                    b = bi;
                    continue;
                }
                Type::U8
                | Type::U16
                | Type::U32
                | Type::U64
                | Type::Bool
                | Type::Duration
                | Type::Fixed(_)
                | Type::Secret { .. }
                | Type::Text(_)
                | Type::Bytes { .. }
                | Type::Option(_)
                | Type::Named(_) => {}
            },
            Type::U8
            | Type::U16
            | Type::U32
            | Type::U64
            | Type::Bool
            | Type::Duration
            | Type::Fixed(_)
            | Type::Secret { .. }
            | Type::Text(_)
            | Type::Bytes { .. }
            | Type::Named(_) => {}
        }
        match leaf(matching, old, new, av, bv) {
            Comparison::Retype => return Comparison::Retype,
            Comparison::Equal => {}
            Comparison::Widen { integer: wider } => {
                widening = true;
                integer |= wider;
            }
            Comparison::Wrap => unreachable!("a leaf does not wrap"),
        }
        return if wrapped {
            Comparison::Wrap
        } else if widening {
            Comparison::Widen { integer }
        } else {
            Comparison::Equal
        };
    }
    unreachable!("each wrapper advances within its schema's bounded types")
}

//! Colours, placement and checked glyphs (skein/docs/design/ui-screen.md, section 3).

use alloc::boxed::Box;
use skein_ui::{Region, Style};

use crate::width;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Rgb {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[expect(clippy::struct_excessive_bools, reason = "the five paint attributes are independent and compose by OR")]
pub struct Paint {
    pub fore: Option<Rgb>,
    pub back: Option<Rgb>,
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: bool,
    pub reverse: bool,
}

impl Paint {
    pub const NONE: Paint =
        Paint { fore: None, back: None, bold: false, dim: false, italic: false, underline: false, reverse: false };

    #[must_use]
    pub fn over(self, under: Paint) -> Paint {
        Paint {
            fore: self.fore.or(under.fore),
            back: self.back.or(under.back),
            bold: self.bold || under.bold,
            dim: self.dim || under.dim,
            italic: self.italic || under.italic,
            underline: self.underline || under.underline,
            reverse: self.reverse || under.reverse,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Place {
    Top,
    Body,
    Bottom,
    Side { percent: u8 },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Arrange {
    Lines,
    Spread,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Border {
    /// Top left, top right, bottom left, bottom right.
    pub corners: [char; 4],
    pub across: char,
    pub down: char,
    pub paint: Paint,
    pub title: bool,
    pub sides: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct RegionLook {
    pub region: Region,
    pub place: Place,
    pub arrange: Arrange,
    pub border: Option<Border>,
    pub pad: u16,
    pub paint: Paint,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Align {
    Start,
    End,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Mark {
    pub glyph: char,
    pub paint: Paint,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct StyleLook {
    pub style: Style,
    pub paint: Paint,
    pub align: Align,
    pub band: bool,
    pub label: bool,
    pub mark: Option<Mark>,
    pub described: Option<Paint>,
    /// Replaces an active options item's name paint over its region, beneath active and focus paint.
    /// None keeps the item's composed paint; descriptions, marks and bands are unchanged.
    pub active_name: Option<Paint>,
    /// On a focused text field or text area carrying this token: the paint
    /// laid over the node's own paint on its value and its caret's cell, in
    /// place of the focus's paint (ui-screen.md, 3 and 5.4).
    pub focused_value: Option<Paint>,
    pub spaced: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Glyphs {
    pub focus: char,
    pub mask: char,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Theme {
    pub base: Paint,
    pub focus: Paint,
    pub placeholder: Paint,
    /// A band across an options menu's active item, beneath the focus paint.
    pub active: Paint,
    pub regions: Box<[RegionLook]>,
    pub styles: Box<[StyleLook]>,
    pub glyphs: Glyphs,
}

impl Theme {
    #[must_use]
    pub fn valid(&self) -> bool {
        if !valid_glyph(self.glyphs.focus) || !valid_glyph(self.glyphs.mask) {
            return false;
        }
        for (at, look) in self.regions.iter().enumerate() {
            for earlier in self.regions.iter().take(at) {
                if earlier.region == look.region {
                    return false;
                }
            }
            match look.place {
                Place::Side { percent } => {
                    if !(1..100).contains(&percent) || look.arrange == Arrange::Spread {
                        return false;
                    }
                    if let Some(border) = look.border
                        && (border.sides || border.title)
                    {
                        return false;
                    }
                }
                Place::Top | Place::Body | Place::Bottom => {}
            }
            if let Some(border) = look.border {
                for glyph in border.corners.into_iter().chain([border.across, border.down]) {
                    if !valid_glyph(glyph) {
                        return false;
                    }
                }
            }
        }
        for (at, look) in self.styles.iter().enumerate() {
            if let Some(mark) = look.mark
                && !valid_glyph(mark.glyph)
            {
                return false;
            }
            for earlier in self.styles.iter().take(at) {
                if earlier.style == look.style {
                    return false;
                }
            }
        }
        true
    }
}

fn valid_glyph(glyph: char) -> bool {
    !glyph.is_control() && width(glyph) == 1
}

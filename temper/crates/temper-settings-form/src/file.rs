//! Bounded whole settings input (client/ctl/results.md, 4; jig's client/domain/contract/requests.md, 2).
mod formats;

use crate::{
    Limits,
    fields::formats::{FORMATS, Formats},
};
use jig_client_protocol_wire::Record as _;
#[path = "settings_file_values.rs"]
pub(crate) mod values;
use crate::view as c;
use alloc::boxed::Box;
use skein_json::{Document, Kind};
use skein_lib::{List, Writer};
pub(super) type Fields = List<(Box<[u8]>, u32)>;
type Notices = List<Box<[u8]>>;

/// A readable repository a host call's charter document may name by OWNER/NAME
/// (temper/docs/design/domain/agent/ask-temper.md, 2).
#[derive(Debug)]
pub struct Repository<'a> {
    pub host: &'a [u8],
    pub owner: &'a [u8],
    pub name: &'a [u8],
}

pub(super) struct Error {
    member: Box<[u8]>,
    rule: Box<[u8]>,
    bound: Option<u64>,
}
impl Error {
    pub(super) fn named(mut self, member: &[u8]) -> Self {
        self.member = member.into();
        self
    }
    pub(super) fn at(mut self, path: &[u8]) -> Self {
        let nested = !self.member.is_empty() && self.member.as_ref() != b"JSON";
        let length = if nested {
            path.len().checked_add(self.member.len()).expect("bounded path").checked_add(1).expect("separator")
        } else {
            path.len()
        };
        let mut member = Writer::new(length);
        member.put(path).expect("path room");
        if nested {
            member.put(b".").expect("separator room");
            member.put(&self.member).expect("member room");
        }
        self.member = member.finish();
        self
    }
}
pub(super) fn error(member: &[u8], rule: &'static [u8]) -> Error {
    Error { member: member.into(), rule: rule.into(), bound: None }
}
fn bound(member: &[u8], maximum: u32) -> Error {
    Error { member: member.into(), rule: b"bound".as_slice().into(), bound: Some(u64::from(maximum)) }
}
fn message(item: Option<u32>, why: Error) -> Box<[u8]> {
    let mut measured = crate::Text::measure();
    describe(item, &why, &mut measured);
    let mut out = crate::Text::writing_exact(measured);
    describe(item, &why, &mut out);
    out.finish()
}
fn describe(item: Option<u32>, why: &Error, out: &mut crate::Text) {
    out.put(b"settings file");
    if let Some(item) = item {
        out.put(b" item ");
        out.number(u64::from(item));
    }
    out.put(b": ");
    out.put(&why.member);
    out.put(b": ");
    out.put(&why.rule);
    if let Some(bound) = why.bound {
        out.put(b" ");
        out.number(bound);
    }
}
pub(super) struct Decoder<'a> {
    pub repositories: Option<&'a [Repository<'a>]>,
    pub doc: Document,
    pub limits: jig_client_protocol_wire::Limits,
    pub kinds: temper_client_protocol_wire::v1::Limits,
}
impl Decoder<'_> {
    pub fn kind(&self, at: u32) -> Result<Kind, Error> {
        Ok(self.doc.token(at).ok_or(error(b"JSON", b"missing value"))?.kind)
    }
    pub fn string(&self, at: u32) -> Result<&[u8], Error> {
        let token = self.doc.token(at).ok_or(error(b"JSON", b"missing string"))?;
        if token.kind != Kind::String && token.kind != Kind::Key {
            return Err(error(b"JSON", b"expected string"));
        }
        self.doc.text(token).ok_or(error(b"JSON", b"invalid string"))
    }
    pub fn number(&self, at: u32) -> Result<u64, Error> {
        if self.kind(at)? != Kind::Number {
            return Err(error(b"JSON", b"expected unsigned integer"));
        }
        let bytes = self
            .doc
            .text(self.doc.token(at).ok_or(error(b"JSON", b"missing number"))?)
            .ok_or(error(b"JSON", b"invalid number"))?;
        let mut number = 0_u64;
        if bytes.is_empty() {
            return Err(error(b"JSON", b"expected unsigned integer"));
        }
        for byte in bytes {
            if !byte.is_ascii_digit() {
                return Err(error(b"JSON", b"expected unsigned integer"));
            }
            number = number.checked_mul(10).ok_or(error(b"JSON", b"integer bound"))?;
            number = number
                .checked_add(u64::from(byte.checked_sub(b'0').expect("ASCII digit")))
                .ok_or(error(b"JSON", b"integer bound"))?;
        }
        Ok(number)
    }
    pub fn boolean(&self, at: u32) -> Result<bool, Error> {
        match self.kind(at)? {
            Kind::True => Ok(true),
            Kind::False => Ok(false),
            Kind::ObjectStart
            | Kind::ObjectEnd
            | Kind::ArrayStart
            | Kind::ArrayEnd
            | Kind::Key
            | Kind::String
            | Kind::Number
            | Kind::Null
            | Kind::Long => Err(error(b"JSON", b"expected boolean")),
        }
    }
    pub fn words(&self, at: u32, most: u32, name: &[u8]) -> Result<c::Text, Error> {
        let bytes = self.string(at)?;
        if bytes.len() > usize::try_from(most).expect("u32 fits") {
            return Err(bound(name, most));
        }
        c::Text::new(bytes.into(), most).or(Err(error(name, b"UTF-8 required")))
    }
    pub fn document_text(&self, at: u32) -> Result<c::DocumentText, Error> {
        let fields = self.object(at, &[b"bytes", b"cut"])?;
        if self.boolean(field(&fields, b"cut")?)? {
            return Err(error(b"cut", b"whole value required"));
        }
        Ok(c::DocumentText::complete(self.words(
            field(&fields, b"bytes")?,
            self.limits.document_text_bytes,
            b"document_text_bytes",
        )?))
    }
    pub fn hex(&self, at: u32, most: u32, name: &[u8]) -> Result<Box<[u8]>, Error> {
        let bytes = self.string(at)?;
        if bytes.len() % 2 != 0 {
            return Err(error(name, b"even hexadecimal bytes required"));
        }
        let count = bytes.len() / 2;
        if count > usize::try_from(most).expect("u32 fits") {
            return Err(bound(name, most));
        }
        let mut out = Writer::new(count);
        for &[high, low] in bytes.as_chunks::<2>().0 {
            let high = digit(high)?;
            let low = digit(low)?;
            out.put(&[high.checked_mul(16).expect("hex digit").checked_add(low).expect("hex byte")])
                .expect("measured hex");
        }
        Ok(out.finish())
    }
    pub fn object(&self, start: u32, allowed: &[&[u8]]) -> Result<Fields, Error> {
        if self.kind(start)? != Kind::ObjectStart {
            return Err(error(b"JSON", b"expected object"));
        }
        let mut fields = List::with_capacity(u32::try_from(allowed.len()).expect("schema field count"));
        let mut depth = 0_u32;
        for at in start.checked_add(1).ok_or(error(b"JSON", b"position bound"))?..self.doc.len() {
            match self.kind(at)? {
                Kind::Key if depth == 0 => {
                    let key = self.string(at)?;
                    if !allowed.contains(&key) {
                        return Err(error(key, b"unknown member"));
                    }
                    if optional(&fields, key).is_some() {
                        return Err(error(key, b"duplicate member"));
                    }
                    fields
                        .push((key.into(), at.checked_add(1).ok_or(error(key, b"position bound"))?))
                        .or(Err(error(key, b"field count bound")))?;
                }
                Kind::ObjectStart | Kind::ArrayStart => depth = depth.checked_add(1).expect("bounded JSON depth"),
                Kind::ObjectEnd | Kind::ArrayEnd => {
                    if depth == 0 {
                        return Ok(fields);
                    }
                    depth = depth.checked_sub(1).expect("positive depth");
                }
                Kind::Key | Kind::String | Kind::Number | Kind::True | Kind::False | Kind::Null | Kind::Long => {}
            }
        }
        Err(error(b"JSON", b"unfinished object"))
    }
    /// A bounded map whose keys are the value's names, not schema members.
    pub fn named(&self, start: u32, most: u32) -> Result<Fields, Error> {
        if self.kind(start)? != Kind::ObjectStart {
            return Err(error(b"JSON", b"expected object"));
        }
        let mut fields = List::with_capacity(most);
        let mut depth = 0_u32;
        for at in start.checked_add(1).ok_or(error(b"JSON", b"position bound"))?..self.doc.len() {
            match self.kind(at)? {
                Kind::Key if depth == 0 => {
                    let key = self.string(at)?;
                    if optional(&fields, key).is_some() {
                        return Err(error(key, b"duplicate member"));
                    }
                    fields
                        .push((key.into(), at.checked_add(1).ok_or(error(key, b"position bound"))?))
                        .or(Err(bound(key, most)))?;
                }
                Kind::ObjectStart | Kind::ArrayStart => depth = depth.checked_add(1).expect("bounded JSON depth"),
                Kind::ObjectEnd | Kind::ArrayEnd => {
                    if depth == 0 {
                        return Ok(fields);
                    }
                    depth = depth.checked_sub(1).expect("positive depth");
                }
                Kind::Key | Kind::String | Kind::Number | Kind::True | Kind::False | Kind::Null | Kind::Long => {}
            }
        }
        Err(error(b"JSON", b"unfinished object"))
    }
    pub fn array(&self, start: u32, most: u32, name: &[u8]) -> Result<List<u32>, Error> {
        if self.kind(start)? != Kind::ArrayStart {
            return Err(error(name, b"expected array"));
        }
        // Count before allocating; every element costs at least one token.
        let mut count = 0_u32;
        let mut depth = 0_u32;
        for at in start.checked_add(1).ok_or(error(name, b"position bound"))?..self.doc.len() {
            let kind = self.kind(at)?;
            if depth == 0 && kind == Kind::ArrayEnd {
                break;
            }
            if depth == 0 {
                count = count.checked_add(1).ok_or(bound(name, most))?;
                if count > most {
                    return Err(bound(name, most));
                }
            }
            match kind {
                Kind::ArrayStart | Kind::ObjectStart => depth = depth.checked_add(1).expect("bounded JSON depth"),
                Kind::ArrayEnd | Kind::ObjectEnd => depth = depth.checked_sub(1).expect("balanced JSON"),
                Kind::Key | Kind::String | Kind::Number | Kind::True | Kind::False | Kind::Null | Kind::Long => {}
            }
        }
        let mut items = List::with_capacity(count);
        depth = 0;
        for at in start.checked_add(1).ok_or(error(name, b"position bound"))?..self.doc.len() {
            let kind = self.kind(at)?;
            if depth == 0 && kind == Kind::ArrayEnd {
                break;
            }
            if depth == 0 {
                items.push(at).expect("counted elements");
            }
            match kind {
                Kind::ArrayStart | Kind::ObjectStart => depth = depth.checked_add(1).expect("bounded JSON depth"),
                Kind::ArrayEnd | Kind::ObjectEnd => depth = depth.checked_sub(1).expect("balanced JSON"),
                Kind::Key | Kind::String | Kind::Number | Kind::True | Kind::False | Kind::Null | Kind::Long => {}
            }
        }
        Ok(items)
    }
}
fn digit(byte: u8) -> Result<u8, Error> {
    match byte {
        b'0'..=b'9' => Ok(byte.checked_sub(b'0').expect("ASCII digit")),
        b'a'..=b'f' => Ok(byte.checked_sub(b'a').expect("hex letter").checked_add(10).expect("hex digit")),
        _ => Err(error(b"JSON", b"lower-case hexadecimal required")),
    }
}
pub(super) fn optional(fields: &Fields, name: &[u8]) -> Option<u32> {
    for (key, at) in fields {
        if key.as_ref() == name {
            return Some(*at);
        }
    }
    None
}
pub(super) fn field(fields: &Fields, name: &[u8]) -> Result<u32, Error> {
    optional(fields, name).ok_or(error(name, b"missing member"))
}

fn setting(d: &Decoder<'_>, at: u32) -> Result<c::Setting, Error> {
    let fields = d.object(at, &[b"kind", b"number", b"project"])?;
    let kind = d.string(field(&fields, b"kind")?)?;
    if [
        b"landing-rule".as_slice(),
        b"forge-charter",
        b"template-goal",
        b"follow",
        b"dependency",
        b"project-ways",
        b"project-questions",
        b"project-classes",
        b"charter-words",
        b"project-words",
        b"project-landing",
        b"project-gates",
        b"project-areas",
        b"project-triager",
        b"agents",
        b"welcome",
        b"temper-limits",
    ]
    .contains(&kind)
    {
        return crate::kinds::setting(d, &fields, kind);
    }
    if kind == b"project-policy" {
        if optional(&fields, b"number").is_some() {
            return Err(error(b"number", b"project-policy has no number"));
        }
        let project = u32::try_from(d.number(field(&fields, b"project")?)?).or(Err(error(b"project", b"u32 bound")))?;
        if project == 0 {
            return Err(error(b"project", b"from 1"));
        }
        return Ok(c::Setting::ProjectPolicy { project });
    }
    if optional(&fields, b"project").is_some() {
        return Err(error(b"project", b"unknown member"));
    }
    if kind == b"limits" {
        if fields.len() != 1 {
            return Err(error(b"number", b"limits has no number"));
        }
        return Ok(c::Setting::Limits);
    }
    if kind == b"rules" {
        if fields.len() != 1 {
            return Err(error(b"number", b"rules has no number"));
        }
        return Ok(c::Setting::Rules);
    }
    let number = c::SettingNumber {
        number: u32::try_from(d.number(field(&fields, b"number")?)?).or(Err(error(b"number", b"u32 bound")))?,
    };
    match kind {
        b"charter" => Ok(c::Setting::Charter(number)),
        b"template" => Ok(c::Setting::Template(number)),
        b"model" => Ok(c::Setting::Model(number)),
        b"worker" => Ok(c::Setting::Worker(number)),
        b"provider" => Ok(c::Setting::Provider(number)),
        b"policy" => Ok(c::Setting::Policy(number)),
        _ => Err(error(b"kind", b"unknown setting kind")),
    }
}
fn item(d: &Decoder<'_>, at: u32) -> Result<c::SettingItem, Error> {
    let fields = d.object(at, &[b"setting", b"set", b"edit", b"overlay", b"remove", b"reset"])?;
    let setting = setting(d, field(&fields, b"setting")?)?;
    if optional(&fields, b"remove").is_some() || optional(&fields, b"reset").is_some() {
        for name in [b"set".as_slice(), b"edit", b"overlay"] {
            if optional(&fields, name).is_some() {
                return Err(error(b"item", b"exactly one edit required"));
            }
        }
        let edit = match (optional(&fields, b"remove"), optional(&fields, b"reset")) {
            (Some(at), None) if d.boolean(at)? => c::SettingEdit::Removed,
            (None, Some(at)) if d.boolean(at)? => c::SettingEdit::Reset,
            (Some(_) | None, Some(_) | None) => return Err(error(b"item", b"one true remove or reset required")),
        };
        if let c::Setting::ProjectPolicy { .. } = setting {
            return Err(error(b"edit", b"a project's policy is set alone"));
        }
        return Ok(c::SettingItem { setting, edit });
    }
    if let Some(at) = optional(&fields, b"overlay") {
        if optional(&fields, b"set").is_some() || optional(&fields, b"edit").is_some() {
            return Err(error(b"item", b"exactly one of set, edit or overlay required"));
        }
        let c::Setting::Charter(n) = setting else {
            return Err(error(b"overlay", b"charter required"));
        };
        let overlay = values::overlay(d, at, n.number)?;
        return Ok(c::SettingItem {
            setting,
            edit: c::SettingEdit::Set(c::SettingSet { value: c::SettingValue::Overlay(Box::new(overlay)) }),
        });
    }
    let edit = match (optional(&fields, b"set"), optional(&fields, b"edit")) {
        (Some(at), None) => {
            let value = match setting {
                c::Setting::Provider(number) => {
                    c::SettingValue::Provider(Box::new(crate::providers::read(d, at, number.number)?))
                }
                c::Setting::Worker(number) => c::SettingValue::Worker(Box::new(values::worker(d, at, number.number)?)),
                c::Setting::Model(_) => c::SettingValue::Model(Box::new(values::catalogue(d, at)?)),
                c::Setting::Charter(_) => c::SettingValue::Charter(Box::new(values::charter(d, at)?)),
                c::Setting::Template(_) => c::SettingValue::Template(Box::new(values::template(d, at)?)),
                c::Setting::Policy(_) | c::Setting::ProjectPolicy { .. } => {
                    c::SettingValue::Policy(Box::new(values::setting_policy(d, at)?))
                }
                c::Setting::Application { kind, .. } => c::SettingValue::Application(crate::kinds::value(d, at, kind)?),
                c::Setting::Limits => c::SettingValue::Limits(crate::kinds::limits::read(d, at)?.into_boxed()),
                c::Setting::Rules => c::SettingValue::Rules(Box::new(values::rules(d, at)?)),
            };
            c::SettingEdit::Set(c::SettingSet { value })
        }
        (None, Some(at)) => {
            if let c::Setting::ProjectPolicy { .. } = setting {
                return Err(error(b"edit", b"a project's policy is set alone"));
            }
            match d.string(at)? {
                b"removed" => c::SettingEdit::Removed,
                b"reset" => c::SettingEdit::Reset,
                _ => return Err(error(b"edit", b"removed or reset required")),
            }
        }
        _ => return Err(error(b"item", b"exactly one of set or edit required")),
    };
    Ok(c::SettingItem { setting, edit })
}
/// Decode the items in a settings file using the same bounds as ctl.
pub fn decode(bytes: &[u8], limits: &Limits) -> Result<List<c::SettingItem>, Box<[u8]>> {
    Ok(read(bytes, limits)?.items)
}

/// A person's file, moved to the newest form, with the input's format and notices.
#[derive(Debug)]
pub struct Read {
    pub format: u64,
    pub notices: Notices,
    pub items: List<c::SettingItem>,
}

pub fn read(bytes: &[u8], limits: &Limits) -> Result<Read, Box<[u8]>> {
    read_with(bytes, limits, FORMATS)
}

pub(crate) fn read_with(bytes: &[u8], limits: &Limits, formats: Formats) -> Result<Read, Box<[u8]>> {
    let mut d = decoder(bytes, limits, None)?;
    let format = formats::read(&d, formats)?;
    let (doc, notices) = formats::migrate(d.doc, limits, format, formats)?;
    d.doc = doc;
    let items = decode_root(&d, limits, false)?;
    Ok(Read { format, notices, items })
}

/// Render a file using the settings document's value spellings, without assembling overlays.
pub fn render(items: &[c::SettingItem], limits: &Limits) -> Result<Box<[u8]>, Box<[u8]>> {
    render_with(items, limits, FORMATS)
}

pub(crate) fn render_with(items: &[c::SettingItem], limits: &Limits, formats: Formats) -> Result<Box<[u8]>, Box<[u8]>> {
    use skein_json::writer::{Encoder, Limits as JsonLimits};
    let most = u32::try_from(limits.result_bytes).unwrap_or(u32::MAX);
    let bounds = JsonLimits { depth: 32, length: most };
    let mut measure = Encoder::measure(&bounds);
    encode(items, formats.newest, &mut measure);
    let size = measure.measured().or(Err(message(None, bound(b"rendered file bytes", most))))?;
    let mut out = Encoder::write(size, &bounds);
    encode(items, formats.newest, &mut out);
    let bytes = out.finish();
    let mut line = crate::Text::measure();
    line.put(&bytes);
    line.put(b"\n");
    let mut line = crate::Text::writing_exact(line);
    line.put(&bytes);
    line.put(b"\n");
    Ok(line.finish())
}

fn encode(items: &[c::SettingItem], newest: u64, out: &mut skein_json::writer::Encoder) {
    out.object_start();
    out.key(b"items");
    out.array_start();
    for item in items {
        out.object_start();
        out.key(b"setting");
        crate::render::encode_setting(item.setting, out);
        match &item.edit {
            c::SettingEdit::Set(set) => {
                if let c::SettingValue::Overlay(overlay) = &set.value {
                    out.key(b"overlay");
                    crate::values::encode_file_overlay(overlay, out);
                } else {
                    out.key(b"set");
                    crate::render::encode_named_value(item.setting, &set.value, out);
                }
            }
            c::SettingEdit::Removed => {
                out.key(b"edit");
                out.string(b"removed");
            }
            c::SettingEdit::Reset => {
                out.key(b"edit");
                out.string(b"reset");
            }
        }
        out.object_end();
    }
    out.array_end();
    if newest > 1 {
        out.key(b"format");
        out.unsigned(newest);
    }
    out.object_end();
}
/// Decode a host read's setting through the settings file's kind decoder.
pub fn read_setting(bytes: &[u8], limits: &Limits) -> Result<c::Setting, Box<[u8]>> {
    if skein_lib::len_u64(bytes.len()) > limits.ask_bytes {
        return Err(message(None, bound(b"file bytes", u32::try_from(limits.ask_bytes).unwrap_or(u32::MAX))));
    }
    let doc = crate::json::document_depth(bytes, 32)
        .or(Err(message(None, error(b"JSON", b"invalid JSON or depth bound 32"))))?;
    let d = Decoder { doc, limits: limits.settings, kinds: limits.kinds, repositories: None };
    let fields = d.object(0, &[b"setting", b"revision", b"history", b"from"]);
    let fields = match fields {
        Ok(fields) => fields,
        Err(why) => return Err(message(None, why)),
    };
    let at = match field(&fields, b"setting") {
        Ok(at) => at,
        Err(why) => return Err(message(None, why)),
    };
    match setting(&d, at) {
        Ok(setting) => Ok(setting),
        Err(why) => Err(message(None, why)),
    }
}
/// Decode a host proposal's items in the settings file form.
pub fn proposal_items(bytes: &[u8], limits: &Limits) -> Result<List<c::SettingItem>, Box<[u8]>> {
    proposal_items_in(bytes, limits, None)
}
/// Resolve an omitted forge among the project's repositories; `None` requires it,
/// as a settings file does (temper/docs/design/domain/agent/ask-temper.md, 2).
pub fn proposal_items_in(
    bytes: &[u8],
    limits: &Limits,
    repositories: Option<&[Repository<'_>]>,
) -> Result<List<c::SettingItem>, Box<[u8]>> {
    decode_root(&decoder(bytes, limits, repositories)?, limits, true)
}

fn decoder<'a>(
    bytes: &[u8],
    limits: &Limits,
    repositories: Option<&'a [Repository<'a>]>,
) -> Result<Decoder<'a>, Box<[u8]>> {
    if skein_lib::len_u64(bytes.len()) > limits.ask_bytes {
        return Err(message(None, bound(b"file bytes", u32::try_from(limits.ask_bytes).unwrap_or(u32::MAX))));
    }
    let doc = crate::json::document_depth(bytes, 32)
        .or(Err(message(None, error(b"JSON", b"invalid JSON or depth bound 32"))))?;
    Ok(Decoder { doc, limits: limits.settings, kinds: limits.kinds, repositories })
}

fn decode_root(d: &Decoder<'_>, limits: &Limits, proposal: bool) -> Result<List<c::SettingItem>, Box<[u8]>> {
    let allowed: &[&[u8]] = if proposal { &[b"items", b"against", b"reason", b"more"] } else { &[b"items", b"format"] };
    let fields = match d.object(0, allowed) {
        Ok(fields) => fields,
        Err(why) => return Err(message(None, why)),
    };
    // The wire carries length-prefixed items in an ask-sized blob. Even an
    // empty item occupies bytes; the exact aggregate is checked below too.
    let at = match field(&fields, b"items") {
        Ok(at) => at,
        Err(why) => return Err(message(None, why)),
    };
    let positions = match d.array(at, 1024, b"items") {
        Ok(positions) => positions,
        Err(why) => return Err(message(None, why)),
    };
    let mut items: List<c::SettingItem> = List::with_capacity(positions.len());
    let mut encoded = 0_u32;
    for (index, at) in positions.iter().enumerate() {
        let index = u32::try_from(index).expect("bounded items").checked_add(1).expect("item count fits");
        let next = match item(d, *at) {
            Ok(item) => item,
            Err(why) => return Err(message(Some(index), why)),
        };
        for previous in &items {
            if let (c::Setting::ProjectPolicy { .. }, c::Setting::ProjectPolicy { .. }) =
                (previous.setting, next.setting)
            {
                return Err(message(Some(index), error(b"setting", b"one project's policy per file")));
            }
            if previous.setting == next.setting {
                return Err(message(Some(index), error(b"setting", b"duplicate setting")));
            }
        }
        let row = match crate::wire::settings_wire_setting_item(&next, &limits.settings) {
            Ok(row) => row,
            Err(problem) => return Err(message(Some(index), wire_error(problem, &limits.settings))),
        };
        let measured = match row.measure(&limits.settings) {
            Ok(size) => size,
            Err(problem) => return Err(message(Some(index), wire_error(problem, &limits.settings))),
        };
        encoded = encoded
            .checked_add(measured)
            .ok_or(message(Some(index), bound(b"encoded items bytes", limits.settings.change_settings_items)))?;
        encoded = encoded
            .checked_add(4)
            .ok_or(message(Some(index), bound(b"encoded items bytes", limits.settings.change_settings_items)))?;
        if encoded > limits.settings.change_settings_items {
            return Err(message(Some(index), bound(b"encoded items bytes", limits.settings.change_settings_items)));
        }
        items.push(next).expect("counted items");
    }
    if proposal {
        let directories = crate::overlay::directories(&items);
        if let Some(name) = directories.get(0) {
            return Err(crate::overlay::refusal(name, "directory: ctl file input required"));
        }
    }
    let ask = c::ChangeSettings { against: 0, items, running: c::Running::Stays };
    let wire = crate::wire::wire_change_settings(&ask, &limits.settings)
        .or(Err(message(None, bound(b"encoded items bytes", limits.settings.change_settings_items))))?;
    let wire = jig_client_protocol_wire::Ask::ChangeSettings(wire);
    let measured = match wire.measure(&limits.settings) {
        Ok(size) => size,
        Err(problem) => return Err(message(None, wire_error(problem, &limits.settings))),
    };
    if u64::from(measured) > limits.ask_bytes {
        return Err(message(
            None,
            bound(b"encoded request bytes", u32::try_from(limits.ask_bytes).unwrap_or(u32::MAX)),
        ));
    }
    Ok(ask.items)
}

fn wire_error(problem: jig_client_protocol_wire::Problem, limits: &jig_client_protocol_wire::Limits) -> Error {
    use jig_client_protocol_wire::paths;
    match problem.path {
        path if path == paths::CHARTER_ALTERNATIVES => bound(b"charter_alternatives", limits.charter_alternatives),
        path if path == paths::CHARTER_INSTRUCTIONS => bound(b"charter_instructions", limits.charter_instructions),
        path if path == paths::FIELD_RULE_NAME => bound(b"field_rule_name", limits.field_rule_name),
        path if path == paths::LLM_CHOICE_SUBAGENTS => bound(b"llm_choice_subagents", limits.llm_choice_subagents),
        path if path == paths::MODEL_EFFORT_EFFORT => {
            bound(b"model_effort_effort_some", limits.model_effort_effort_some)
        }
        path if path == paths::MODEL_EFFORT_MODEL => bound(b"model_effort_model", limits.model_effort_model),
        path if path == paths::MODEL_SET_NAMES_NAMES => bound(b"model_set_names_names", limits.model_set_names_names),
        path if path == paths::OWNED_RECORD => bound(b"owned_record", limits.owned_record),
        path if path == paths::AUTHORITY_GRANTS => bound(b"authority_grants", limits.authority_grants),
        path if path == paths::AUTHORITY_NOTE_RESOURCES => {
            bound(b"authority_note_resources", limits.authority_note_resources)
        }
        path if path == paths::DELEGATION_KINDS => bound(b"delegation_kinds", limits.delegation_kinds),
        path if path == paths::PART_KEY_PATH_VALUE => bound(b"part_key_path_value", limits.part_key_path_value),
        path if path == paths::PATTERN_END_EXACT_VALUE => {
            bound(b"pattern_end_exact_value", limits.pattern_end_exact_value)
        }
        path if path == paths::PATTERN_END_OPEN_VALUE => {
            bound(b"pattern_end_open_value", limits.pattern_end_open_value)
        }
        path if path == paths::PATTERN_SEGMENTS => bound(b"pattern_segments", limits.pattern_segments),
        path if path == paths::RULES_IMPLIES => bound(b"rules_implies", limits.rules_implies),
        path if path == paths::RULES_REQUIREMENTS => bound(b"rules_requirements", limits.rules_requirements),
        path if path == paths::SETTING_MODEL_EFFORT => {
            bound(b"setting_model_effort_some", limits.setting_model_effort_some)
        }
        path if path == paths::SETTING_MODEL_EFFORTS => bound(b"setting_model_efforts", limits.setting_model_efforts),
        path if path == paths::SETTING_MODEL_NAME => bound(b"setting_model_name", limits.setting_model_name),
        path if path == paths::SETTING_MODEL_PROGRAM => {
            bound(b"setting_model_program_some", limits.setting_model_program_some)
        }
        path if path == paths::SETTING_POLICY_PERMISSIONS => {
            bound(b"setting_policy_permissions", limits.setting_policy_permissions)
        }
        path if path == paths::SETTING_POLICY_PROJECTIONS => {
            bound(b"setting_policy_projections", limits.setting_policy_projections)
        }
        path if path == paths::SETTING_POLICY_REQUIREMENTS => {
            bound(b"setting_policy_requirements", limits.setting_policy_requirements)
        }
        path if path == paths::SETTING_POLICY_ROLES => bound(b"setting_policy_roles", limits.setting_policy_roles),
        path if path == paths::SETTING_POLICY_TEMPLATES => {
            bound(b"setting_policy_templates", limits.setting_policy_templates)
        }
        path if path == paths::TASK_CONTRACT_DOCUMENT_APPROVAL_STATE => {
            bound(b"task_contract_document_approval_state", limits.task_contract_document_approval_state)
        }
        path if path == paths::TASK_CONTRACT_DOCUMENT_VERDICT_CHOICES => {
            bound(b"task_contract_document_verdict_choices", limits.task_contract_document_verdict_choices)
        }
        path if path == paths::VERDICT_CHOICE_FIELDS => bound(b"verdict_choice_fields", limits.verdict_choice_fields),
        _ => error(b"value", b"wire value bound"),
    }
}

/// Verdict choices, like authority grants and settings items, are sized wire
/// pieces rather than a counted wire list. Check their byte envelope too.
pub(super) fn verdict_bound(
    choices: &List<c::VerdictChoice>,
    limits: &jig_client_protocol_wire::Limits,
) -> Result<(), Error> {
    use jig_client_protocol_wire as w;
    let mut size = 0_u32;
    for choice in choices {
        let mut fields = List::with_capacity(choice.fields.len());
        for field in &choice.fields {
            let field = match Ok(w::FieldRule { name: field.name.clone(), max: field.max }) {
                Ok(field) => field,
                Err(problem) => return Err(wire_error(problem, limits)),
            };
            fields.push(field).expect("same field count");
        }
        let row = match Ok(w::VerdictChoice {
            code: choice.code,
            words: choice.words,
            followups: choice.followups,
            fields,
        }) {
            Ok(row) => row,
            Err(problem) => return Err(wire_error(problem, limits)),
        };
        let measured = match row.measure(limits) {
            Ok(size) => size,
            Err(problem) => return Err(wire_error(problem, limits)),
        };
        size = size
            .checked_add(measured)
            .ok_or(bound(b"task_contract_document_verdict_choices", limits.task_contract_document_verdict_choices))?;
        size = size
            .checked_add(4)
            .ok_or(bound(b"task_contract_document_verdict_choices", limits.task_contract_document_verdict_choices))?;
        if size > limits.task_contract_document_verdict_choices {
            return Err(bound(
                b"task_contract_document_verdict_choices",
                limits.task_contract_document_verdict_choices,
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{message, wire_error};
    use alloc::boxed::Box;
    use jig_client_protocol_wire as wire;
    use jig_client_protocol_wire::Record as _;

    #[test]
    fn an_unlisted_wire_path_keeps_the_generic_diagnostic_words() {
        let mut limits = wire::CEILINGS;
        limits.keyed_request = 0;
        let problem = (wire::Keyed {
            key: [0; 16],
            owner: wire::Owner::Jig,
            version: 1,
            request: Box::from(b"x".as_slice()),
            value: None,
        })
        .encode(&limits)
        .expect_err("one request byte is beyond the zero-byte fixture limit");
        assert_eq!(problem.path, wire::paths::KEYED_REQUEST);
        assert_eq!(message(None, wire_error(problem, &limits)).as_ref(), b"settings file: value: wire value bound");
    }
}

#[cfg(test)]
#[path = "settings_file_tests.rs"]
mod wake_tests;

#[cfg(test)]
pub(crate) mod format_tests;

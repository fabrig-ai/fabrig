//! One bounded adoption draft, with typed values supplied by the view (client/README.md, 4.4).
use super::{Address, Adoption, Answer, Role, Served};
use alloc::boxed::Box;
use skein_lib::{RequestProgress, Token};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Text {
    Forge,
    Owner,
    Name,
    Prefix,
    Landing,
    Checks,
}
impl Text {
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Forge => 0,
            Self::Owner => 1,
            Self::Name => 2,
            Self::Prefix => 3,
            Self::Landing => 4,
            Self::Checks => 5,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Broken {
    Required,
    ForgeNumber,
    Checks,
    ContextHome,
    NoCheck,
    Long { bound: u32 },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Act {
    Go { address: Address },
    Write { field: Text, text: Box<[u8]>, typed: Typed },
    Choose { role: Role },
    Home,
    Ci,
    Adopt,
    SetUp { forge: u16 },
    SetUpAtOnce { forge: u16 },
    SetupWrite { field: super::forges::SetupField, text: Box<[u8]> },
    StartIn { forge: u16 },
    Picker { forge: u16, act: super::picker::Act },
    StartDemo,
    AdminWrite { field: super::admin::Field, text: Box<[u8]> },
    ConnectAdmin { forge: u16 },
    AppWrite { text: Box<[u8]> },
    ConnectApp { forge: u16 },
    ConnectManifest { forge: u16, back: Box<[u8]> },
    Install { forge: u16, back: Box<[u8]> },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Typed {
    Bytes,
    Forge(Option<u16>),
    Checks(Option<Box<[u32]>>),
}
#[derive(Clone, Debug, PartialEq, Eq)]
#[expect(
    clippy::partial_pub_fields,
    reason = "the view reads a field's presentation; typed values stay inside the form"
)]
pub struct Field {
    pub text: Box<[u8]>,
    pub written: u64,
    pub broken: Option<Broken>,
    typed: Typed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Check {
    pub on: bool,
    pub broken: Option<Broken>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Form {
    pub project: u32,
    pub texts: [Field; 6],
    pub role: Role,
    pub home: Check,
    pub ci: Check,
    pub pending: Option<(Token, RequestProgress)>,
    pub outcome: Option<Answer>,
    pub served: Served,
    pub confirmed: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Marks {
    pub texts: [Option<Broken>; 6],
    pub home: Option<Broken>,
}
fn field(text: &[u8], typed: Typed) -> Field {
    Field { text: Box::from(text), written: 0, broken: None, typed }
}
#[must_use]
pub fn form(project: u32) -> Form {
    Form {
        project,
        texts: [
            field(b"1", Typed::Forge(Some(1))),
            field(b"", Typed::Bytes),
            field(b"", Typed::Bytes),
            field(b"temper/", Typed::Bytes),
            field(b"", Typed::Bytes),
            field(b"", Typed::Checks(Some(Box::new([])))),
        ],
        role: Role::Owned,
        home: Check { on: true, broken: None },
        ci: Check { on: true, broken: None },
        pending: None,
        outcome: None,
        served: Served::Unknown,
        confirmed: false,
    }
}
fn forge_number(form: &Form) -> Option<u16> {
    let [forge, _, _, _, _, _] = &form.texts;
    match forge.typed {
        Typed::Forge(value) => match value {
            Some(0) | None => None,
            Some(value) => Some(value),
        },
        Typed::Bytes | Typed::Checks(_) => None,
    }
}
fn checks(form: &Form, most: u32) -> Option<&[u32]> {
    let [_, _, _, _, _, checks] = &form.texts;
    let checks = match &checks.typed {
        Typed::Checks(Some(checks)) => checks,
        Typed::Checks(None) | Typed::Bytes | Typed::Forge(_) => return None,
    };
    if checks.len() > usize::try_from(most).expect("u32 fits usize") {
        return None;
    }
    for (index, check) in checks.iter().enumerate() {
        if *check == 0 {
            return None;
        }
        for earlier in checks.get(..index).expect("enumerated prefix") {
            if earlier == check {
                return None;
            }
        }
    }
    Some(checks)
}
#[must_use]
pub fn broken(form: &Form, most: u32) -> Marks {
    let mut marks = Marks { texts: [None; 6], home: None };
    for text in [Text::Owner, Text::Name, Text::Prefix, Text::Landing] {
        if form.texts.get(text.index()).expect("six fields").text.is_empty() {
            *marks.texts.get_mut(text.index()).expect("six marks") = Some(Broken::Required);
        }
    }
    if forge_number(form).is_none() {
        *marks.texts.get_mut(Text::Forge.index()).expect("forge mark") = Some(Broken::ForgeNumber);
    }
    match checks(form, most) {
        None => {
            *marks.texts.get_mut(Text::Checks.index()).expect("checks mark") = Some(Broken::Checks);
        }
        Some(checks) => {
            if !form.ci.on && checks.is_empty() {
                *marks.texts.get_mut(Text::Checks.index()).expect("checks mark") = Some(Broken::NoCheck);
            }
        }
    }
    match form.role {
        Role::Context => {
            if form.home.on {
                marks.home = Some(Broken::ContextHome);
            }
        }
        Role::Owned | Role::Adopted | Role::Fork => {}
    }
    for (index, field) in form.texts.iter().enumerate() {
        match field.broken {
            Some(Broken::Long { bound }) => {
                *marks.texts.get_mut(index).expect("six fields") = Some(Broken::Long { bound });
            }
            None
            | Some(Broken::Required | Broken::ForgeNumber | Broken::Checks | Broken::ContextHome | Broken::NoCheck) => {
            }
        }
    }
    marks
}
#[must_use]
pub fn adoption(form: &Form, most: u32) -> Option<Adoption> {
    let marks = broken(form, most);
    if marks.home.is_some() {
        return None;
    }
    for mark in marks.texts {
        if mark.is_some() {
            return None;
        }
    }
    let [_, owner, name, prefix, landing, _] = &form.texts;
    Some(Adoption {
        project: form.project,
        home: form.home.on,
        forge: forge_number(form)?,
        owner: owner.text.clone(),
        name: name.text.clone(),
        prefix: prefix.text.clone(),
        role: form.role,
        landing: landing.text.clone(),
        ci: form.ci.on,
        checks: Box::from(checks(form, most)?),
    })
}
pub fn write(form: &mut Form, text: Text, value: Box<[u8]>, typed: Typed, bound: u32, most: u32) {
    form.outcome = None;
    let field = form.texts.get_mut(text.index()).expect("six form fields");
    // A refused write also sends the retained value back to the view.
    field.written = field.written.checked_add(1).expect("field writes fit u64");
    if value.len() > usize::try_from(bound).expect("u32 fits usize") {
        field.broken = Some(Broken::Long { bound });
        return;
    }
    field.text = value;
    field.typed = match typed {
        Typed::Checks(Some(checks)) => {
            if checks.len() > usize::try_from(most).expect("u32 fits usize") {
                Typed::Checks(None)
            } else {
                Typed::Checks(Some(checks))
            }
        }
        Typed::Checks(None) | Typed::Bytes | Typed::Forge(_) => typed,
    };
    field.broken = None;
}
pub(super) fn mark(form: &mut Form, most: u32) {
    let marks = broken(form, most);
    for (field, mark) in form.texts.iter_mut().zip(marks.texts) {
        field.broken = mark;
    }
    form.home.broken = marks.home;
}
pub(super) fn reset(form: &mut Form, answer: Answer) {
    let mut defaults = self::form(form.project);
    defaults.served = form.served;
    defaults.confirmed = form.confirmed;
    for (field, old) in defaults.texts.iter_mut().zip(&form.texts) {
        field.written = old.written.checked_add(1).expect("field writes fit u64");
    }
    defaults.outcome = Some(answer);
    *form = defaults;
}

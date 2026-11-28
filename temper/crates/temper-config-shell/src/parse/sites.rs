//! Positions in the source TOML, kept alongside the values while keys move.
use std::borrow::Cow;
use toml::{
    Table, Value,
    de::{DeTable, DeValue},
};

pub(super) type Fields<'a> = Vec<(Cow<'a, str>, Site<'a>)>;

pub(super) struct Site<'a> {
    pub(super) offset: usize,
    pub(super) children: Children<'a>,
}

pub(super) enum Children<'a> {
    Table(Fields<'a>),
    Array(Vec<Site<'a>>),
    Leaf,
}

pub(super) fn fields<'a>(table: &'a DeTable<'_>) -> Fields<'a> {
    table
        .iter()
        .map(|(key, value)| (Cow::Borrowed(key.get_ref().as_ref()), parsed(key.span().start, value.get_ref())))
        .collect()
}

fn parsed<'a>(offset: usize, value: &'a DeValue<'_>) -> Site<'a> {
    let children = match value {
        DeValue::Table(table) => Children::Table(fields(table)),
        DeValue::Array(array) => {
            Children::Array(array.iter().map(|item| parsed(item.span().start, item.get_ref())).collect())
        }
        DeValue::String(_) | DeValue::Integer(_) | DeValue::Float(_) | DeValue::Boolean(_) | DeValue::Datetime(_) => {
            Children::Leaf
        }
    };
    Site { offset, children }
}

impl<'a> Site<'a> {
    /// A conversion's newly constructed values all come from its original key.
    pub(super) fn converted(offset: usize, value: &Value) -> Self {
        let children = match value {
            Value::Table(table) => Children::Table(
                table.iter().map(|(key, item)| (Cow::Owned(key.clone()), Self::converted(offset, item))).collect(),
            ),
            Value::Array(array) => Children::Array(array.iter().map(|item| Self::converted(offset, item)).collect()),
            Value::String(_) | Value::Integer(_) | Value::Float(_) | Value::Boolean(_) | Value::Datetime(_) => {
                Children::Leaf
            }
        };
        Self { offset, children }
    }

    pub(super) fn table(offset: usize) -> Self {
        Self { offset, children: Children::Table(Vec::new()) }
    }

    pub(super) fn table_mut(&mut self) -> &mut Fields<'a> {
        let Children::Table(fields) = &mut self.children else { unreachable!("table site") };
        fields
    }
}

/// Remove a move's source without leaving an empty table that no longer exists in the schema.
pub(super) fn take<'a>(
    table: &mut Table,
    fields: &mut Fields<'a>,
    key: &str,
    parent: &str,
) -> Option<(Value, Site<'a>)> {
    if let Some((name, rest)) = key.split_once('.') {
        let index = fields.iter().position(|(key, _)| *key == name)?;
        let child = table.get_mut(name)?.as_table_mut()?;
        let path = if parent.is_empty() { name.to_owned() } else { format!("{parent}.{name}") };
        let found = take(child, fields[index].1.table_mut(), rest, &path)?;
        if child.is_empty() && crate::rows::find(&path).is_none() {
            table.remove(name);
            fields.remove(index);
        }
        Some(found)
    } else {
        let value = table.remove(key)?;
        let index = fields.iter().position(|(name, _)| *name == key).expect("source site");
        Some((value, fields.remove(index).1))
    }
}

/// Refuse a collision, rather than discarding either of the written values.
pub(super) fn put<'a>(
    table: &mut Table,
    fields: &mut Fields<'a>,
    key: &'static str,
    value: Value,
    site: Site<'a>,
) -> bool {
    if let Some((name, rest)) = key.split_once('.') {
        if !table.contains_key(name) {
            table.insert(name.into(), Value::Table(Table::new()));
            fields.push((Cow::Borrowed(name), Site::table(site.offset)));
        }
        let Some(child) = table.get_mut(name).and_then(Value::as_table_mut) else { return false };
        let (_, field) = fields.iter_mut().find(|(key, _)| *key == name).expect("destination site");
        put(child, field.table_mut(), rest, value, site)
    } else if table.contains_key(key) {
        false
    } else {
        table.insert(key.into(), value);
        fields.push((Cow::Borrowed(key), site));
        true
    }
}

//! Concrete accepted result decoding (smith/docs/design/protocol/charter.md, section 5;
//! smith/docs/design/protocol/channel.md, section 5). No state or policy is kept here.
use crate::Error;
use alloc::boxed::Box;
use skein_lib::{List, Reader, Writer};
use smith_charter as wire;
use smith_domain::run;

/// Decode the accepted outcome body after the host channel has bounded it.
pub fn decode_declared(bytes: &[u8], limits: &smith_charter::v4::Limits) -> Result<run::outcome::Declared, Error> {
    let mut reader = Reader::new(bytes);
    let Ok(record) = smith_charter::RunResult::decode(limits, &mut reader) else {
        return Err(Error::InvalidResult);
    };
    let fields = decode_fields(record.fields())?;
    let declared = match record.form() {
        smith_charter::Form::Change => {
            if record.label().is_some() || !record.text().is_empty() || !record.items().is_empty() {
                return Err(Error::InvalidResult);
            }
            run::outcome::Declared::Change(run::outcome::Change { fields })
        }
        smith_charter::Form::Report => {
            if record.label().is_some() || !record.items().is_empty() {
                return Err(Error::InvalidResult);
            }
            run::outcome::Declared::Report(run::outcome::Report { text: Box::from(record.text()), fields })
        }
        smith_charter::Form::Failure => {
            if record.label().is_some() || !record.items().is_empty() {
                return Err(Error::InvalidResult);
            }
            run::outcome::Declared::Failure(run::outcome::DeclaredFailure { reason: Box::from(record.text()), fields })
        }
        smith_charter::Form::Verdict => {
            let Some(label) = record.label() else { return Err(Error::InvalidResult) };
            let mut items = List::with_capacity(record.items().len());
            for item in record.items() {
                let value = run::outcome::Item { kind: Box::from(item.kind()), fields: decode_fields(item.fields())? };
                if items.push(value).is_err() {
                    return Err(Error::InvalidResult);
                }
            }
            run::outcome::Declared::Verdict(run::outcome::Verdict {
                name: label.clone(),
                text: Box::from(record.text()),
                fields,
                items: items.into_boxed(),
            })
        }
    };
    Ok(declared)
}

fn decode_fields(source: &List<smith_charter::Field>) -> Result<Box<[run::outcome::Field]>, Error> {
    let mut fields = List::with_capacity(source.len());
    for field in source {
        let item = run::outcome::Field { name: Box::from(field.name()), value: Box::from(field.text()) };
        if fields.push(item).is_err() {
            return Err(Error::InvalidResult);
        }
    }
    Ok(fields.into_boxed())
}

/// Write an accepted domain result in the current charter family.
pub fn encode_result(result: &run::outcome::Declared, limits: &wire::v4::Limits) -> Result<Box<[u8]>, Error> {
    let mut fields = List::with_capacity(limits.run_result_fields);
    let mut items = List::with_capacity(limits.run_result_items);
    let (form, label, text, values) = match result {
        run::outcome::Declared::Change(change) => (wire::Form::Change, None, &[][..], &change.fields[..]),
        run::outcome::Declared::Verdict(verdict) => {
            for item in &verdict.items {
                let mut item_fields = List::with_capacity(limits.item_fields);
                copy_fields(&item.fields, &mut item_fields, limits)?;
                let item =
                    match wire::Item::new(limits, wire::ItemParts { kind: item.kind.clone(), fields: item_fields }) {
                        Ok(value) => value,
                        Err(problem) => return Err(Error::CharterBody(problem)),
                    };
                if items.push(item).is_err() {
                    return Err(Error::Capacity);
                }
            }
            (wire::Form::Verdict, Some(verdict.name.clone()), &verdict.text[..], &verdict.fields[..])
        }
        run::outcome::Declared::Report(report) => (wire::Form::Report, None, &report.text[..], &report.fields[..]),
        run::outcome::Declared::Failure(failure) => {
            (wire::Form::Failure, None, &failure.reason[..], &failure.fields[..])
        }
    };
    copy_fields(values, &mut fields, limits)?;
    let record = match wire::RunResult::new(
        limits,
        wire::RunResultParts { form, label, text: Box::from(text), fields, items },
    ) {
        Ok(value) => value,
        Err(problem) => return Err(Error::CharterBody(problem)),
    };
    let Ok(length) = usize::try_from(record.measure()) else {
        return Err(Error::Capacity);
    };
    let mut writer = Writer::new(length);
    match record.encode(&mut writer) {
        Ok(value) => value,
        Err(problem) => return Err(Error::Write(problem)),
    }
    Ok(writer.finish())
}

fn copy_fields(
    values: &[run::outcome::Field],
    target: &mut List<wire::Field>,
    limits: &wire::v4::Limits,
) -> Result<(), Error> {
    for field in values {
        let value =
            match wire::Field::new(limits, wire::FieldParts { name: field.name.clone(), text: field.value.clone() }) {
                Ok(value) => value,
                Err(problem) => return Err(Error::CharterBody(problem)),
            };
        if target.push(value).is_err() {
            return Err(Error::Capacity);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::decode_declared;
    use alloc::boxed::Box;
    use smith_domain::run;
    #[test]
    fn a_change_and_report_decode_from_the_agent_result_body() {
        let change = run::outcome::Declared::Change(run::outcome::Change {
            fields: Box::from([run::outcome::Field {
                name: Box::from(&b"title"[..]),
                value: Box::from(&b"Commit"[..]),
            }]),
        });
        let bytes = crate::encode_result(&change, &smith_charter::CEILINGS).expect("change bytes");
        assert_eq!(decode_declared(&bytes, &smith_charter::CEILINGS), Ok(change));
        let report = run::outcome::Declared::Report(run::outcome::Report {
            text: Box::from(&b"finished"[..]),
            fields: Box::new([]),
        });
        let bytes = crate::encode_result(&report, &smith_charter::CEILINGS).expect("report bytes");
        assert_eq!(decode_declared(&bytes, &smith_charter::CEILINGS), Ok(report));
    }
}

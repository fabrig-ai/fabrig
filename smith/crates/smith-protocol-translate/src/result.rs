//! Concrete accepted result decoding (smith/docs/design/protocol/charter.md, section 5;
//! smith/docs/design/protocol/channel.md, section 5). No state or policy is kept here.
use crate::Error;
use crate::generated::charter as mapped;
use alloc::boxed::Box;
use skein_lib::{List, Reader};
use smith_charter as wire;
use smith_charter::Record;
use smith_domain::run;

/// Decode the accepted outcome body after the host channel has bounded it.
pub fn decode_declared(bytes: &[u8], limits: &smith_charter::v5::Limits) -> Result<run::outcome::Declared, Error> {
    let mut reader = Reader::new(bytes);
    let Ok(record) = smith_charter::RunResult::decode(limits, &mut reader) else {
        return Err(Error::InvalidResult);
    };
    let fields = decode_fields(record.fields);
    let declared = match record.form {
        smith_charter::Form::Change => {
            if record.label.is_some() || !record.text.as_ref().is_empty() || !record.items.is_empty() {
                return Err(Error::InvalidResult);
            }
            run::outcome::Declared::Change(run::outcome::Change { fields })
        }
        smith_charter::Form::Report => {
            if record.label.is_some() || !record.items.is_empty() {
                return Err(Error::InvalidResult);
            }
            run::outcome::Declared::Report(run::outcome::Report { text: record.text, fields })
        }
        smith_charter::Form::Failure => {
            if record.label.is_some() || !record.items.is_empty() {
                return Err(Error::InvalidResult);
            }
            run::outcome::Declared::Failure(run::outcome::DeclaredFailure { reason: record.text, fields })
        }
        smith_charter::Form::Verdict => {
            let Some(label) = record.label else { return Err(Error::InvalidResult) };
            let mut items = List::with_capacity(record.items.len());
            for item in record.items.into_boxed() {
                assert!(items.push(mapped::item_to_domain(item)).is_ok(), "one output per item");
            }
            run::outcome::Declared::Verdict(run::outcome::Verdict {
                name: label,
                text: record.text,
                fields,
                items: items.into_boxed(),
            })
        }
    };
    Ok(declared)
}

fn decode_fields(source: List<wire::Field>) -> Box<[run::outcome::Field]> {
    let mut fields = List::with_capacity(source.len());
    for field in source.into_boxed() {
        assert!(fields.push(mapped::field_to_domain(field)).is_ok(), "one output per field");
    }
    fields.into_boxed()
}

/// Write an accepted domain result in the current charter family.
pub fn encode_result(result: &run::outcome::Declared, limits: &wire::v5::Limits) -> Result<Box<[u8]>, Error> {
    let mut fields = List::with_capacity(limits.run_result_fields);
    let mut items = List::with_capacity(limits.run_result_items);
    let (form, label, text, values) = match result {
        run::outcome::Declared::Change(change) => (wire::Form::Change, None, &[][..], &change.fields[..]),
        run::outcome::Declared::Verdict(verdict) => {
            for item in &verdict.items {
                if item.fields.len() > skein_lib::usize_of(limits.item_fields) {
                    return Err(Error::Capacity);
                }
                let item = mapped::item_to_record(item).or(Err(Error::Capacity))?;
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
    copy_fields(values, &mut fields)?;
    let record = wire::RunResult { form, label, text: Box::from(text), fields, items };
    match record.encode(limits) {
        Ok(bytes) => Ok(bytes),
        Err(problem) => Err(Error::CharterBody(problem)),
    }
}

fn copy_fields(values: &[run::outcome::Field], target: &mut List<wire::Field>) -> Result<(), Error> {
    for field in values {
        if target.push(mapped::field_to_record(field)).is_err() {
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

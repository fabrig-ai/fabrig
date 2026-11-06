//! Translate the owned grants of a prepared assignment to the fleet.
use alloc::boxed::Box;
use jig_domain_link::{Form, Grant, Scheme};
use skein_lib::List;

pub(crate) fn fleet(grants: &[Grant]) -> Box<[Grant]> {
    let mut translated = List::with_capacity(u32::try_from(grants.len()).expect("bounded grants"));
    for grant in grants {
        translated
            .push(Grant {
                form: fleet_form(grant.form),
                account: grant.account,
                generation: grant.generation,
                valid: grant.valid,
            })
            .expect("grant room");
    }
    translated.into_boxed()
}

fn fleet_form(form: Form) -> Form {
    match form {
        Form::Credential(scheme) => Form::Credential(match scheme {
            Scheme::Bearer => Scheme::Bearer,
            Scheme::Key => Scheme::Key,
        }),
        Form::Application => Form::Application,
        Form::Program(name) => Form::Program(name),
    }
}

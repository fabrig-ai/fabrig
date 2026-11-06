//! Translate the owned grants of a prepared assignment to the fleet.
use alloc::boxed::Box;
use jig_domain_fleet as fleet;
use skein_lib::List;

pub(crate) fn fleet(grants: &[jig_domain_link::Grant]) -> Box<[fleet::Grant]> {
    let mut translated = List::with_capacity(u32::try_from(grants.len()).expect("bounded grants"));
    for grant in grants {
        translated
            .push(fleet::Grant {
                form: fleet_form(grant.form),
                account: grant.account,
                generation: grant.generation,
                valid: grant.valid,
            })
            .expect("grant room");
    }
    translated.into_boxed()
}

fn fleet_form(form: jig_domain_link::Form) -> fleet::Form {
    match form {
        jig_domain_link::Form::Credential(scheme) => fleet::Form::Credential(match scheme {
            jig_domain_link::Scheme::Bearer => fleet::Scheme::Bearer,
            jig_domain_link::Scheme::Key => fleet::Scheme::Key,
        }),
        jig_domain_link::Form::Application => fleet::Form::Application,
        jig_domain_link::Form::Program(name) => fleet::Form::Program(name),
    }
}

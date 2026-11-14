use crate::render;

pub(super) fn fault_back(fault: crate::records::errors::ExportFault) -> render::errors::ExportFault {
    use crate::records::errors::{ExportFault, Root};
    match fault {
        ExportFault::Form => render::errors::ExportFault::Form,
        ExportFault::Root { root } => render::errors::ExportFault::Root {
            root: match root {
                Root::Tasks => render::errors::Root::Tasks,
                Root::Exports => render::errors::Root::Exports,
                Root::Debian => render::errors::Root::Debian,
            },
        },
    }
}

const FAULTS: &[render::errors::ExportFault] = &[
    render::errors::ExportFault::Form,
    render::errors::ExportFault::Root { root: render::errors::Root::Tasks },
    render::errors::ExportFault::Root { root: render::errors::Root::Exports },
    render::errors::ExportFault::Root { root: render::errors::Root::Debian },
];
#[test]
fn each_export_fault_translates_and_back() {
    for (row, input) in FAULTS.iter().enumerate() {
        let crate::records::errors::Schema::Export { fault } =
            crate::translate::errors::schema(render::errors::Schema::Export { fault: *input })
        else {
            panic!("export fault")
        };
        let back = fault_back(fault);
        assert_eq!(back, *input, "row {row}: export fault");
    }
}

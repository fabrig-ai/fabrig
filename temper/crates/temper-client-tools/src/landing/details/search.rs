//! Search selection and pass words (temper/docs/design/domain/landing-search-by-test.md, 8).
use super::{Encoder, Text, hex, w};

pub(super) fn text(value: &w::SearchTests, out: &mut Text) {
    match value {
        w::SearchTests::Whole => {}
        w::SearchTests::WholeAgain => out.put(b"search: whole again\n"),
        w::SearchTests::ByTest(value) => {
            for gate in value.gates() {
                out.put(b"search by test: gate ");
                out.put(gate.name());
                out.put(b", ");
                out.number(u64::from(gate.count()));
                out.put(b" tests: ");
                for (index, id) in gate.ids().iter().enumerate() {
                    if index != 0 {
                        out.put(b", ");
                    }
                    out.put(id);
                }
                if gate.count() > gate.ids().len() {
                    out.put(", …".as_bytes());
                }
                out.put(b"\n");
            }
        }
    }
}
pub(super) fn passes_text(value: &w::SearchTests, prefix: &[u8; 32], out: &mut Text) {
    if let w::SearchTests::ByTest(value) = value {
        for pass in value.passes() {
            if pass.prefix() == prefix {
                out.put(b"  gate ");
                out.put(pass.name());
                out.put(b" green by test in run ");
                out.number(pass.run());
                out.put(b"\n");
            }
        }
    }
}
pub(super) fn json(value: &w::SearchTests, out: &mut Encoder) {
    out.object_start();
    out.key(b"mode");
    match value {
        w::SearchTests::Whole => out.string(b"whole"),
        w::SearchTests::WholeAgain => out.string(b"whole_again"),
        w::SearchTests::ByTest(value) => {
            out.string(b"by_test");
            out.key(b"gates");
            out.array_start();
            for gate in value.gates() {
                out.object_start();
                out.key(b"gate");
                out.unsigned(u64::from(gate.gate()));
                out.key(b"name");
                out.string(gate.name());
                out.key(b"count");
                out.unsigned(u64::from(gate.count()));
                out.key(b"ids");
                out.array_start();
                for id in gate.ids() {
                    out.string(id);
                }
                out.array_end();
                out.object_end();
            }
            out.array_end();
            out.key(b"passes");
            out.array_start();
            for pass in value.passes() {
                out.object_start();
                out.key(b"member");
                out.unsigned(u64::from(pass.member()));
                out.key(b"prefix");
                out.string(&hex(pass.prefix()));
                out.key(b"gate");
                out.unsigned(u64::from(pass.gate()));
                out.key(b"name");
                out.string(pass.name());
                out.key(b"run");
                out.unsigned(pass.run());
                out.object_end();
            }
            out.array_end();
        }
    }
    out.object_end();
}

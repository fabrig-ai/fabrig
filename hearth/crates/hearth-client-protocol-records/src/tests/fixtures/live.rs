use hearth_domain_render_vocabulary as render;

pub fn live_spec() -> crate::records::Record {
    let crate::records::Record::Spec(mut spec) = super::secret_spec() else { panic!("spec") };
    spec.parts = Box::new([render::parts::Part {
        digest: hearth_lib::encode::Digest([0; 32]),
        record: render::parts::PartRecord::Live(render::parts::live::Live {
            paths: Box::new([render::parts::live::Covered {
                path: Box::from(&b"/opt/tool/"[..]),
                consumers: Box::new([render::parts::live::Consumer {
                    task: Box::from(&b"motd"[..]),
                    unit: None,
                    action: render::parts::live::Action::None,
                }]),
            }]),
        }),
    }]);
    crate::records::Record::Spec(spec)
}

pub fn software_spec() -> crate::records::Record {
    use hearth_domain_render_vocabulary as render;
    use render::parts::software::Download;
    use render::parts::software::Fetch;
    use render::parts::software::How;
    use render::parts::software::Software;
    let crate::records::Record::Spec(mut spec) = super::secret_spec() else { panic!("spec") };
    spec.parts = Box::new([render::parts::Part {
        digest: hearth_lib::encode::Digest([0; 32]),
        record: render::parts::PartRecord::Software(Software {
            downloads: Box::new([
                Download {
                    path: b"/opt/a".as_slice().into(),
                    sha256: hearth_lib::encode::Digest([1; 32]),
                    owner: b"root".as_slice().into(),
                    group: b"staff".as_slice().into(),
                    how: How::File { mode: 0o755 },
                },
                Download {
                    path: b"/opt/b".as_slice().into(),
                    sha256: hearth_lib::encode::Digest([1; 32]),
                    owner: b"root".as_slice().into(),
                    group: b"staff".as_slice().into(),
                    how: How::Tar { strip: 1 },
                },
            ]),
            scripts: Box::new([super::directives::script([1; 32])]),
            fetch: Box::new([Fetch {
                sha256: hearth_lib::encode::Digest([1; 32]),
                url: b"https://example.org/a".as_slice().into(),
            }]),
        }),
    }]);
    crate::records::Record::Spec(spec)
}

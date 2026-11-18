use super::{path, shell};
use shell::stopped;
use temper_engine_service as service;
use temper_shell::directories::Directories;

// Bring the copied floor to this build once, then stop it before testing the tool.
pub(super) fn upgrade_fixture(directories: &Directories) {
    use std::os::fd::IntoRawFd;
    let listeners = [0; 3].map(|_| std::net::TcpListener::bind("127.0.0.1:0").unwrap());
    let document_path = directories.document();
    let mut document: serde_json::Value = serde_json::from_slice(&std::fs::read(&document_path).unwrap()).unwrap();
    for (pointer, listener) in [
        "/application/clients/listener/socket",
        "/engine/hosts/listener/0/socket",
        "/application/webhooks/listener/socket",
    ]
    .iter()
    .zip(&listeners)
    {
        *document.pointer_mut(pointer).unwrap() = listener.local_addr().unwrap().to_string().into();
    }
    std::fs::write(document_path, serde_json::to_vec(&document).unwrap()).unwrap();
    let (mut service, deployment) = stopped::prepare("temper-engine", directories).unwrap();
    for listener in listeners {
        let address = listener.local_addr().unwrap();
        service.adopt_listener(skein_io::kernel::Fd::new(listener.into_raw_fd()), address).unwrap();
    }
    let root = skein_shell::open_root(path(&deployment)).unwrap();
    service.adopt_store(root, skein_shell::effective_user()).unwrap();
    let served = temper_service_shell::Served {
        iterate_stage: |service: &mut service::Service, stage, now, wall| {
            service::iterate_stage(service, stage, now, wall);
            if stage == service::Stage::Engine
                && service.engine_component().domain().ready()
                && !service.engine_component().stopping()
            {
                service.shutdown();
                service.shutdown();
            }
        },
        ..shell::served::SERVED
    };
    let mut host = temper_service_shell::Host::with_binary("temper-engine", service, served, directories.clone(), &[]);
    let mut kernel = skein_shell::Kernel::open(skein_shell::Config { operations: host.service.operations() }).unwrap();
    let exit = skein_shell::drive(&mut kernel, &skein_shell::Clock::new(), &mut host);
    assert_eq!(exit, skein_io::kernel::Exit::Code(1), "fixture abort closes the upgraded store");
    assert_eq!(host.service.end(), Some(&service::End::Aborted));
}

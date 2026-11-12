pub fn kernel(service: &mut crate::Service) {
    super::Kernel::new().answer(service);
}

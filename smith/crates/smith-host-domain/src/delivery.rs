//! Sealed run delivery values shared by the host (domain/host.md, section 3;
//! domain/run.md, section 8). Constructors bound ownership before admission;
//! the host retains the run's vocabulary without copying terminal evidence.
pub use smith_domain::run::{
    Delivered, Delivery, DeliveryFailure, DeliveryReason, DeliveryRefusal, Diagnostic, MAX_DIRECTORIES, Marker, Receipt,
};

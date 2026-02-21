pub mod types;
pub mod discovery;
pub mod authentication;
pub mod transfer;
pub mod session;
pub mod connection;
pub mod error;
pub mod mdns;
pub mod ble;
pub mod receipt_helper;
pub mod qr;
pub mod platform;
pub mod permissions;
pub mod lifecycle;

pub use types::*;
pub use error::{ProximityError, Result, ErrorContext, ErrorCategory};
pub use discovery::DiscoveryService;
pub use authentication::{AuthenticationService, AuthenticationProof, Challenge};
pub use connection::{PeerConnection, PeerConnectionManager, RetryConfig};
pub use transfer::TransferService;
pub use session::SessionManager;
pub use receipt_helper::ProximityReceiptData;
pub use qr::QrCodeService;
pub use platform::{PlatformConnection, PlatformConnectionFactory, get_default_factory};
pub use permissions::{PermissionManager, PermissionStatus};
pub use lifecycle::{LifecycleManager, AppState};


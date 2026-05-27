#[cfg(feature = "domain")]
pub mod email;
#[cfg(feature = "domain")]
pub mod user;
#[cfg(feature = "domain")]
pub mod user_id;

#[cfg(feature = "domain")]
pub use user::*;

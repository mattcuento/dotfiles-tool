pub mod doctor;
pub mod init;
pub mod setup;

pub use doctor::run as doctor;
pub use init::run as init;
pub use setup::run as setup;

use log::info;
use super::Status;

pub fn set_status(status: Status) {
    match status {
        Status::None => {
            info!("Status: None");
        }
        Status::Start => {
            info!("Status: Start");
        }
        Status::WifiConnected => {
            info!("Status: WifiConnected");
        }
        Status::Activated => {
            info!("Status: Activated");
        }
        Status::Deactivated => {
            info!("Status: Deactivated");
        }
        Status::ClipboardSize(_) => {
            info!("Status: ClipboardSize");
        }
    }
}

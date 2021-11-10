use druid::{Data, Lens};
use uuid::Uuid;

use crate::views::theme::Theme;
use signal::AppData as SignalAppData;
use matrix::AppData as MatrixAppData;
pub static mut OWNER: String = String::new();
#[derive(Debug, Data, Clone, PartialEq, Eq)]
pub enum Platform {
    Signal,
    Matrix,
}

impl Default for Platform {
    fn default() -> Self {
        Self::Signal
    }
}

#[derive( Data, Clone)]
pub enum PlatformAppData {
    Signal(SignalAppData),
    Matrix(MatrixAppData),
}

/*impl Default for PlatformAppData {
    fn default() -> Self {
        Self::Signal
    }
}*/


#[derive(Data, Clone, Lens)]
pub struct AppState {
    pub data: PlatformAppData,
    pub theme: Theme,
    platform: Platform
}

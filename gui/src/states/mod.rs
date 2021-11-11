use druid::im::{HashMap, Vector};
use druid::{Data, Lens};
use std::path::PathBuf;
// use uuid::Uuid;

use crate::views::style::Theme;
use matrix::AppData as MatrixAppData;
use signal::signal::{GROUP_IDENTIFIER_LEN, GROUP_MASTER_KEY_LEN};
// use signal::{Channel, ChannelId};
// use signal::AppData as SignalAppData;

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

#[derive(Data, Clone)]
pub enum PlatformAppData {
    Signal(SignalState),
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
    platform: Platform,
}

#[derive(Data, Lens, Clone, Default, Debug)]
pub struct SignalData {
    pub channels: HashMap<ChannelId, Channel>,
    pub names: HashMap<String, String>,
    pub input: String,
    pub current_channel: Option<ChannelId>,
    pub current_platform: Option<Platform>,
    pub user_id: String,
}

#[derive(Data, Lens, Clone, Default, Debug)]
pub struct SignalState {
    pub data: SignalData,
    pub theme: Theme,
}

#[derive(Debug, Data,  Clone,Eq,PartialEq, Hash)]
pub enum ChannelId {
    User(String),
    Group(GroupIdentifierBytes),
}

#[derive(Debug, Data, Lens, Clone)]
pub struct Channel {
    pub id: ChannelId,
    pub name: String,
    pub group_data: Option<GroupData>,
    pub messages: Vector<Message>,
    pub unread_messages: usize,
    // pub selected: Option<Message>
}
#[derive(Debug, Data, Lens, Clone)]
pub struct GroupData {
    pub master_key_bytes: GroupMasterKeyBytes,
    pub members: Vector<String>,
    pub revision: u32,
}

pub type GroupMasterKeyBytes = [u8; GROUP_MASTER_KEY_LEN];
pub type GroupIdentifierBytes = [u8; GROUP_IDENTIFIER_LEN];

#[derive(Debug, Data, Lens, Clone)]
pub struct Message {
    pub from_id: String,
    pub message: Option<String>,
    #[data(ignore)]
    pub quote: Option<Box<Message>>,
    pub attachments: Vector<Attachment>,
    pub reactions: Vector<(String, String)>,
}

#[derive(Debug, Data, Lens, Clone)]
pub struct Attachment {
    pub id: String,
    pub content_type: String,
    pub filename: String,
    pub size: u64,
}

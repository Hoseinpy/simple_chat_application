use std::{collections::HashMap, sync::Arc};

use tokio::sync::{Mutex, broadcast::Sender};
use uuid::Uuid;

pub type DefaultError = Box<dyn std::error::Error>;
pub type Channel = Arc<Mutex<HashMap<Uuid, Sender<String>>>>;

use std::str::FromStr;

use uuid::Uuid;

use crate::registry_error::{RegistryError, RegistryResult};

#[derive(Debug, Clone)]
pub struct SessionId(Uuid);

impl SessionId {
    pub fn parse(session_id: &str) -> RegistryResult<Self> {
        match Uuid::from_str(session_id) {
            Ok(id) => Ok(Self(id)),
            Err(e) => {
                error!("Failed to parse session id ({session_id}), err: {e:?}");
                Err(RegistryError::InvalidSessionId)
            }
        }
    }
}

impl From<SessionId> for Uuid {
    fn from(value: SessionId) -> Self {
        value.0
    }
}

impl From<Uuid> for SessionId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

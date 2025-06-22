#[derive(Debug)]
pub enum ClientMessage {
    Connect = 1,
    Disconnect,
}

#[derive(Debug)]
pub enum ClientMessageError {
    Empty,
    InvalidRequest,
}

impl TryFrom<u8> for ClientMessage {
    type Error = ClientMessageError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value == 0 {
            return Err(ClientMessageError::InvalidRequest);
        }
        if value == 1 {
            return Ok(ClientMessage::Connect);
        }
        if value == 2 {
            return Ok(ClientMessage::Disconnect);
        }

        Err(ClientMessageError::InvalidRequest)
    }
}

impl From<ClientMessage> for u8 {
    fn from(value: ClientMessage) -> Self {
        value as u8
    }
}

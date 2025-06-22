#[derive(Debug)]
pub enum ServerRequest {
    Connect = 1,
    Disconnect,
    NewPane,
}

#[derive(Debug)]
pub enum RequestError {
    Empty,
    InvalidRequest,
}

impl TryFrom<u8> for ServerRequest {
    type Error = RequestError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value == 0 {
            return Err(RequestError::InvalidRequest);
        }

        match value {
            1 => Ok(Self::Connect),
            2 => Ok(Self::Disconnect),
            3 => Ok(Self::NewPane),
            _ => Err(RequestError::InvalidRequest),
        }
    }
}

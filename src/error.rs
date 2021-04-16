#[derive(Debug)]
pub enum OMXError {
    CreateComponentFailed,
    UnableToGetParameter,
    UnableToSetParameter,
    UnableToSetConfig,
    InvalidNumberOfPorts,
    SendCommandFailed,
    UseBufferFailed,
    EmptyBufferFailed,
    EventTimeout,
}

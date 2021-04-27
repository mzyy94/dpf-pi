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
    FreeBufferFailed,
    EventTimeout,
    SetupTunnelFailed,
}

#[derive(Debug)]
pub enum PipelineError {
    ILClientError(Operation, i32),
    OMXError(Operation, i32),
    Assertion(Operation),
}

#[derive(Debug)]
pub enum Operation {
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

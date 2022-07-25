use crate::command::Command;

#[derive(Debug)]
pub enum Error {
    Pdu(PduError),
    WorkingCounter {
        expected: u16,
        received: u16,
        context: Option<&'static str>,
    },
    /// There is not enough storage to hold the number of detected slaves.
    TooManySlaves,
}

#[derive(Debug)]
pub enum PduError {
    Timeout,
    IndexInUse,
    Send,
    Decode,
    TooLong,
    CreateFrame(smoltcp::Error),
    Encode(cookie_factory::GenError),
    Address,
}

#[derive(Copy, Clone, Debug)]
pub enum PduValidationError {
    IndexMismatch { sent: Command, received: Command },
    CommandMismatch { sent: Command, received: Command },
}

// TODO: Can I just replace the `context` field with some logging instead?
#[macro_export]
macro_rules! check_working_counter {
    ($received:expr, $expected:expr, $msg:expr) => {{
        if $received == $expected {
            Result::<(), Error>::Ok(())
        } else {
            Result::<(), Error>::Err(Error::WorkingCounter {
                expected: $expected,
                received: $received,
                context: Some($msg),
            })
        }
    }};
    ($received:expr, $expected:expr) => {{
        if $received == $expected {
            Result::<(), Error>::Ok(())
        } else {
            Result::<(), Error>::Err(Error::WorkingCounter {
                expected: $expected,
                received: $received,
                context: None,
            })
        }
    }};
}

impl From<PduError> for Error {
    fn from(e: PduError) -> Self {
        Self::Pdu(e)
    }
}

#[derive(Debug)]
pub enum MenuError {
    /// Too many arguments (expected, actual)
    TooManyArgs(u8, u8),
    /// Not enough arguments (expected, actual)
    NotEnoughArgs(u8, u8),
    /// Command not found
    CommandNotFound,
    /// Write stdout/stderr error
    WriteError(core::fmt::Error),
    /// Command error
    CommandError(Option<&'static str>),
    /// Invalid Argument
    InvalidArgument,
}

impl From<core::fmt::Error> for MenuError {
    fn from(err: core::fmt::Error) -> Self {
        Self::WriteError(err)
    }
}

impl core::fmt::Display for MenuError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::TooManyArgs(expected, actual) | Self::NotEnoughArgs(expected, actual) => {
                write!(
                    f,
                    "Expected {} {}, got {}",
                    expected,
                    if *expected == 1 {
                        "argument"
                    } else {
                        "arguments"
                    },
                    actual,
                )
            }
            Self::CommandNotFound => write!(f, "Command not found"),
            Self::WriteError(we) => write!(f, "Write error: {:?}", we),
            Self::CommandError(Some(err)) => write!(f, "Command error: {}", err),
            Self::CommandError(None) => write!(f, "Command error"),
            Self::InvalidArgument => write!(f, "Invalid argument"),
        }
    }
}

pub type MenuResult = Result<(), MenuError>;

use std::fmt;

#[derive(Debug)]
pub enum CommandError {
    NoCommandProvided,
    InvalidCommand,
    MissingKey,
    MissingValue,
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandError::NoCommandProvided => write!(f, "NoCommandProvided"),
            CommandError::InvalidCommand => write!(f, "InvalidCommand"),
            CommandError::MissingKey => write!(f, "MissingKey"),
            CommandError::MissingValue => write!(f, "MissingValue"),
        }
    }
}

/*
    The source method, when implemented for a type that implements the std::error::Error trait,
    provides a way to access the "cause" or "source" of an error. This is especially useful in scenarios
    where errors can be wrapped or chained, allowing for a more detailed context or hierarchy of errors.
*/
impl std::error::Error for CommandError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

#[derive(Debug)]
pub enum Command {
    SET(String, String),
    GET(String),
    // TODO: delete
}

impl Command {
    pub fn from_args(args: Vec<String>) -> Result<Command, CommandError> {
        let command = args.get(1).ok_or(CommandError::NoCommandProvided)?.as_str();

        match command {
            "GET" => {
                let key = args.get(2).ok_or(CommandError::MissingKey)?;

                Ok(Command::GET(key.to_owned()))
            }
            "SET" => {
                let key = args.get(2).ok_or(CommandError::MissingKey)?;
                let value = args.get(3).ok_or(CommandError::MissingValue)?;

                Ok(Command::SET(key.to_owned(), value.to_owned()))
            }
            _ => Err(CommandError::InvalidCommand),
        }
    }
}

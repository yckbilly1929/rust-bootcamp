use crate::{RespArray, RespFrame};

use super::{extract_args, validate_command, CommandError, CommandExecutor, Echo};

impl CommandExecutor for Echo {
    fn execute(self, _backend: &crate::Backend) -> RespFrame {
        RespFrame::BulkString(self.message.into())
    }
}

impl TryFrom<RespArray> for Echo {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["echo"], 1)?;

        let mut args = extract_args(value, 1)?.into_iter();
        match args.next() {
            Some(RespFrame::BulkString(msg)) => Ok(Echo {
                message: String::from_utf8(msg.0)?,
            }),
            _ => Err(CommandError::InvalidArgument("".to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::{Ok, Result};

    use crate::{
        cmd::{CommandExecutor as _, Echo},
        Backend, RespFrame,
    };

    #[test]
    fn test_echo_command() -> Result<()> {
        let backend = Backend::new();
        let cmd = Echo {
            message: "hello".to_string(),
        };
        let result = cmd.execute(&backend);
        assert_eq!(result, RespFrame::BulkString(b"hello".into()));

        Ok(())
    }
}

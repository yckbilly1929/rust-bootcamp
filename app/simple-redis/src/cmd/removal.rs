use crate::{RespArray, RespFrame};

use super::{extract_args, validate_command, CommandError, CommandExecutor, Del};

impl CommandExecutor for Del {
    fn execute(self, backend: &crate::Backend) -> RespFrame {
        let keys: Vec<&str> = self.keys.iter().map(<_>::as_ref).collect();
        let removed_keys = backend.del(&keys);
        RespFrame::Integer(removed_keys.len() as i64)
    }
}

impl TryFrom<RespArray> for Del {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        let n_args = value.len() - 1;
        validate_command(&value, &["del"], n_args)?;

        let args = extract_args(value, 1)?.into_iter();
        let mut keys = Vec::new();
        for v in args {
            match v {
                RespFrame::BulkString(key) => {
                    keys.push(String::from_utf8(key.0)?);
                }
                _ => {
                    return Err(CommandError::InvalidArgument(
                        "Invalid key or value".to_string(),
                    ))
                }
            }
        }
        Ok(Del { keys })
    }
}

#[cfg(test)]
mod tests {
    use anyhow::{Ok, Result};

    use crate::{
        cmd::{CommandExecutor as _, Del, HSet, Set},
        Backend, RespFrame,
    };

    #[test]
    fn test_set_del_command() -> Result<()> {
        let backend = Backend::new();
        let cmd = Set {
            key: "a".to_string(),
            value: RespFrame::BulkString(b"1".into()),
        };
        cmd.execute(&backend);
        let cmd2 = Set {
            key: "b".to_string(),
            value: RespFrame::BulkString(b"2".into()),
        };
        cmd2.execute(&backend);
        let cmd3 = HSet {
            key: "c".to_string(),
            field: "k".to_string(),
            value: RespFrame::BulkString(b"v".into()),
        };
        cmd3.execute(&backend);

        let cmd = Del {
            keys: vec!["a".to_string(), "b".to_string(), "c".to_string()],
        };
        let resp = cmd.execute(&backend);

        assert_eq!(resp, RespFrame::Integer(3));

        Ok(())
    }
}

pub mod article;
pub mod attachment;
pub mod board;
pub mod comment;
pub mod config;
pub mod group;
pub mod login;
pub mod logout;
pub mod open;
pub mod open_target;
pub mod project;
pub mod search;
pub mod sprint;
pub mod tag;
pub mod ticket;
pub mod url;
pub mod visibility;
pub mod whoami;

use crate::error::YtdError;
use std::io::{self, BufRead, IsTerminal, Write};

pub fn confirm_delete(entity_type: &str, id: &str, assume_yes: bool) -> Result<bool, YtdError> {
    if assume_yes {
        return Ok(true);
    }

    if !io::stdin().is_terminal() {
        return Err(YtdError::Input(format!(
            "Refusing to delete {entity_type} {id} without confirmation. Pass -y to confirm."
        )));
    }

    eprint!("Delete {entity_type} {id}? Type 'yes' to confirm: ");
    io::stderr().flush()?;
    let mut line = String::new();
    io::stdin().lock().read_line(&mut line)?;
    Ok(line.trim() == "yes")
}

#[cfg(test)]
mod tests {
    use super::confirm_delete;

    #[test]
    fn confirm_delete_accepts_assume_yes() {
        assert!(confirm_delete("ticket", "DWP-1", true).unwrap());
    }

    #[test]
    fn confirm_delete_rejects_non_tty_without_assume_yes() {
        let err = confirm_delete("ticket", "DWP-1", false).unwrap_err();
        assert_eq!(
            err.to_string(),
            "Refusing to delete ticket DWP-1 without confirmation. Pass -y to confirm."
        );
    }
}

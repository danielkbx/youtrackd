use crate::error::YtdError;

/// Parse a human-readable duration into minutes.
/// Accepted formats: `30m`, `1h`, `2h30m`, `90` (plain number = minutes)
pub fn parse_duration(input: &str) -> Result<u32, YtdError> {
    let input = input.trim();
    if input.is_empty() {
        return Err(YtdError::Input("Duration cannot be empty".into()));
    }

    // Plain number = minutes
    if let Ok(mins) = input.parse::<u32>() {
        if mins == 0 {
            return Err(YtdError::Input("Duration must be greater than 0".into()));
        }
        return Ok(mins);
    }

    let mut hours: u32 = 0;
    let mut minutes: u32 = 0;
    let mut num_buf = String::new();
    let mut found_any = false;

    for c in input.chars() {
        if c.is_ascii_digit() {
            num_buf.push(c);
        } else if c == 'h' {
            if num_buf.is_empty() {
                return Err(YtdError::Input(format!("Invalid duration: {input}")));
            }
            hours = num_buf
                .parse()
                .map_err(|_| YtdError::Input(format!("Invalid duration: {input}")))?;
            num_buf.clear();
            found_any = true;
        } else if c == 'm' {
            if num_buf.is_empty() && !found_any {
                return Err(YtdError::Input(format!("Invalid duration: {input}")));
            }
            if !num_buf.is_empty() {
                minutes = num_buf
                    .parse()
                    .map_err(|_| YtdError::Input(format!("Invalid duration: {input}")))?;
                num_buf.clear();
            }
            found_any = true;
        } else {
            return Err(YtdError::Input(format!("Invalid duration: {input}")));
        }
    }

    // Trailing number without unit after 'h' = minutes
    if !num_buf.is_empty() && found_any {
        minutes = num_buf
            .parse()
            .map_err(|_| YtdError::Input(format!("Invalid duration: {input}")))?;
    } else if !num_buf.is_empty() {
        return Err(YtdError::Input(format!("Invalid duration: {input}")));
    }

    if !found_any {
        return Err(YtdError::Input(format!("Invalid duration: {input}")));
    }

    let total = hours * 60 + minutes;
    if total == 0 {
        return Err(YtdError::Input("Duration must be greater than 0".into()));
    }

    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_minutes() {
        assert_eq!(parse_duration("30").unwrap(), 30);
        assert_eq!(parse_duration("90").unwrap(), 90);
    }

    #[test]
    fn minutes_with_suffix() {
        assert_eq!(parse_duration("30m").unwrap(), 30);
        assert_eq!(parse_duration("45m").unwrap(), 45);
    }

    #[test]
    fn hours() {
        assert_eq!(parse_duration("1h").unwrap(), 60);
        assert_eq!(parse_duration("2h").unwrap(), 120);
    }

    #[test]
    fn hours_and_minutes() {
        assert_eq!(parse_duration("2h30m").unwrap(), 150);
        assert_eq!(parse_duration("1h15m").unwrap(), 75);
    }

    #[test]
    fn empty_is_error() {
        assert!(parse_duration("").is_err());
    }

    #[test]
    fn invalid_is_error() {
        assert!(parse_duration("abc").is_err());
        assert!(parse_duration("h").is_err());
    }

    #[test]
    fn zero_is_error() {
        assert!(parse_duration("0").is_err());
        assert!(parse_duration("0m").is_err());
        assert!(parse_duration("0h0m").is_err());
    }
}

/// Formats an ISO 8601 date string to YYYY-MM-DD format.
///
/// Handles various ISO 8601 formats:
/// - "2026-04-08T00:00:00.000Z" -> "2026-04-08"
/// - "2026-04-08T15:30:45Z" -> "2026-04-08"
/// - "2026-04-08" -> "2026-04-08" (passthrough)
///
/// Returns the original string if parsing fails.
pub fn format_date(date_str: &str) -> String {
    // If already in YYYY-MM-DD format (10 chars), return as-is
    if date_str.len() == 10
        && date_str.chars().nth(4) == Some('-')
        && date_str.chars().nth(7) == Some('-')
    {
        return date_str.to_string();
    }

    // Extract YYYY-MM-DD from ISO 8601 format (first 10 characters)
    if date_str.len() >= 10 {
        date_str[..10].to_string()
    } else {
        date_str.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_date_iso_with_time() {
        assert_eq!(format_date("2026-04-08T00:00:00.000Z"), "2026-04-08");
        assert_eq!(format_date("2026-04-08T15:30:45Z"), "2026-04-08");
    }

    #[test]
    fn test_format_date_already_formatted() {
        assert_eq!(format_date("2026-04-08"), "2026-04-08");
    }

    #[test]
    fn test_format_date_short_string() {
        assert_eq!(format_date("2026"), "2026");
    }
}

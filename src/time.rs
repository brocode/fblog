use chrono::{LocalResult, TimeZone, Utc};

fn timestamp_to_iso8601(timestamp: i64) -> Option<String> {
    let now = Utc::now();

    let dt_secs = Utc.timestamp_opt(timestamp, 0);
    let dt_millis = Utc.timestamp_millis_opt(timestamp);

    match (dt_secs, dt_millis) {
        (LocalResult::Single(dt_secs), LocalResult::Single(dt_millis)) => {
            let diff_secs = (now - dt_secs).num_milliseconds().abs();
            let diff_millis = (now - dt_millis).num_milliseconds().abs();

            if diff_secs < diff_millis {
                Some(dt_secs.to_rfc3339_opts(chrono::SecondsFormat::Millis, true))
            } else {
                Some(dt_millis.to_rfc3339_opts(chrono::SecondsFormat::Millis, true))
            }
        }
        (LocalResult::Single(dt_secs), _) => Some(dt_secs.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)),
        (_, LocalResult::Single(dt_millis)) => Some(dt_millis.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)),
        _ => None, // Neither is valid (or ambiguous due to time zone transitions)
    }
}

pub fn try_convert_timestamp_to_readable(input: String) -> String {
    if input.is_empty() {
        return input;
    }

    if let Some(parsed) = input.parse::<i64>().ok().and_then(timestamp_to_iso8601) {
        return parsed;
    }

    input
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;

    use super::*;

    #[test]
    fn test_valid_second_timestamp() {
        let iso = "2024-05-21T11:21:29.000Z";
        let timestamp = DateTime::parse_from_rfc3339(iso).expect("should be valid").timestamp();
        let result = timestamp_to_iso8601(timestamp).unwrap();
        assert_eq!(iso, result)
    }

    #[test]
    fn test_valid_millis_timestamp() {
        let iso = "2024-05-21T11:21:29.536Z";
        let timestamp = DateTime::parse_from_rfc3339(iso).expect("should be valid").timestamp_millis();
        let result = timestamp_to_iso8601(timestamp).unwrap();
        assert_eq!(iso, result)
    }

    #[test]
    fn test_try_convert() {
        assert_eq!(try_convert_timestamp_to_readable("1716292213381".to_string()), "2024-05-21T11:50:13.381Z");
        assert_eq!(try_convert_timestamp_to_readable("1716292213".to_string()), "2024-05-21T11:50:13.000Z");
        assert_eq!(try_convert_timestamp_to_readable("bla".to_string()), "bla");
        assert_eq!(try_convert_timestamp_to_readable("1234bla".to_string()), "1234bla");
        assert_eq!(try_convert_timestamp_to_readable("".to_string()), "");
    }
}

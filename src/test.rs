

#[cfg(test)]
mod tests {
    use std::fs::{read_to_string, File};
    use std::io::{BufRead, BufReader};
    use std::str::FromStr;
    use regex::Regex;
    use chrono::{NaiveDateTime, DateTime, TimeZone, Utc, ParseError};

    #[derive(Debug, PartialEq)]
    struct LogEntry {
        timestamp: DateTime<Utc>,
        severity: String,
        logger: String,
        message: String,
    }
    impl LogEntry {
        fn parse_log_entry(entry: &str) -> Option<Self> {
            let re = Regex::new(r"^(?P<timestamp>[^ ]+ [^ ]+)(\s+)(?P<severity>[A-Z]+)(\s+)(?P<logger>[^ ]+)(\s+)-(\s+)(?P<message>.*)").unwrap();

            if let Some(captures) = re.captures(entry) {
                let timestamp_str = captures.name("timestamp").unwrap().as_str();
                let timestamp = DateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S%,3f")
                    .expect("Failed to parse timestamp string")
                    .with_timezone(&Utc);
                let severity = captures.name("severity").unwrap().as_str().to_string();
                let logger = captures.name("logger").unwrap().as_str().to_string();
                let message = captures.name("message").unwrap().as_str().to_string();
                Some(LogEntry {
                    timestamp,
                    severity,
                    logger,
                    message,
                })
            } else {
                None
            }
        }
    }
    impl FromStr for LogEntry {
        type Err = ();

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match LogEntry::parse_log_entry(s) {
                Some(log_entry) => Ok(log_entry),
                None => Err(()),
            }
        }
    }

    #[test]
    fn parse_log_from_file_and_return_majority_element() {
        let mut logs: &str = "2022-03-16 01:25:11,194 DEBUG c.a.d.i.j.a.activities.DriveActivity - Change state from none to started.";
        let expected_entry = LogEntry {
            timestamp:  DateTime::parse_from_str("2022-03-16 01:25:11", "%Y-%m-%d %H:%M:%S").expect("Failed to parse timestamp string")
                .with_timezone(&Utc),
            severity: "DEBUG".to_string(),
            logger: "c.a.d.i.j.a.activities.DriveActivity".to_string(),
            message: "Change state from none to started.".to_string(),
        };
        assert_eq!(parse_log(logs),Ok(expected_entry));
    }
    use super::*;

    fn parse_timestamp(timestamp_str: &str) -> Result<DateTime<Utc>, ParseError> {
        let parsed_datetime = NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S,%f");
        match parsed_datetime {
            Ok(parsed_datetime) => Ok(Utc.from_utc_datetime(&parsed_datetime)),
            Err(err) => Err(err),
        }
    }

    #[test]
    fn test_parse_date_timestamp() {
        let timestamp_str = "2015-09-05 23:56:04,233";
        let parsed_datetime = NaiveDateTime::parse_from_str("2015-09-05 23:56:04,233", "%Y-%m-%d %H:%M:%S,%f");

        match parsed_datetime {
            Ok(parsed_datetime) => {
                println!("Parsed datetime: {:?}", parsed_datetime);
                let expected_timestamp = Utc
                    .from_utc_datetime(&parsed_datetime);
                assert_eq!(parse_timestamp(timestamp_str), Ok(expected_timestamp));
            }
            Err(err) => {
                println!("Failed to parse timestamp string: {}", err);
                assert!(false);
            }
        }
    }
    #[test]
    fn parse_ishelves_log_file() {
        // Open the ishelves.log file
        let file = File::open("/home/jcgouleau/Bureau/Dior/ishelves/ishelves.log")
            .expect("Failed to open file");
        let reader = BufReader::new(file);



        let mut log_entries = Vec::new();
        for line_result in reader.lines() {
            match line_result {
                Ok(line) => {
                    if let Ok(log_entry) = LogEntry::from_str(&line) {
                        log_entries.push(log_entry);
                    } else {
                        if let Some(mut last_value) = log_entries.pop() {
                            // Modify the last value (e.g., add 10 to it)
                            let new_last_value = last_value.message + line.as_str();
                            last_value.message = new_last_value;
                            // Push the modified value back onto the vector
                            log_entries.push(last_value);
                        } else {
                            println!("Vector is empty");
                        }
                        println!("Failed to parse log entry: {:?}", line);
                    }
                }
                Err(err) => {
                    eprintln!("Error reading line: {}", err);
                }
            }
        }

        // Print the actual number of parsed LogEntry objects

        // Assert that the length of the vector equals 3196
        assert_eq!(log_entries.len(), 3196);
    }

    fn parse_log(p0: &str) -> Result<LogEntry, ()> {
        LogEntry::from_str(p0)
    }
}

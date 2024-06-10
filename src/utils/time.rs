use std::time::{SystemTime, UNIX_EPOCH};
fn is_leap_year(year: u32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}
fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        1 => 31,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        3 => 31,
        4 => 30,
        5 => 31,
        6 => 30,
        7 => 31,
        8 => 31,
        9 => 30,
        10 => 31,
        11 => 30,
        12 => 31,
        _ => 30, // should not occur
    }
}
/// Get the current utc time
pub(crate) fn now_utc() -> String {
    let now = SystemTime::now();
    let duration_since_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");

    // Get the total number of seconds and milliseconds since the epoch
    let total_seconds = duration_since_epoch.as_secs();
    let millis = duration_since_epoch.subsec_millis();

    // Convert the total number of seconds to the current date and time
    let mut remaining_seconds = total_seconds;
    let mut year = 1970;
    while remaining_seconds
        >= if is_leap_year(year) {
            366 * 86400
        } else {
            365 * 86400
        }
    {
        remaining_seconds -= if is_leap_year(year) {
            366 * 86400
        } else {
            365 * 86400
        };
        year += 1;
    }

    let mut month = 1;
    while remaining_seconds >= (days_in_month(year, month) * 86400) as u64 {
        remaining_seconds -= (days_in_month(year, month) * 86400) as u64;
        month += 1;
    }

    let day = (remaining_seconds / 86400) as u32 + 1;
    remaining_seconds %= 86400;
    let hour = (remaining_seconds / 3600) as u32;
    let minute = ((remaining_seconds % 3600) / 60) as u32;
    let second = (remaining_seconds % 60) as u32;

    (year, month, day, hour, minute, second, millis);

    format!(
        "{}-{}-{} {}:{}:{}.{}",
        year, month, day, hour, minute, second, millis
    )
}

#[cfg(test)]
mod time_test {
    use super::*;
    #[test]
    fn test_leap_years() {
        assert!(is_leap_year(2000));
        assert!(!is_leap_year(1900));
        assert!(is_leap_year(2004));
        assert!(!is_leap_year(2001));
    }
    #[test]
    fn test_days_in_month() {
        assert_eq!(days_in_month(2021, 1), 31);
        assert_eq!(days_in_month(2021, 2), 28);
        assert_eq!(days_in_month(2020, 2), 29); // leap year
        assert_eq!(days_in_month(2021, 4), 30);
        assert_eq!(days_in_month(2021, 12), 31);
    }
}

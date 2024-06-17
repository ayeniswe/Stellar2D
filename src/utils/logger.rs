use std::io::{stdout, Write};

use super::time;
/// Logger threshold levels
///
/// Error - 1
///
/// Warn - 2
///
/// Info - 3
///
/// The `threshold` will include all levels less than or equal to
/// the `threshold`
#[derive(Debug, Default)]
pub struct Logger<T: Write> {
    output: T,
    threshold: usize,
}
impl<T: Write> Logger<T> {
    pub fn new(output: T, threshold: usize) -> Self {
        Self { output, threshold }
    }
    /// Info log with a newline '/n'
    pub fn logln(&mut self, msg: &str) {
        if self.threshold == 3 {
            match writeln!(self.output, "[INFO] {}: {}", time::now_utc(), msg) {
                Err(x) => eprintln!("{}", x),
                _ => (),
            }
        }
    }
    pub fn log(&mut self, msg: &str) {
        if self.threshold == 3 {
            match write!(self.output, "[INFO] {}: {}", time::now_utc(), msg) {
                Err(x) => eprintln!("{}", x),
                _ => (),
            }
        }
    }
    /// Warning log with a newline '/n'
    pub fn wlogln(&mut self, msg: &str) {
        if self.threshold >= 2 {
            match writeln!(self.output, "[WARNING] {}: {}", time::now_utc(), msg) {
                Err(x) => eprintln!("{}", x),
                _ => (),
            }
        }
    }
    pub fn wlog(&mut self, msg: &str) {
        if self.threshold >= 2 {
            match write!(self.output, "[WARNING] {}: {}", time::now_utc(), msg) {
                Err(x) => eprintln!("{}", x),
                _ => (),
            }
        }
    }
    /// Error log with a newline '/n'
    pub fn elogln(&mut self, msg: &str) {
        if self.threshold >= 1 {
            match writeln!(self.output, "[ERROR] {}: {}", time::now_utc(), msg) {
                Err(x) => eprintln!("{}", x),
                _ => (),
            }
        }
    }
    pub fn elog(&mut self, msg: &str) {
        if self.threshold >= 1 {
            match write!(self.output, "[ERROR] {}: {}", time::now_utc(), msg) {
                Err(x) => eprintln!("{}", x),
                _ => (),
            }
        }
    }
}

#[cfg(test)]
mod logger_log_test {
    use super::*;
    use regex::Regex;
    #[test]
    fn test_log() {
        let mut buffer = Vec::new();
        let mut logger = Logger::new(&mut buffer, 3);
        logger.log("Test message");
        let timestamp = Regex::new(
            r"^\[INFO\] \d{4}-\d{1,2}-\d{1,2} \d{1,2}:\d{1,2}:\d{1,2}.\d{1,3}: Test message$",
        )
        .unwrap();
        let log = String::from_utf8(buffer).unwrap();

        assert!(timestamp.is_match(&log));
    }
    #[test]
    fn test_info_log() {
        let mut buffer = Vec::new();
        let mut logger = Logger::new(&mut buffer, 3);
        logger.log("Test message");

        assert!(String::from_utf8(buffer).unwrap().starts_with("[INFO]"))
    }
    #[test]
    fn test_warn_log() {
        let mut buffer = Vec::new();
        let mut logger = Logger::new(&mut buffer, 2);
        logger.wlog("Test message");

        assert!(String::from_utf8(buffer).unwrap().starts_with("[WARNING]"))
    }
    #[test]
    fn test_error_log() {
        let mut buffer = Vec::new();
        let mut logger = Logger::new(&mut buffer, 1);
        logger.elog("Test message");

        assert!(String::from_utf8(buffer).unwrap().starts_with("[ERROR]"))
    }
}

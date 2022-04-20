//! Logger for Azure Function Custom Handler,
//! abusing the undocumented (at least I don't know where) rule of Azure Function
//! "infering" the log level from stderr.
//! 
//! For custom handler, if you print a message to stdout, it will be considered as a `Information` 
//! level log by Azure Function runtime.
//! 
//! If you print a message to stderr, then it will be consider `Error` if it does not contain `warn` (case insensitive),
//! otherwise it will be `Warning`.
//! 
//! So the strategy is, for error-level log, we find the occurence of `warn` and replace `r` by `𝗋`(\U+1d5cb)
//! and `R` by `𝖱`(\U+1d5b1). For warning-level log, if `warn` does not occur, we add a `warning:` prefix.
//! 
//! You can initialize the log by [init_logger]. If you want to use the replacement logic
//! in other logger, you can call [to_error_log] to get the replaced error log, and [contains_warn]
//! to test whether the message contains `warn` (case insensitive).
use core::fmt;

const WARN: [char; 4] = ['w', 'a', 'r', 'n'];
struct ErrorLogBuilder {
    log: String,
    warn: [char; 4],
    warn_ptr: u32,
}

impl fmt::Write for ErrorLogBuilder {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.log.reserve(s.len());
        for ch in s.chars() {
            if ch.eq_ignore_ascii_case(&WARN[self.warn_ptr as usize]) {
                self.warn[self.warn_ptr as usize] = ch;

                if self.warn_ptr == 3 {
                    self.log.push(self.warn[0]);
                    self.log.push(self.warn[1]);
                    if self.warn[2] == 'r' {
                        self.log.push('\u{1d5cb}');
                    } else {
                        self.log.push('\u{1d5b1}');
                    }
                    self.log.push(self.warn[3]);
                    self.warn_ptr = 0;
                } else {
                    self.warn_ptr += 1;
                }
            } else {
                for i in 0 .. self.warn_ptr {
                    self.log.push(self.warn[i as usize]);
                }
                self.warn_ptr = 0;
                self.log.push(ch);
            }
        }
        Ok(())
    }
}

impl fmt::Display for ErrorLogBuilder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.log)?;
        for i in 0..self.warn_ptr {
            write!(f, "{}", self.warn[i as usize])?;
        }
        Ok(())
    }
}

impl ErrorLogBuilder {
    fn build(mut self) -> String {
        for i in 0..self.warn_ptr {
            self.log.push(self.warn[i as usize])
        }
        self.log
    }
}

/// Returns a string that, for each occurance of `warn` (case insensitive),
/// `r` is replaced by `𝗋`(\U+1d5cb) and `R` is replaced by `𝖱`(\U+1d5b1).
pub fn to_error_log(args: fmt::Arguments) -> String {
    let mut builder = ErrorLogBuilder {
        log: String::new(),
        warn: ['\0'; 4],
        warn_ptr: 0,
    };
    let _ = fmt::Write::write_fmt(&mut builder, args);
    builder.build()
}

/// Returns true if the message contains `warn` (case insensitive).
pub fn contains_warn(s: &str) -> bool {
    let mut warn_ptr = 0;
    for ch in s.chars() {
        if ch.eq_ignore_ascii_case(&WARN[warn_ptr]) {
            if warn_ptr == 3 {
                return true
            }
            warn_ptr += 1;
        }
    }
    false
}

struct Logger;
impl log::Log for Logger {
    fn enabled(&self, m: &log::Metadata) -> bool {
        m.level() <= log::Level::Info
    }

    fn log(&self, record: &log::Record) {
        match record.level() {
            log::Level::Error => eprintln!("{}", to_error_log(*record.args())),
            log::Level::Warn => if contains_warn(record.args().to_string().as_str()) {
                eprintln!("{}", record.args());
            } else {
                eprintln!("warning: {}", record.args());
            },
            log::Level::Info => println!("{}", record.args()),
            _ => {}
        }
        
    }

    fn flush(&self) {}
}
const LOGGER: Logger = Logger;

pub fn init_logger() -> Result<(), log::SetLoggerError>{
    log::set_logger(&LOGGER)?;
    log::set_max_level(log::LevelFilter::Info);
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{contains_warn, to_error_log};

    #[test]
    fn test_no_change_without_warn() {
        assert_eq!("abcdefg", to_error_log(format_args!("abcdefg")));
    }
    #[test]
    fn suffix_incomplete_warn() {
        assert_eq!("abcdwar", to_error_log(format_args!("abcdwar")));
    }
    #[test]
    fn simple_replace() {
        assert_eq!("awa\u{1d5cb}na", to_error_log(format_args!("awarna")));
    }
    #[test]
    fn capacity_preserve() {
        assert_eq!("aWa\u{1d5cb}Na", to_error_log(format_args!("aWarNa")));
    }
    #[test]
    fn suffix_replace() {
        assert_eq!("aWA\u{1d5cb}n", to_error_log(format_args!("aWArn")));
    }
    #[test]
    fn capacity_replace() {
        assert_eq!("awa\u{1d5b1}Na", to_error_log(format_args!("awaRNa")));
    }

    #[test]
    fn test_no_warn() {
        assert!(!contains_warn("abcdefg"));
    }
    #[test]
    fn has_warn() {
        assert!(contains_warn("awarng"));
    }
    #[test]
    fn differnt_case_warn() {
        assert!(contains_warn("awARng"));
    }
    #[test]
    fn suffix_warn() {
        assert!(contains_warn("awARn"));
    }
}


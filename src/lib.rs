use core::fmt;

const WARN: [char; 4] = ['w', 'a', 'r', 'n'];
pub struct ErrorLog {
    log: String,
    warn: [char; 4],
    warn_ptr: u32,
}
impl ErrorLog {
    pub fn from_args(args: fmt::Arguments) -> Self {
        let mut builder = ErrorLog {
            log: String::new(),
            warn: ['\0'; 4],
            warn_ptr: 0,
        };
        let _ = fmt::Write::write_fmt(&mut builder, args);
        builder
    }
}
impl fmt::Write for ErrorLog {
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

impl fmt::Display for ErrorLog {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.log)?;
        for i in 0..self.warn_ptr {
            write!(f, "{}", self.warn[i as usize])?;
        }
        Ok(())
    }
}

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
            log::Level::Error => eprintln!("{}", ErrorLog::from_args(*record.args())),
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
    use crate::{ErrorLog, contains_warn};

    #[test]
    fn test_no_change_without_warn() {
        assert_eq!("abcdefg", format!("{}", ErrorLog::from_args(format_args!("abcdefg"))));
    }
    #[test]
    fn suffix_incomplete_warn() {
        assert_eq!("abcdwar", format!("{}", ErrorLog::from_args(format_args!("abcdwar"))));
    }
    #[test]
    fn simple_replace() {
        assert_eq!("awa\u{1d5cb}na", format!("{}", ErrorLog::from_args(format_args!("awarna"))));
    }
    #[test]
    fn capacity_preserve() {
        assert_eq!("aWa\u{1d5cb}Na", format!("{}", ErrorLog::from_args(format_args!("aWarNa"))));
    }
    #[test]
    fn suffix_replace() {
        assert_eq!("aWA\u{1d5cb}n", format!("{}", ErrorLog::from_args(format_args!("aWArn"))));
    }
    #[test]
    fn capacity_replace() {
        assert_eq!("awa\u{1d5b1}Na", format!("{}", ErrorLog::from_args(format_args!("awaRNa"))));
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


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
//! So the strategy is, for error-level log, if `warn` occurs, base64-encode it, if the encoded string still contains `warn`,
//! base-encode again, and if the twice-encoded string still contains `warn` (which should be impossible), log an error explain that the
//! following warning is error, then log it as a warning. For warning-level log, if `warn` does not occur, add a `warning:` prefix.
//! 
//! You can initialize the log by [init]. You can also implement your own transform logic by implementing
//! [Transform] trait and passing it to [init_transform].
const WARN: [char; 4] = ['w', 'a', 'r', 'n'];

pub trait Transform {
    /// Transform the error log message that contains `warn` (case insensitive).
    fn transform_error(&self, msg: String) -> String;
    /// Transform the warning log message that does not contain `warn` (case insensitive).
    fn transform_warning(&self, msg: String) -> String;
}
struct Logger<T>(T);
impl<T: Transform + Send + Sync> log::Log for Logger<T> {
    fn enabled(&self, m: &log::Metadata) -> bool {
        m.level() <= log::Level::Info
    }

    fn log(&self, record: &log::Record) {
        match record.level() {
            log::Level::Error => {
                let mut log = record.args().to_string();
                if contains_warn(&log) {
                    log = self.0.transform_error(log);
                }

                eprintln!("{}", log);
            },
            log::Level::Warn => {
                let mut log = record.args().to_string();
                if !contains_warn(&log) {
                    log = self.0.transform_warning(log);
                }

                eprintln!("{}", log);
            }
            log::Level::Info => println!("{}", record.args()),
            _ => {}
        }
        
    }

    fn flush(&self) {}
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
        } else {
            warn_ptr = 0;
        }
    }
    false
}

pub struct DefaultTransform;
impl Transform for DefaultTransform {
    fn transform_error(&self, msg: String) -> String {
        let mut transformed = base64::encode(&msg);

        if !contains_warn(&transformed) {
            "base64-encoded log: ".to_string() + &transformed
        } else {
            transformed = base64::encode(transformed);
            if !contains_warn(&transformed) {
                "base64-encoded-twice log: ".to_string() + &transformed
            } else {
                // Should be impossible.
                "The following error log has to be logged as Warning: \n".to_string() + &msg
            }
        }
    }

    fn transform_warning(&self, msg: String) -> String {
        "warning: ".to_string() + &msg
    }
}

pub fn init() {
    init_transform(DefaultTransform);
}

pub fn init_transform<T: Transform + 'static + Send + Sync>(transform: T) {
    log::set_logger(Box::leak(Box::new(Logger(transform))))
        .expect("Failed to initialize logger");
    log::set_max_level(log::LevelFilter::Info);
}


#[cfg(test)]
mod tests {
    use crate::contains_warn;

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
    #[test]
    fn split_warn() {
        assert!(!contains_warn("wa#rn"));
    }
}


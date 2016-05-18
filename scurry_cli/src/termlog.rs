use log::{self, LogRecord, LogLevel, LogMetadata, SetLoggerError, LogLevelFilter};

struct TermLogger {}

impl log::Log for TermLogger {
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() <= LogLevel::Info
    }

    fn log(&self, record: &LogRecord) {
        if self.enabled(record.metadata()) {
            if record.level() < LogLevel::Info {
                println!("{} - {}", record.level(), record.args());
            } else {
                println!("{}", record.args());
            }
        }
    }
}

pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(|max_log_level| {
        max_log_level.set(LogLevelFilter::Info);
        Box::new(TermLogger{})
    })
}

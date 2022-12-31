use log::{Log, Metadata, Record, Level};
use log::Level::{Debug, Warn};

pub struct MalLogger;

impl Log for MalLogger{
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Warn
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()){
            println!("{}", record.args())
        }
    }

    fn flush(&self) {
    }
}

use log::{SetLoggerError, LevelFilter};

static LOGGER: MalLogger = MalLogger;

pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Warn))
}
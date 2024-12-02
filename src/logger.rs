use std::{
    io::{Stdout, StdoutLock},
    path::{Path, PathBuf},
};

use chrono::Local;
use tracing::Level;
use tracing_subscriber::fmt::writer::MakeWriter;

#[derive(Debug)]
pub struct Logger {
    stdout: Stdout,
    trace_path: PathBuf,
    error_path: PathBuf,
}

impl<'a> MakeWriter<'a> for Logger {
    type Writer = LogWriter<'a>;

    /// # Panics
    ///
    /// The function panics when it fails to open the file at
    /// `self.trace_path`.
    fn make_writer(&'a self) -> Self::Writer {
        self.trace_writer().unwrap()
    }

    /// # Panics
    ///
    /// The function panics when it fails to open the file either at
    /// `self.trace_path` or `self.error_path`.
    fn make_writer_for(&'a self, meta: &tracing::Metadata<'_>) -> Self::Writer {
        match meta.level() {
            &Level::ERROR => self.error_writer().unwrap(),
            _others => self.make_writer(),
        }
    }
}

impl Logger {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, LoggerError> {
        let trace_path = path.as_ref().to_owned();
        std::fs::create_dir_all(&trace_path).map_err(LoggerError::CreateDirectory)?;

        let error_path = path.as_ref().join("error").to_owned();
        std::fs::create_dir_all(&error_path).map_err(LoggerError::CreateDirectory)?;

        Ok(Self {
            stdout: std::io::stdout(),
            trace_path,
            error_path,
        })
    }

    pub fn init(self) {
        tracing_subscriber::fmt().with_writer(self).init();
        std::panic::set_hook(Box::new(|panic_info| tracing::error!("{:?}", panic_info)));
    }

    fn today() -> String {
        Local::now().date_naive().to_string()
    }

    fn open_file(path: impl AsRef<Path>) -> Result<std::fs::File, LoggerError> {
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(LoggerError::OpenFile)
    }

    fn trace_writer<'a>(&'a self) -> Result<LogWriter<'a>, LoggerError> {
        let trace_path = self.trace_path.join(Self::today());

        Ok(LogWriter::new(
            self.stdout.lock(),
            Self::open_file(trace_path)?,
            None,
        ))
    }

    fn error_writer<'a>(&'a self) -> Result<LogWriter<'a>, LoggerError> {
        let trace_path = self.trace_path.join(Self::today());
        let error_path = self.error_path.join(Self::today());

        Ok(LogWriter::new(
            self.stdout.lock(),
            Self::open_file(trace_path)?,
            Some(Self::open_file(error_path)?),
        ))
    }
}

pub struct LogWriter<'a> {
    stdout: StdoutLock<'a>,
    trace_file: std::fs::File,
    error_file: Option<std::fs::File>,
}

impl<'a> std::io::Write for LogWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.stdout.write(buf)?;
        if let Some(error_file) = &mut self.error_file {
            error_file.write(buf)?;
        }

        self.trace_file.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.stdout.flush()?;
        if let Some(error_file) = &mut self.error_file {
            error_file.flush()?;
        }
        self.trace_file.flush()?;

        Ok(())
    }
}

impl<'a> LogWriter<'a> {
    pub fn new(
        stdout: StdoutLock<'a>,
        trace_file: std::fs::File,
        error_file: Option<std::fs::File>,
    ) -> Self {
        Self {
            stdout,
            trace_file,
            error_file,
        }
    }
}

#[derive(Debug)]
pub enum LoggerError {
    CreateDirectory(std::io::Error),
    OpenFile(std::io::Error),
}

impl std::fmt::Display for LoggerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for LoggerError {}

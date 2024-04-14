// use jsonrpsee::types::error::{ErrorCode, ErrorObjectOwned};

// pub trait WrapError {
//     type Output;

//     fn wrap(self, caller: impl Into<Caller>) -> Self::Output;

//     fn wrap_context<C>(self, caller: impl Into<Caller>, context: C) -> Self::Output
//     where
//         C: std::fmt::Debug;
// }

// impl<T, E> WrapError for Result<T, E>
// where
//     E: std::error::Error + 'static,
// {
//     type Output = Result<T, Error>;

//     fn wrap(self, caller: impl Into<Caller>) -> Self::Output {
//         match self {
//             Ok(value) => Ok(value),
//             Err(error) => Err(Error::new(caller, error)),
//         }
//     }

//     fn wrap_context<C>(self, caller: impl Into<Caller>, context: C) -> Self::Output
//     where
//         C: std::fmt::Debug,
//     {
//         match self {
//             Ok(value) => Ok(value),
//             Err(error) => Err(Error::new_with_context(caller, context, error)),
//         }
//     }
// }

// impl<T> WrapError for Option<T> {
//     type Output = Result<T, Error>;

//     fn wrap(self, caller: impl Into<Caller>) -> Self::Output {
//         match self {
//             Some(value) => Ok(value),
//             None => Err(Error::new(caller, ErrorKind::NoneType)),
//         }
//     }

//     fn wrap_context<C>(self, caller: impl Into<Caller>, context: C) -> Self::Output
//     where
//         C: std::fmt::Debug,
//     {
//         match self {
//             Some(value) => Ok(value),
//             None => Err(Error::new_with_context(
//                 caller,
//                 context,
//                 ErrorKind::NoneType,
//             )),
//         }
//     }
// }

// pub struct Error {
//     caller: Caller,
//     context: Option<String>,
//     source: ErrorKind,
// }

// unsafe impl Send for Error {}

// impl std::fmt::Debug for Error {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", self)
//     }
// }

// impl std::fmt::Display for Error {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match &self.context {
//             Some(context) => {
//                 writeln!(f, "{} at {}", self.source, self.caller)?;
//                 write!(f, "{}", context)?;
//                 Ok(())
//             }
//             None => write!(f, "{} at {}", self.source, self.caller),
//         }
//     }
// }

// impl std::error::Error for Error {}

// impl Into<ErrorObjectOwned> for Error {
//     fn into(self) -> ErrorObjectOwned {
//         match self.context {
//             Some(context) => ErrorObjectOwned::owned::<String>(
//                 ErrorCode::InternalError.code(),
//                 format!("{} at {} ", self.source, self.caller),
//                 Some(context),
//             ),
//             None => ErrorObjectOwned::owned::<String>(
//                 ErrorCode::InternalError.code(),
//                 format!("{} at {}", self.source, self.caller),
//                 None,
//             ),
//         }
//     }
// }

// impl Error {
//     pub fn new<E>(caller: impl Into<Caller>, error: E) -> Self
//     where
//         E: std::error::Error + 'static,
//     {
//         Self {
//             caller: caller.into(),
//             context: None,
//             source: ErrorKind::Boxed(Box::new(error)),
//         }
//     }

//     pub fn new_with_context<C, E>(caller: impl Into<Caller>, context: C, error: E) -> Self
//     where
//         C: std::fmt::Debug,
//         E: std::error::Error + 'static,
//     {
//         Self {
//             caller: caller.into(),
//             context: Some(format!("{:?}", context)),
//             source: ErrorKind::Boxed(Box::new(error)),
//         }
//     }

//     pub fn str_error(caller: impl Into<Caller>, error: &'static str) -> Self {
//         Self {
//             caller: caller.into(),
//             context: None,
//             source: ErrorKind::StaticStr(error),
//         }
//     }

//     pub fn is_none_type(&self) -> bool {
//         match &self.source {
//             ErrorKind::NoneType => true,
//             _others => false,
//         }
//     }
// }

// pub struct Caller {
//     file: &'static str,
//     line: u32,
//     operation: &'static str,
// }

// impl std::fmt::Display for Caller {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}:{}:{}", self.file, self.line, self.operation)
//     }
// }

// impl From<(&'static str, u32, &'static str)> for Caller {
//     fn from(value: (&'static str, u32, &'static str)) -> Self {
//         Self {
//             file: value.0,
//             line: value.1,
//             operation: value.2,
//         }
//     }
// }

// pub enum ErrorKind {
//     Boxed(Box<dyn std::error::Error>),
//     StaticStr(&'static str),
//     NoneType,
// }

// impl std::fmt::Debug for ErrorKind {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", self)
//     }
// }

// impl std::fmt::Display for ErrorKind {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Self::Boxed(error) => write!(f, "{}", error),
//             Self::StaticStr(error) => write!(f, "{}", error),
//             Self::NoneType => write!(f, "NoneType Error"),
//         }
//     }
// }

// impl std::error::Error for ErrorKind {}

// #[test]
// fn works() {
//     fn outer() -> Result<(), Error> {
//         std::fs::read_to_string("no_file").wrap_context(crate::caller!(outer()), "no_file")?;
//         Ok(())
//     }

//     outer().unwrap();
// }

use std::{fs, panic::Location};

pub trait WrapError {
    type Output;

    fn wrap<C>(self, context: C) -> Self::Output
    where
        C: std::fmt::Debug;
}

impl<T, E> WrapError for Result<T, Box<E>>
where
    E: std::error::Error + 'static,
{
    type Output = Result<T, Error>;

    #[track_caller]
    fn wrap<C>(self, context: C) -> Self::Output
    where
        C: std::fmt::Debug,
    {
        match self {
            Ok(value) => Ok(value),
            Err(error) => Err(Error::new(context, error)),
        }
    }
}

impl<T> WrapError for Result<T, Error> {
    type Output = Result<T, Error>;

    #[track_caller]
    fn wrap<C>(self, context: C) -> Self::Output
    where
        C: std::fmt::Debug,
    {
        match self {
            Ok(value) => Ok(value),
            Err(mut error) => {
                error.push_context(context);
                Err(error)
            }
        }
    }
}

pub struct Error {
    backtrace: Vec<ErrorContext>,
    source: Box<dyn std::error::Error>,
}

unsafe impl Send for Error {}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Error:")?;
        for context in self.backtrace.iter().rev() {
            writeln!(f, "\t{:?}", context)?;
        }
        writeln!(f, "Caused by: \n\t{:?}", self.source)?;
        Ok(())
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.backtrace.first() {
            Some(context) => write!(f, "{}", context),
            None => write!(f, "{}", self.source),
        }
    }
}

impl std::error::Error for Error {}

impl Error {
    #[track_caller]
    pub fn new<C>(context: C, source: Box<dyn std::error::Error>) -> Self
    where
        C: std::fmt::Debug,
    {
        Self {
            backtrace: vec![ErrorContext::new(context)],
            source,
        }
    }

    #[track_caller]
    pub fn push_context<C>(&mut self, context: C)
    where
        C: std::fmt::Debug,
    {
        self.backtrace.push(ErrorContext::new(context))
    }
}

pub struct ErrorContext {
    location: std::panic::Location<'static>,
    context: String,
}

impl std::fmt::Debug for ErrorContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} at {}:{}",
            self.context,
            self.location.file(),
            self.location.line(),
        )
    }
}

impl std::fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.context)
    }
}

impl ErrorContext {
    #[track_caller]
    pub fn new<C>(context: C) -> Self
    where
        C: std::fmt::Debug,
    {
        Self {
            location: *Location::caller(),
            context: format!("{:?}", context),
        }
    }
}

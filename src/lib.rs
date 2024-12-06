#![feature(negative_impls, auto_traits)]

use std::error::Error;

use easy_ext::ext;

/// An exception type that can be either recoverable or unrecoverable.
///
/// This removes the need for panic!() in the codebase.
#[derive(Debug, thiserror::Error)]
pub enum Exception<E = Unrecoverable> {
    #[error("A recoverable exception occured: {0}")]
    Unrecoverable(eyre::Report),
    #[error("An unrecoverable exception occured: {0}")]
    Recoverable(E),
}

impl Exception<Unrecoverable> {
    pub fn into_unrecoverable(self) -> eyre::Report {
        match self {
            Self::Unrecoverable(e) => e,
            Self::Recoverable(_) => unreachable!(),
        }
    }
}

impl<E> Exception<E> {
    pub fn new_recoverable(e: impl Into<eyre::Report>) -> Self {
        Self::Unrecoverable(e.into())
    }

    pub fn new_unrecoverable(e: E) -> Self {
        Self::Recoverable(e)
    }

    pub fn is_recoverable(&self) -> bool {
        matches!(self, Self::Recoverable(_))
    }

    pub fn try_as_recoverable(&self) -> eyre::Result<&E> {
        match self {
            Self::Recoverable(e) => Ok(e),
            _ => Err(eyre::eyre!("Not a specific error")),
        }
    }

    pub fn try_as_recoverable_mut(&mut self) -> eyre::Result<&mut E> {
        match self {
            Self::Recoverable(e) => Ok(e),
            _ => Err(eyre::eyre!("Not a specific error")),
        }
    }

    pub fn try_into_recoverable(self) -> Result<E, Self> {
        match self {
            Self::Recoverable(e) => Ok(e),
            e => Err(e),
        }
    }

    pub fn try_as_unrecoverable(&self) -> eyre::Result<&eyre::Report> {
        match self {
            Self::Unrecoverable(e) => Ok(e),
            _ => Err(eyre::eyre!("Not a generic error")),
        }
    }

    pub fn try_as_unrecoverable_mut(&mut self) -> eyre::Result<&mut eyre::Report> {
        match self {
            Self::Unrecoverable(e) => Ok(e),
            _ => Err(eyre::eyre!("Not a generic error")),
        }
    }

    pub fn try_into_unrecoverable(self) -> Result<eyre::Report, Self> {
        match self {
            Self::Unrecoverable(e) => Ok(e),
            e => Err(e),
        }
    }

    /// Splits the `Exception<E>` into a `(Option<eyre::Report>, Option<E>)`.
    pub fn split(self) -> (Option<eyre::Report>, Option<E>) {
        match self {
            Self::Unrecoverable(e) => (Some(e), None),
            Self::Recoverable(e) => (None, Some(e)),
        }
    }

    /// Maps an `Exception<E>` to `Exception<T>` by applying a function to a contained value.
    pub fn map<F, T>(self, f: F) -> Exception<T>
    where
        F: FnOnce(E) -> T,
    {
        match self {
            Self::Unrecoverable(e) => Exception::Unrecoverable(e),
            Self::Recoverable(e) => Exception::Recoverable(f(e)),
        }
    }

    /// Maps an `Exception<E>` to `Exception<T>` using `Into<T>`.
    pub fn map_into<T>(self) -> Exception<T>
    where
        E: Into<T>,
    {
        self.map(Into::into)
    }
}

/// Marks an error as recoverable, allowing it to be converted into an [`Exception<E>`]
/// using `?` or [`Into`].
pub auto trait RecoverableError {}

// Exceptions are not recoverable.
impl<T> !RecoverableError for Exception<T> {}
// Unrecoverable errors are not recoverable.
impl !RecoverableError for Unrecoverable {}

// Implement `RecoverableError` for all types that implement `Into<E>` and the marker
// trait `RecoverableError`.
impl<T, E> From<T> for Exception<E>
where
    T: RecoverableError + Into<E>,
{
    fn from(error: T) -> Self {
        Exception::Recoverable(error.into())
    }
}

impl<E> From<Exception> for Exception<E>
where
    E: RecoverableError,
{
    fn from(error: Exception) -> Self {
        match error {
            Exception::Unrecoverable(e) => Exception::Unrecoverable(e),
            Exception::Recoverable(_) => unreachable!(),
        }
    }
}

impl<E> From<eyre::Report> for Exception<E> {
    fn from(error: eyre::Report) -> Self {
        Exception::Unrecoverable(error)
    }
}

/// An error that can never be instantiated. (Unreachable)
#[derive(Debug, thiserror::Error)]
#[error("unreachable")]
pub enum Unrecoverable {}

pub type ExceptionResult<T, E = Unrecoverable> = Result<T, Exception<E>>;

#[ext(ExceptionResultExt)]
pub impl<T, E> ExceptionResult<T, E> {
    #[inline]
    fn map_exception<F, E2>(self, f: F) -> ExceptionResult<T, E2>
    where
        F: FnOnce(E) -> E2,
    {
        self.map_err(|e| e.map(f))
    }

    #[inline]
    fn map_exception_into<E2>(self) -> ExceptionResult<T, E2>
    where
        E: Into<E2>,
    {
        self.map_exception(Into::into)
    }

    /// Splits the [`ExceptionResult<T, E>`] into a `Result<Result<T, E>, eyre::Report>`.
    ///
    /// This allows for easy propagation of unrecoverable errors.
    #[inline]
    fn split(self) -> eyre::Result<Result<T, E>> {
        match self {
            Ok(t) => Ok(Ok(t)),
            Err(e) => match e {
                Exception::Unrecoverable(e) => Err(e),
                Exception::Recoverable(e) => Ok(Err(e)),
            },
        }
    }
}

#[ext(UnrecoverableExceptionResultExt)]
pub impl<T> ExceptionResult<T> {
    #[inline]
    fn into_unrecoverable(self) -> Result<T, eyre::Report> {
        match self {
            Ok(val) => Ok(val),
            Err(e) => Err(e.into_unrecoverable()),
        }
    }
}

#[ext(ResultExt)]
pub impl<T, E> Result<T, E> {
    #[inline]
    fn recoverable(self) -> ExceptionResult<T, E> {
        self.map_err(Exception::Recoverable)
    }

    #[inline]
    fn unrecoverable(self) -> ExceptionResult<T>
    where
        E: Into<eyre::Report>,
    {
        self.map_err(|e| Exception::Unrecoverable(e.into()))
    }
}

pub trait Finalize: Sized {
    type Output<T>;

    fn finalize<T>(res: Result<T, Self>) -> Self::Output<T>;
}

impl<E: RecoverableError> Finalize for E {
    type Output<T> = Result<T, E>;

    fn finalize<T>(res: Result<T, E>) -> Result<T, E> {
        res
    }
}

impl Finalize for Unrecoverable {
    type Output<T> = T;

    fn finalize<T>(res: Result<T, Unrecoverable>) -> T {
        res.expect("NoCustomBackendError can't be created")
    }
}

use exception::{Exception, ExceptionResult, ExceptionResultExt};
use eyre::Context;

#[derive(Debug, thiserror::Error)]
#[error("MyError: {0}")]
struct MyError(String);

impl From<String> for MyError {
    fn from(s: String) -> Self {
        Self(s)
    }
}

fn main() {
    match test().split().unwrap() {
        Ok(()) => todo!(),
        Err(e) => todo!(),
    }
}

fn test() -> ExceptionResult<(), MyError> {
    my_error()?;

    generic_error()?;

    string_error()?;

    unrecoverable_exception()?;

    myerror_exception()?;

    string_exception().split()??;
    string_exception().map_exception_into::<MyError>()?;

    if let Err(e) = string_exception().split()? {
        println!("An error occured{:?}", e)
    }

    Ok(())
}

fn my_error() -> Result<(), MyError> {
    Ok(())
}

fn generic_error() -> eyre::Result<()> {
    Ok(())
}

fn string_error() -> Result<(), String> {
    Ok(())
}

fn unrecoverable_exception() -> ExceptionResult<()> {
    Ok(())
}

fn myerror_exception() -> ExceptionResult<(), MyError> {
    Ok(())
}

fn string_exception() -> ExceptionResult<(), String> {
    Ok(())
}

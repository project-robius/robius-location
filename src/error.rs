/// The result of an authentication operation.
pub type Result<T> = std::result::Result<T, Error>;

// TODO: How specific do we want the errors to be?

/// An error produced during authentication.
#[derive(Debug)]
pub enum Error {
    Unknown,
}

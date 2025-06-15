use proc_macro2::Span;
use syn::Error as SynError;

/// Custom error types for the HTTP provider macro.
///
/// This enum represents the different types of errors that can occur
/// during macro expansion and code generation.
#[derive(Debug)]
pub enum MacroError {
    /// Wraps syntax-related errors from the `syn` crate
    Syn(SynError),

    /// Represents custom errors with a message and location span
    Custom {
        /// Error message describing what went wrong
        message: String,
        /// Source code location where the error occurred
        span: Span,
    },
}

impl MacroError {
    /// Converts the error into a token stream that can be used in compile-time error reporting.
    ///
    /// This method ensures that errors are properly displayed in the Rust compiler's
    /// error messages with appropriate source code locations.
    ///
    /// # Returns
    /// * `proc_macro2::TokenStream` - A token stream representing the error message
    pub fn to_compile_error(self) -> proc_macro2::TokenStream {
        match self {
            MacroError::Syn(err) => err.to_compile_error(),
            MacroError::Custom { message, span } => SynError::new(span, message).to_compile_error(),
        }
    }
}

impl From<SynError> for MacroError {
    /// Provides automatic conversion from syn::Error to MacroError.
    ///
    /// This implementation allows using the `?` operator with syn::Error results
    /// in functions that return MacroError.
    fn from(err: SynError) -> Self {
        MacroError::Syn(err)
    }
}

/// A specialized Result type for macro operations.
///
/// This type alias makes it more convenient to work with Results that use MacroError
/// as their error type.
pub type MacroResult<T> = std::result::Result<T, MacroError>;

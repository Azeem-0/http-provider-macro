use proc_macro2::Span;
use syn::Error as SynError;

#[derive(Debug)]
pub enum MacroError {
    Syn(SynError),
    Custom { message: String, span: Span },
}

impl MacroError {
    pub fn to_compile_error(self) -> proc_macro2::TokenStream {
        match self {
            MacroError::Syn(err) => err.to_compile_error(),
            MacroError::Custom { message, span } => SynError::new(span, message).to_compile_error(),
        }
    }
}

impl From<SynError> for MacroError {
    fn from(err: SynError) -> Self {
        MacroError::Syn(err)
    }
}

pub type MacroResult<T> = std::result::Result<T, MacroError>;

use proc_macro2::TokenStream;

pub trait ExpectTokens {
    fn convert(self) -> String;
}

impl ExpectTokens for TokenStream {
    fn convert(self) -> String {
        let Ok(file) = syn::parse2::<syn::File>(self) else {
            return "[ERROR: Unable to parse input to expect_tokens!]".to_string();
        };
        prettyplease::unparse(&file)
    }
}
impl<E: std::fmt::Debug> ExpectTokens for Result<TokenStream, E> {
    fn convert(self) -> String {
        match self {
            Ok(tokens) => tokens.convert(),
            Err(e) => format!("Error: {:?}", e),
        }
    }
}

impl ExpectTokens for &TokenStream {
    fn convert(self) -> String {
        self.clone().convert()
    }
}

impl<E: std::fmt::Debug> ExpectTokens for &Result<TokenStream, E> {
    fn convert(self) -> String {
        match self {
            Ok(tokens) => tokens.convert(),
            Err(e) => format!("Error: {:?}", e),
        }
    }
}

use proc_macro2::{Ident, TokenStream};
use quote::ToTokens;
use syn::parse::{Parse, ParseBuffer};
use syn::punctuated::Punctuated;
use syn::token::Paren;
use syn::*;

#[derive(Debug)]
pub struct Attr {
    pub items: Punctuated<AttrItem, Token![,]>,
}

impl Parse for Attr {
    fn parse(input: &ParseBuffer) -> syn::parse::Result<Self> {
        Ok(Attr {
            items: input.call(Punctuated::<AttrItem, Token![,]>::parse_terminated)?,
        })
    }
}

mod kw {

    syn::custom_keyword!(인용);
    syn::custom_keyword!(안함);
    syn::custom_keyword!(문자열);
    syn::custom_keyword!(문서화주석);

    syn::custom_keyword!(입력);
    syn::custom_keyword!(자동);
    syn::custom_keyword!(명령행인자);
    syn::custom_keyword!(표준입력);
    syn::custom_keyword!(인자);
}

#[derive(Debug)]
pub enum AttrItem {
    Quote {
        quote_token: kw::인용,
        eq_token: Token![=],
        quote: Quote,
    },
    Input {
        input_token: kw::입력,
        eq_token: Token![=],
        input: Input,
    },
}

impl Parse for AttrItem {
    fn parse(input: &ParseBuffer) -> syn::parse::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::인용) {
            let quote_token = input.parse()?;
            let eq_token: Token![=] = input.parse()?;
            let quote = input.parse()?;
            Ok(AttrItem::Quote {
                quote_token,
                eq_token,
                quote,
            })
        } else if lookahead.peek(kw::입력) {
            let input_token = input.parse()?;
            let eq_token: Token![=] = input.parse()?;
            let input = input.parse()?;
            Ok(AttrItem::Input {
                input_token,
                eq_token,
                input,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for AttrItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            AttrItem::Quote {
                quote_token,
                eq_token,
                quote,
            } => {
                quote_token.to_tokens(tokens);
                eq_token.to_tokens(tokens);
                quote.to_tokens(tokens);
            }
            AttrItem::Input {
                input_token,
                eq_token,
                input,
            } => {
                input_token.to_tokens(tokens);
                eq_token.to_tokens(tokens);
                input.to_tokens(tokens);
            }
        }
    }
}

#[derive(Debug)]
pub enum Quote {
    Raw(kw::안함),
    DocComment(kw::문서화주석),
    String(kw::문자열),
}

impl Parse for Quote {
    fn parse(input: &ParseBuffer) -> syn::parse::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::안함) {
            Ok(Quote::Raw(input.parse()?))
        } else if lookahead.peek(kw::문서화주석) {
            Ok(Quote::DocComment(input.parse()?))
        } else if lookahead.peek(kw::문자열) {
            Ok(Quote::String(input.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for Quote {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Quote::Raw(x) => x.to_tokens(tokens),
            Quote::DocComment(x) => x.to_tokens(tokens),
            Quote::String(x) => x.to_tokens(tokens),
        }
    }
}

#[derive(Debug)]
pub enum Input {
    Auto(kw::자동),
    Cli(kw::명령행인자),
    Stdin(kw::표준입력),
    Arg {
        arg_token: kw::인자,
        paren_token: Paren,
        name: Ident,
    },
}

impl Parse for Input {
    fn parse(input: &ParseBuffer) -> syn::parse::Result<Input> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::자동) {
            Ok(Input::Auto(input.parse()?))
        } else if lookahead.peek(kw::명령행인자) {
            Ok(Input::Cli(input.parse()?))
        } else if lookahead.peek(kw::표준입력) {
            Ok(Input::Stdin(input.parse()?))
        } else if lookahead.peek(kw::인자) {
            let arg_token = input.parse()?;
            let content;
            let paren_token = parenthesized!(content in input);
            let name = content.parse()?;
            Ok(Input::Arg {
                arg_token,
                paren_token,
                name,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for Input {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Input::Auto(x) => x.to_tokens(tokens),
            Input::Cli(x) => x.to_tokens(tokens),
            Input::Stdin(x) => x.to_tokens(tokens),
            Input::Arg {
                arg_token,
                paren_token,
                name,
            } => {
                arg_token.to_tokens(tokens);
                paren_token.surround(tokens, |tokens| {
                    name.to_tokens(tokens);
                });
            }
        }
    }
}

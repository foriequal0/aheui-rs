use proc_macro2::{Ident, TokenStream};
use syn::export::ToTokens;
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

    syn::custom_keyword!(quote);
    syn::custom_keyword!(raw);
    syn::custom_keyword!(str);
    syn::custom_keyword!(doc_comment);

    syn::custom_keyword!(input);
    syn::custom_keyword!(auto);
    syn::custom_keyword!(cli);
    syn::custom_keyword!(stdin);
    syn::custom_keyword!(arg);
}

#[derive(Debug)]
pub enum AttrItem {
    Quote {
        quote_token: kw::quote,
        eq_token: Token![=],
        quote: Quote,
    },
    Input {
        input_token: kw::input,
        eq_token: Token![=],
        input: Input,
    },
}

impl Parse for AttrItem {
    fn parse(input: &ParseBuffer) -> syn::parse::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::quote) {
            let quote_token = input.parse()?;
            let eq_token: Token![=] = input.parse()?;
            let quote = input.parse()?;
            Ok(AttrItem::Quote {
                quote_token,
                eq_token,
                quote,
            })
        } else if lookahead.peek(kw::input) {
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
    Raw(kw::raw),
    DocComment(kw::doc_comment),
    String(kw::str),
}

impl Parse for Quote {
    fn parse(input: &ParseBuffer) -> syn::parse::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::raw) {
            Ok(Quote::Raw(input.parse()?))
        } else if lookahead.peek(kw::doc_comment) {
            Ok(Quote::DocComment(input.parse()?))
        } else if lookahead.peek(kw::str) {
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
    Auto(kw::auto),
    Cli(kw::cli),
    Stdin(kw::stdin),
    Arg {
        arg_token: kw::arg,
        paren_token: Paren,
        name: Ident,
    },
}

impl Parse for Input {
    fn parse(input: &ParseBuffer) -> syn::parse::Result<Input> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::auto) {
            Ok(Input::Auto(input.parse()?))
        } else if lookahead.peek(kw::cli) {
            Ok(Input::Cli(input.parse()?))
        } else if lookahead.peek(kw::stdin) {
            Ok(Input::Stdin(input.parse()?))
        } else if lookahead.peek(kw::arg) {
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

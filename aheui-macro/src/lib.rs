#![feature(proc_macro_span, proc_macro_diagnostic)]
extern crate proc_macro;

use std::collections::HashMap;
use std::io::Write;

use aheui_core::OwnedCode;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::ops::Range;
use std::str::FromStr;
use syn;
use syn::spanned::Spanned;
use syn::*;

mod attr;

#[proc_macro_attribute]
pub fn aheui(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let attr = parse_macro_input!(attr as attr::Attr);
    let item_fn: ItemFn = parse_macro_input!(item as ItemFn);

    let config = parse_config(&attr, &item_fn.sig);

    let code = {
        let lines = get_lines(&config, &item_fn);
        let owned = OwnedCode::parse_lines(lines.iter().map(|x| x.as_str()));
        let borrowed = owned.render_as_borrowed("::aheui_core::");
        TokenStream::from_str(&borrowed).unwrap()
    };
    let fnsig = item_fn.sig;
    let input_prepare = config.input.prepare_input();
    let output_prepare = config.output.prepare_output();
    let output_convert = config.output.convert_output();

    let result = quote! {
        #fnsig {
            use ::aheui_core;
            use ::std::convert::TryInto;
            use ::std::io::BufRead;
            use ::std::io::Write;

            #input_prepare
            #output_prepare

            let code = #code;
            let result = ::aheui_core::Env::new(code, &mut input, &mut output).execute();

            #output_convert
        }
    };
    proc_macro::TokenStream::from(result)
}

struct Config {
    quote: Quote,
    input: Input,
    output: Output,
}

#[derive(Debug)]
enum Quote {
    Raw,
    DocComment,
    String,
}

impl<'a> From<&attr::Quote> for Quote {
    fn from(quote: &attr::Quote) -> Quote {
        match quote {
            attr::Quote::Raw(_) => Quote::Raw,
            attr::Quote::DocComment(_) => Quote::DocComment,
            attr::Quote::String(_) => Quote::String,
        }
    }
}

#[derive(Debug)]
enum Input {
    Stdin,
    Arg(Ident),
    Cli(Span),
    Auto,
}

impl Input {
    fn prepare_input(&self) -> TokenStream {
        match self {
            Input::Stdin => quote! {
                let stdin = ::std::io::stdin();
                let mut input = stdin.lock();
            },
            Input::Arg(ident) => quote! {
                let mut input = ::std::io::Cursor::new(#ident);
            },
            Input::Cli(_) => quote! {
                let some_arg = ::std::env::args().nth(1);
                let arg = match some_arg.as_ref() {
                    Some(arg) => arg.as_str(),
                    None => {
                        eprintln!("오류: 한 개의 실행 인자가 주어지지 않았습니다");
                        ::std::process::exit(-1);
                    },
                };
                let mut input = ::std::io::Cursor::new(arg);
            },
            Input::Auto => quote! {
                let some_arg = ::std::env::args().nth(1);
                let mut cursor_input = some_arg.as_ref().map(|arg| ::std::io::Cursor::new(arg));

                let stdin = ::std::io::stdin();
                let mut stdin_input = if cursor_input.is_some() {
                    None
                } else {
                    Some(stdin.lock())
                };

                let mut input = cursor_input.as_mut().map(|i| i as &mut dyn BufRead)
                    .or(stdin_input.as_mut().map(|i| i as &mut dyn BufRead))
                    .unwrap();
            },
        }
    }
}

#[derive(Debug)]
enum Output {
    Stdout,
    String,
    Code,
    CodeString,
    StringCode,
    ExitCode,
}

impl Output {
    fn prepare_output(&self) -> TokenStream {
        match self {
            Output::Stdout | Output::Code | Output::ExitCode => quote! {
                let stdout = ::std::io::stdout();
                let mut output = stdout.lock();
            },
            _ => quote! {
                let mut output = Vec::new();
            },
        }
    }

    fn convert_output(&self) -> TokenStream {
        match self {
            Output::Stdout => quote! {
                output.flush().unwrap()
            },
            Output::String => quote! {
                String::from_utf8(output).unwrap()
            },
            Output::Code => quote! {
                result.try_into().unwrap()
            },
            Output::CodeString => quote! {
                let exit_code = result.try_into().unwrap();
                let string = String::from_utf8(output).unwrap();
                (exit_code, string)
            },
            Output::StringCode => quote! {
                let exit_code = result.try_into().unwrap();
                let string = String::from_utf8(output).unwrap();
                (string, exit_code)
            },
            Output::ExitCode => quote! {
                output.flush().unwrap();
                ::std::process::exit(result);
            },
        }
    }
}

fn parse_config(attr: &attr::Attr, signature: &Signature) -> Config {
    fn get_quote(attr: &attr::Attr) -> Quote {
        let mut result = Quote::Raw;
        let mut seen = HashMap::new();
        for arg_item in attr.items.iter() {
            let discriminant = std::mem::discriminant(arg_item);
            if let Some(existing) = seen.insert(discriminant, arg_item) {
                arg_item
                    .span()
                    .unwrap()
                    .error("이전 옵션을 덮어씁니다")
                    .emit();
                existing.span().unwrap().note("이전 옵션은 여기에").emit();
            }

            match arg_item {
                attr::AttrItem::Quote { quote, .. } => result = Quote::from(quote.clone()),
                _ => {}
            }
        }
        result
    }
    let (input, output) = get_input_output(attr, signature);
    Config {
        quote: get_quote(attr),
        input,
        output,
    }
}

fn get_input_output(attr: &attr::Attr, signature: &Signature) -> (Input, Output) {
    fn get_input<'a>(
        attr: &'a attr::Attr,
        signature: &Signature,
    ) -> Option<(Input, Option<&'a attr::AttrItem>)> {
        for item in attr.items.iter() {
            let input = if let attr::AttrItem::Input { input, .. } = item {
                input
            } else {
                continue;
            };
            let input = match input {
                attr::Input::Auto(_) => Input::Auto,
                attr::Input::Cli(kw) => Input::Cli(kw.span()),
                attr::Input::Stdin(_) => Input::Stdin,
                attr::Input::Arg { name, .. } => Input::Arg(name.clone()),
            };
            return Some((input, Some(item)));
        }
        for input in signature.inputs.iter() {
            let pat = match input {
                FnArg::Receiver(_) => continue,
                FnArg::Typed(pat) => pat,
            };
            let ident = match pat.pat.as_ref() {
                Pat::Ident(pat_ident) => &pat_ident.ident,
                _ => continue,
            };
            if ident == "input" {
                return Some((Input::Arg(ident.clone()), None));
            }
        }
        None
    }

    fn is_string(typ: &Type) -> bool {
        match typ {
            Type::Path(path) if path.path.is_ident("String") => true,
            Type::Group(group) if is_string(&group.elem) => true,
            Type::Paren(paren) if is_string(&paren.elem) => true,
            _ => false,
        }
    }

    fn is_integer(typ: &Type) -> bool {
        const INTEGER_TYPE_IDENTS: &[&str] = &[
            "isize", "i8", "i16", "i32", "i64", "i128", "u8", "usize", "u8", "u16", "u32", "u64",
            "u128",
        ];
        match typ {
            Type::Path(path) => {
                for ident in INTEGER_TYPE_IDENTS {
                    if path.path.is_ident(ident) {
                        return true;
                    }
                }
                false
            }
            Type::Group(group) if is_integer(&group.elem) => true,
            Type::Paren(paren) if is_integer(&paren.elem) => true,
            _ => false,
        }
    }

    fn get_output(signature: &Signature) -> Output {
        let typ = match &signature.output {
            ReturnType::Default => return Output::Stdout,
            ReturnType::Type(_, typ) => typ.as_ref(),
        };
        let elems = match typ {
            Type::Tuple(tuple) => tuple.elems.iter().collect::<Vec<_>>(),
            _ if is_string(typ) => return Output::String,
            _ if is_integer(typ) => return Output::Code,
            _ => {
                typ.span()
                    .unwrap()
                    .error("지원되지 않는 리턴 타입 형식")
                    .emit();
                panic!();
            }
        };
        match elems.as_slice() {
            &[] => Output::Stdout,
            &[left, right] if is_string(left) && is_integer(right) => Output::StringCode,
            &[left, right] if is_integer(left) && is_string(right) => Output::CodeString,
            _ => {
                typ.span()
                    .unwrap()
                    .error("지원되지 않는 리턴 타입 형식")
                    .emit();
                panic!()
            }
        }
    }

    if signature.ident == "main" {
        let input = match get_input(attr, signature) {
            None => Input::Auto,
            Some((Input::Arg(ident), _)) => {
                ident
                    .span()
                    .unwrap()
                    .error("지원되지 않는 입력 형식")
                    .emit();
                panic!();
            }
            Some((input, _)) => input,
        };
        (input, Output::ExitCode)
    } else {
        let input = match get_input(attr, signature) {
            None => Input::Stdin,
            Some((Input::Cli(span), _)) => {
                span.unwrap().error("지원되지 않는 입력 형식").emit();
                panic!();
            }
            Some((Input::Auto, attr_item)) => {
                if let Some(item) = attr_item {
                    item.span().unwrap().error("지원되지 않는 입력 형식").emit();
                } else {
                }
                panic!();
            }
            Some((input, _)) => input,
        };
        (input, get_output(signature))
    }
}

fn get_lines<'a>(config: &Config, item_fn: &'a ItemFn) -> Vec<String> {
    fn from_raw(item_fn: &ItemFn) -> Vec<String> {
        let range = parse_span(&item_fn.block.brace_token.span);
        let source_text = item_fn.block.span().unwrap().source_text().unwrap();
        let body = &source_text[1..range.end - range.start - 1];
        let trimmed = trim_empty_lines(body.lines());
        dedent(trimmed)
    }

    fn from_doc_comment(item_fn: &ItemFn) -> Vec<String> {
        let attrs = &item_fn.attrs;
        let mut result: Vec<String> = Vec::new();
        for attr in attrs {
            match &attr.style {
                &AttrStyle::Inner(_) if attr.path.is_ident("doc") => {}
                _ => continue,
            }
            let value = match attr.parse_meta() {
                Ok(Meta::NameValue(MetaNameValue {
                    lit: Lit::Str(litstr),
                    ..
                })) => litstr.value(),
                _ => {
                    attr.span().unwrap().error("지원되지 않는 형식").emit();
                    panic!();
                }
            };
            result.append(&mut trim_empty_lines(value.lines()));
        }
        dedent(result)
    }

    fn from_str(item_fn: &ItemFn) -> Vec<String> {
        let stmts = &item_fn.block.stmts;
        let mut result = Vec::new();
        for stmt in stmts {
            let expr = match stmt {
                Stmt::Expr(expr) | Stmt::Semi(expr, _) => expr,
                _ => {
                    stmt.span().unwrap().error("지원되지 않는 형식").emit();
                    panic!();
                }
            };
            let value = match expr {
                Expr::Lit(ExprLit {
                    lit: Lit::Str(litstr),
                    ..
                }) => litstr.value(),
                _ => {
                    stmt.span().unwrap().error("지원되지 않는 형식").emit();
                    panic!();
                }
            };
            result.append(&mut trim_empty_lines(value.lines()));
        }
        dedent(result)
    }

    match config.quote {
        Quote::Raw => from_raw(item_fn),
        Quote::DocComment => from_doc_comment(item_fn),
        Quote::String => from_str(item_fn),
    }
}

fn dedent(lines: Vec<String>) -> Vec<String> {
    fn get_indent(line: &str) -> &str {
        let begin_of_non_ws = line
            .find(|x: char| !x.is_whitespace())
            .unwrap_or(line.len());
        &line[..begin_of_non_ws]
    }

    fn get_common_indent<'a>(a: &'a str, b: Option<&'a str>) -> &'a str {
        if b.is_none() {
            return a;
        }
        let mut chars_a = a.chars();
        let mut chars_b = b.unwrap().chars();
        loop {
            let char_a = chars_a.next();
            let char_b = chars_b.next();
            if char_a != char_b || char_a == None && char_b == None {
                return &a[0..(a.len() - chars_a.as_str().len())];
            }
        }
    }

    if lines.is_empty() {
        return Vec::new();
    }

    let mut common_indent = None;
    for line in lines.iter() {
        let indent = get_indent(&line);
        common_indent = Some(get_common_indent(indent, common_indent));
    }
    let indent_size = common_indent.unwrap().len();
    lines
        .iter()
        .map(|line| line[indent_size..].to_string())
        .collect()
}

/// HACK: Span이 range를 직접 제공하지 않음
fn parse_span(span: &Span) -> Range<usize> {
    let mut buffer = Vec::new();
    write!(&mut buffer, "{:?}", span).unwrap();
    let start_left = buffer.iter().position(|x| *x == '(' as u8).unwrap() + 1;
    let start_right = start_left
        + buffer[start_left..]
            .iter()
            .position(|x| *x == '.' as u8)
            .unwrap();
    let end_left = start_right + 2;
    let end_right = buffer.len() - 1;
    let start = std::str::from_utf8(&buffer[start_left..start_right])
        .unwrap()
        .parse()
        .unwrap();
    let end = std::str::from_utf8(&buffer[end_left..end_right])
        .unwrap()
        .parse()
        .unwrap();
    start..end
}

fn trim_empty_lines<'a, I: Iterator<Item = &'a str>>(lines: I) -> Vec<String> {
    fn is_all_ws(line: &str) -> bool {
        line.chars().all(|c| c.is_ascii_whitespace())
    }

    let mut trimmed: Vec<_> = lines
        .skip_while(|line| is_all_ws(line))
        .map(|line| line.to_string())
        .collect();
    loop {
        match trimmed.last() {
            Some(line) if is_all_ws(line) => {
                trimmed.pop();
            }
            _ => break,
        }
    }
    return trimmed;
}

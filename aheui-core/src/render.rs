use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};

use crate::inst::{CursorControl, Inst, Oper, Select};
use crate::vm::BorrowedCode;
use crate::{Address, Cursor, Step};

impl ToTokens for Inst {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let cursor_control = self.cursor_control;
        let oper = self.oper;

        tokens.extend(quote! { ::aheui_core::Inst::new(#cursor_control, #oper) });
    }
}

impl ToTokens for CursorControl {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let variant_name = match self {
            CursorControl::Nop => "Nop",
            CursorControl::Left => "Left",
            CursorControl::Left2 => "Left2",
            CursorControl::Right => "Right",
            CursorControl::Right2 => "Right2",
            CursorControl::Up => "Up",
            CursorControl::Up2 => "Up2",
            CursorControl::Down => "Down",
            CursorControl::Down2 => "Down2",
            CursorControl::MirrorV => "MirrorV",
            CursorControl::MirrorH => "MirrorH",
            CursorControl::Mirror => "Mirror",
        };
        let ident = Ident::new(variant_name, Span::call_site());
        tokens.extend(quote! { ::aheui_core::CursorControl::#ident });
    }
}

impl ToTokens for Oper {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let variant_name = match self {
            Oper::Nop => "Nop",
            Oper::Halt => "Halt",
            Oper::Add => "Add",
            Oper::Mul => "Mul",
            Oper::Sub => "Sub",
            Oper::Div => "Div",
            Oper::Mod => "Mod",
            Oper::WriteChar => "WriteChar",
            Oper::WriteInt => "WriteInt",
            Oper::Pop => "Pop",
            Oper::ReadChar => "ReadChar",
            Oper::ReadInt => "ReadInt",
            Oper::Push(_) => "Push",
            Oper::Dup => "Dup",
            Oper::Swap => "Swap",
            Oper::Select(_) => "Select",
            Oper::Move(_) => "Move",
            Oper::Compare => "Compare",
            Oper::Cond => "Cond",
        };
        let ident = Ident::new(variant_name, Span::call_site());
        match self {
            Oper::Push(value) => tokens.extend(quote! { ::aheui_core::Oper::#ident(#value) }),
            Oper::Select(storage) | Oper::Move(storage) => tokens.extend(quote! {
                ::aheui_core::Oper::#ident(#storage)
            }),
            _ => tokens.extend(quote! { ::aheui_core::Oper::#ident }),
        }
    }
}

impl ToTokens for Select {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Select::Stack(id) => tokens.extend(quote! { ::aheui_core::Select::Stack(#id) }),
            Select::Queue => tokens.extend(quote! { ::aheui_core::Select::Queue }),
            Select::Channel => tokens.extend(quote! { ::aheui_core::Select::Channel }),
        }
    }
}

impl<'a> ToTokens for BorrowedCode<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let index = self.index;
        let code = self.code;
        tokens.extend(quote! {
            ::aheui_core::BorrowedCode {
                index: &[#(#index),*],
                code: &[#(#code),*],
            }
        });
    }
}

impl ToTokens for Cursor {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let address = self.address;
        let step = self.step;

        tokens.extend(quote! {
            ::aheui_core::Cursor {
                address: #address,
                step: #step,
            }
        });
    }
}

impl ToTokens for Address {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let row = self.row;
        let col = self.col;

        tokens.extend(quote! {
            ::aheui_core::Address {
                row: #row,
                col: #col,
            }
        });
    }
}

impl ToTokens for Step {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Step::Row(row) => tokens.extend(quote! { ::aheui_core::Step::Row(#row) }),
            Step::Column(col) => tokens.extend(quote! { ::aheui_core::Step::Column(#col) }),
        }
    }
}

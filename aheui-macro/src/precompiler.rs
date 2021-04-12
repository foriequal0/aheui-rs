use std::collections::{HashMap, HashSet, VecDeque};
use std::convert::TryFrom;
use std::fmt::{Display, Formatter};

use proc_macro2::Ident;
use quote::quote;
use syn::export::{Span, ToTokens, TokenStream2, TokenStreamExt};

use aheui_core::{Address, BorrowedCode, Cursor, CursorControl, Inst, Oper, Step};

pub struct Precompiled {
    pub code: TokenStream2,
    pub create_instance: TokenStream2,
}

pub fn precompile(code: &BorrowedCode) -> Precompiled {
    let mut queue = VecDeque::new();
    queue.push_back(Cursor::new());

    let mut jump_table = HashMap::new();
    let mut traces = HashMap::new();
    while let Some(cursor) = queue.pop_front() {
        if jump_table.contains_key(&cursor) {
            continue; // visted
        }

        let entry = {
            let entry = skip_nops(code, cursor);
            jump_table.insert(cursor, entry);
            match entry {
                Some(entry) => entry,
                None => continue, // infinite loop
            }
        };

        if traces.contains_key(&entry) {
            continue;
        }

        let trace = match HappyPathTrace::trace(code, entry) {
            Some(trace) => trace,
            None => continue,
        };

        match traces.get_mut(&entry) {
            Some(list) => list.push(trace),
            None => traces.insert(entry, vec![trace]),
        }
    }

    result
}

fn skip_nops(code: &BorrowedCode, mut cursor: Cursor) -> Option<EntryPoint> {
    let mut visited = HashSet::new();
    while visited.insert(cursor.address) {
        let inst = code.get_inst(cursor.address).unwrap();
        cursor.advance(code, inst.cursor_control, false);
        if let Oper::Nop = inst.oper {
            continue;
        } else {
            return Some(EntryPoint::new(cursor, inst));
        }
    }
    // Infinite loop
    None
}

struct HappyPathTrace {
    stack_height: usize,
    steps: Vec<LinearOp>,
    end: Cursor,
}

impl HappyPathTrace {
    fn trace(code: &BorrowedCode, start: EntryPoint) -> Option<Self> {
        let mut cursor = Cursor {
            address: start.address,
            step: start.step.unwrap_or(Step::Column(0)), // step is irrelevant if None
        };

        let mut curent_depth = 0;
        let mut max_depth = 0;
        let mut start = None;
        let mut steps = Vec::new();
        loop {
            let inst = code.get_inst(cursor.address).unwrap();
            if let Oper::Nop = inst.oper {
                cursor.advance(code, inst.cursor_control, false);
                continue;
            }
            let op = match LinearOp::try_from(inst.oper) {
                Ok(op) => op,
                Err(_) => break,
            };

            match op {
                LinearOp::Binary(_) => {
                    curent_depth -= 2;
                    max_depth = max_depth.min(curent_depth);
                    curent_depth += 1;
                }
                LinearOp::Pop => {
                    curent_depth -= 1;
                    max_depth = max_depth.min(curent_depth);
                }
                LinearOp::Push(_) => {
                    curent_depth += 1;
                }
                LinearOp::Dup => {
                    curent_depth -= 1;
                    max_depth = max_depth.min(curent_depth);
                    curent_depth += 2;
                }
                LinearOp::Swap => {
                    curent_depth -= 2;
                    max_depth = max_depth.min(curent_depth);
                    curent_depth += 2;
                }
            }

            if start.is_none() {
                start = Some(EntryPoint::new(cursor, inst));
            }
            steps.push(op);
            cursor.advance(code, inst.cursor_control, false);
        }

        if let Some(start) = start {
            Some(Self {
                stack_height: (-max_depth) as _,
                steps,
                end: cursor,
            })
        } else {
            None
        }
    }
}

fn precompile_linear_ops_body(
    storage: Ident,
    ops: &[LinearOp],
    last_cursor: Cursor,
) -> TokenStream2 {
    let mut allocator = VarAlloc::new();
    let mut prologue = TokenStream2::new();
    let mut body = TokenStream2::new();
    let mut epilogue = TokenStream2::new();

    for op in ops {
        match op {
            LinearOp::Binary(bin) => {
                let lhs = allocator.pop();
                if lhs.stack {
                    prologue.extend(quote!(let #lhs = #storage.pop();));
                }

                let rhs = allocator.pop();
                if rhs.stack {
                    prologue.extend(quote!(let #rhs = #storage.pop();));
                }

                let var = allocator.alloc();
                let expr = bin.to_expr(lhs, rhs);
                body.extend(quote! (let #var = #expr;));
            }
            LinearOp::Pop => {
                let var = allocator.pop();
                if var.stack {
                    prologue.extend(quote!(let #var = #storage.pop();));
                }
            }
            LinearOp::Push(v) => {
                let var = allocator.alloc();
                body.extend(quote!(let #var = #v;));
            }
            LinearOp::Dup => {
                let var = allocator.pop();
                if var.stack {
                    prologue.extend(quote!(let #var = #storage.pop();));
                }
                allocator.push(var);
            }
            LinearOp::Swap => {
                let lhs = allocator.pop();
                if lhs.stack {
                    prologue.extend(quote!(let #lhs = #storage.pop();));
                }

                let rhs = allocator.pop();
                if rhs.stack {
                    prologue.extend(quote!(let #rhs = #storage.pop();));
                }

                allocator.push(lhs);
                allocator.push(rhs);
            }
        }
    }

    for var in allocator.stack {
        epilogue.extend(quote!(#storage.push(#var);));
    }

    return quote! {
        #prologue
        #body
        #epilogue
        #last_cursor
    };
}

#[derive(Eq, PartialEq, Hash, Copy, Clone)]
struct EntryPoint {
    address: Address,
    step: Option<Step>,
}

impl EntryPoint {
    fn new(cursor: Cursor, inst: Inst) -> Self {
        let step = match inst.cursor_control {
            CursorControl::Nop
            | CursorControl::MirrorV
            | CursorControl::MirrorH
            | CursorControl::Mirror => Some(cursor.step),
            _ => None,
        };

        EntryPoint {
            address: cursor.address,
            step,
        }
    }
}

#[derive(Eq, PartialEq, Hash)]
struct BudgetedEntryPoint {
    entry: EntryPoint,
    budget: i32,
}

struct VarAlloc {
    count: usize,
    stack: Vec<Var>,
}

#[derive(Eq, PartialEq, Copy, Clone)]
struct Var {
    stack: bool,
    id: usize,
}

impl Display for Var {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.stack {
            write!(f, "stackvar_{}", self.id)
        } else {
            write!(f, "var_{}", self.id)
        }
    }
}

impl ToTokens for Var {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        tokens.append(Ident::new(&format!("{}", self), Span::call_site()))
    }
}

impl VarAlloc {
    fn new() -> Self {
        Self {
            count: 0,
            stack: Vec::new(),
        }
    }

    fn alloc(&mut self) -> Var {
        self.count += 1;
        let var = Var {
            stack: false,
            id: self.count,
        };
        self.stack.push(var);
        var
    }

    fn push(&mut self, var: Var) {
        self.stack.push(var);
    }

    fn pop(&mut self) -> Var {
        if let Some(var) = self.stack.pop() {
            var
        } else {
            self.count += 1;
            Var {
                stack: true,
                id: self.count,
            }
        }
    }
}

#[derive(Copy, Clone)]
enum LinearOp {
    Binary(BinaryOp),

    Pop,
    Push(i32),

    Dup,
    Swap,
}

impl TryFrom<Oper> for LinearOp {
    type Error = ();

    fn try_from(value: Oper) -> Result<LinearOp, Self::Error> {
        let traced_op = match value {
            Oper::Add => LinearOp::Binary(BinaryOp::Add),
            Oper::Mul => LinearOp::Binary(BinaryOp::Mul),
            Oper::Sub => LinearOp::Binary(BinaryOp::Sub),
            Oper::Div => LinearOp::Binary(BinaryOp::Div),
            Oper::Mod => LinearOp::Binary(BinaryOp::Mod),
            Oper::Compare => LinearOp::Binary(BinaryOp::Cmp),
            Oper::Pop => LinearOp::Pop,
            Oper::Push(v) => LinearOp::Push(v as i32),
            Oper::Dup => LinearOp::Dup,
            Oper::Swap => LinearOp::Swap,
            _ => return Err(()),
        };
        Ok(traced_op)
    }
}

#[derive(Copy, Clone)]
enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Cmp,
}

impl BinaryOp {
    fn to_expr(&self, lhs: Var, rhs: Var) -> TokenStream2 {
        match self {
            BinaryOp::Add => quote! { #lhs + #rhs },
            BinaryOp::Sub => quote! { #lhs - #rhs },
            BinaryOp::Mul => quote! { #lhs * #rhs },
            BinaryOp::Div => quote! { #lhs / #rhs },
            BinaryOp::Mod => quote! { #lhs % #rhs },
            BinaryOp::Cmp => quote! { (#lhs < #rhs) as i32 },
        }
    }
}

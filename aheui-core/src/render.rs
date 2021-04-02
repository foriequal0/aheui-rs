use std::fmt::Write as FmtWrite;

use crate::inst::{CursorControl, Inst, Oper, Select};
use crate::vm::BorrowedCode;

pub trait Render {
    fn render(&self, prefix: &str) -> String;
}

impl Render for Inst {
    fn render(&self, prefix: &str) -> String {
        format!(
            "{}Inst::new({}, {})",
            prefix,
            self.cursor_control.render(prefix),
            self.oper.render(prefix)
        )
    }
}

impl Render for CursorControl {
    fn render(&self, prefix: &str) -> String {
        use CursorControl::*;
        let variant = match self {
            Nop => "Nop",
            Left => "Left",
            Left2 => "Left2",
            Right => "Right",
            Right2 => "Right2",
            Up => "Up",
            Up2 => "Up2",
            Down => "Down",
            Down2 => "Down2",
            MirrorV => "MirrorV",
            MirrorH => "MirrorH",
            Mirror => "Mirror",
        };
        format!("{}CursorControl::{}", prefix, variant)
    }
}

impl Render for Oper {
    fn render(&self, prefix: &str) -> String {
        use Oper::*;
        let variant_name = match self {
            Nop => "Nop",
            Halt => "Halt",
            Add => "Add",
            Mul => "Mul",
            Sub => "Sub",
            Div => "Div",
            Mod => "Mod",
            WriteChar => "WriteChar",
            WriteInt => "WriteInt",
            Pop => "Pop",
            ReadChar => "ReadChar",
            ReadInt => "ReadInt",
            Push(_) => "Push",
            Dup => "Dup",
            Swap => "Swap",
            Select(_) => "Select",
            Move(_) => "Move",
            Compare => "Compare",
            Cond => "Cond",
        };
        match self {
            Push(value) => format!("{}Oper::{}({})", prefix, variant_name, value),
            Select(storage) | Move(storage) => format!(
                "{}Oper::{}({})",
                prefix,
                variant_name,
                storage.render(prefix)
            ),
            _ => format!("{}Oper::{}", prefix, variant_name),
        }
    }
}

impl Render for Select {
    fn render(&self, prefix: &str) -> String {
        match self {
            Select::Stack(id) => format!("{}Select::Stack({})", prefix, id),
            Select::Queue => format!("{}Select::Queue", prefix),
            Select::Channel => format!("{}Select::Channel", prefix),
        }
    }
}

impl<'a> BorrowedCode<'a> {
    pub fn render(&self, prefix: &str) -> String {
        let mut buf = String::new();
        write!(buf, "{}BorrowedCode {{", prefix).unwrap();
        write!(buf, "index: &[").unwrap();
        for i in self.index.iter() {
            write!(buf, "{},", i).unwrap();
        }
        write!(buf, "],").unwrap();
        write!(buf, "code: &[").unwrap();
        for code in self.code.iter() {
            write!(buf, "{},", code.render(prefix)).unwrap();
        }
        write!(buf, "],").unwrap();
        write!(buf, "}}").unwrap();
        buf
    }
}

use std::io::{BufRead, Write};

use crate::inst::{CursorControl, Inst};
use crate::storage::StorageSelector;

pub struct BorrowedCode<'a> {
    pub index: &'a [usize],
    pub code: &'a [Inst],
}

impl<'a> BorrowedCode<'a> {
    fn get_line(&self, row: i32) -> Option<&[Inst]> {
        if row < 0 {
            return None;
        }
        let row = row as usize;
        if let &[begin, end] = self.index.get(row..=row + 1)? {
            return self.code.get(begin..end);
        }
        unreachable!();
    }

    pub fn get_inst(&self, address: Address) -> Option<Inst> {
        let col = address.col as usize;
        let line = self.get_line(address.row)?;
        line.get(col).cloned()
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
#[cfg_attr(feature = "derive_debug", derive(Debug))]
pub struct Cursor {
    pub address: Address,
    pub step: Step,
}

impl Cursor {
    pub fn new() -> Cursor {
        Cursor {
            address: Address { row: 0, col: 0 },
            step: Step::Row(1),
        }
    }

    pub fn advance(&mut self, code: &BorrowedCode, cursor_control: CursorControl, reverse: bool) {
        self.step = self.step.next(cursor_control, reverse);
        match self.step {
            Step::Row(amount) => {
                self.address.row += amount as i32;
                if amount > 0 {
                    let height = (code.index.len() - 1) as i32;
                    let mut inst = code.get_inst(self.address);
                    while let None = inst {
                        if self.address.row + 1 < height {
                            self.address.row += 1;
                        } else {
                            self.address.row = 0;
                        }
                        assert!(self.address.row >= 0 && self.address.row < height);
                        inst = code.get_inst(self.address);
                    }
                } else if amount < 0 {
                    let height = (code.index.len() - 1) as i32;
                    let mut inst = code.get_inst(self.address);
                    while let None = inst {
                        if self.address.row >= 1 {
                            self.address.row -= 1;
                        } else {
                            self.address.row = height - 1;
                        }
                        assert!(self.address.row >= 0 && self.address.row < height);
                        inst = code.get_inst(self.address);
                    }
                } else {
                    unreachable!()
                }
            }
            Step::Column(amount) => {
                self.address.col += amount as i32;
                let line = code.get_line(self.address.row).unwrap();
                let line_len = line.len() as i32;
                if self.address.col < 0 && amount < 0 {
                    self.address.col = line_len - 1;
                } else if self.address.col >= line_len && amount > 0 {
                    self.address.col = 0;
                }
                assert!(self.address.col >= 0 && self.address.col < line_len);
            }
        }
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
#[cfg_attr(feature = "derive_debug", derive(Debug))]
pub struct Address {
    pub row: i32,
    pub col: i32,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
#[cfg_attr(feature = "derive_debug", derive(Debug))]
pub enum Step {
    Row(i8),
    Column(i8),
}

impl Step {
    fn next(&self, cursor_control: CursorControl, reverse: bool) -> Step {
        let result = match cursor_control {
            CursorControl::Nop => *self,
            CursorControl::Left => Step::Column(-1),
            CursorControl::Left2 => Step::Column(-2),
            CursorControl::Right => Step::Column(1),
            CursorControl::Right2 => Step::Column(2),
            CursorControl::Up => Step::Row(-1),
            CursorControl::Up2 => Step::Row(-2),
            CursorControl::Down => Step::Row(1),
            CursorControl::Down2 => Step::Row(2),
            CursorControl::MirrorV => match *self {
                Step::Row(value) => Step::Row(-value),
                _ => *self,
            },
            CursorControl::MirrorH => match *self {
                Step::Column(value) => Step::Column(-value),
                _ => *self,
            },
            CursorControl::Mirror => match *self {
                Step::Row(value) => Step::Row(-value),
                Step::Column(value) => Step::Column(-value),
            },
        };
        if reverse {
            result.reverse()
        } else {
            result
        }
    }

    fn reverse(&self) -> Step {
        match *self {
            Step::Row(value) => Step::Row(-value),
            Step::Column(value) => Step::Column(-value),
        }
    }
}

pub struct VM<'a, E> {
    engine: E,
    env: Env<'a>,
}

impl<'a, E> VM<'a, E> {
    pub fn new(env: Env<'a>, engine: E) -> Self {
        Self { engine, env }
    }
}

impl<E> VM<'_, E>
where
    E: Engine,
{
    pub fn execute(mut self) -> i32 {
        loop {
            if let Some(exitcode) = self.engine.step(&mut self.env) {
                return exitcode;
            }
        }
    }
}

pub struct Env<'a> {
    pub cursor: Cursor,
    pub storages: StorageSelector,
    input: &'a mut dyn BufRead,
    pub output: &'a mut dyn Write,
}

impl<'a> Env<'a> {
    pub fn new(input: &'a mut dyn BufRead, output: &'a mut dyn Write) -> Self {
        Env {
            cursor: Cursor::new(),
            storages: StorageSelector::new(),
            input,
            output,
        }
    }
}

impl<'a> Env<'a> {
    pub fn read_char(&mut self) -> i32 {
        let mut buf = [0u8; 4];
        for i in 0..4 {
            self.input
                .read_exact(&mut buf[i..i + 1])
                .expect("Read Error");
            match std::str::from_utf8(&buf[0..i + 1]) {
                Ok(s) => return s.chars().nth(0).unwrap() as i32,
                Err(e) => {
                    if e.error_len().is_none() {
                        continue;
                    } else {
                        panic!("an unexpected byte was encountered");
                    }
                }
            };
        }
        unreachable!();
    }

    pub fn read_int(&mut self) -> i32 {
        let mut line = String::new();
        self.input.read_line(&mut line).expect("Read Error");
        line.trim().parse::<i32>().expect("Parse error")
    }
}

pub trait Engine {
    fn step(&self, env: &mut Env) -> Option<i32>;
}

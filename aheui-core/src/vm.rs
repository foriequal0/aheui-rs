use std::collections::VecDeque;
use std::io::{BufRead, Write};

use crate::inst::{CursorControl, Inst, Oper, Select};

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

    pub fn advance(
        &mut self,
        code: &BorrowedCode,
        cursor_control: CursorControl,
        reverse: bool,
    ) -> Inst {
        self.step = self.step.next(cursor_control, reverse);

        match self.step {
            Step::Row(amount) => {
                self.address.row += amount as i32;
                if amount > 0 {
                    let height = (code.index.len() - 1) as i32;
                    let mut inst = code.get_inst(self.address);
                    loop {
                        if let Some(inst) = inst {
                            break inst;
                        }
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
                    loop {
                        if let Some(inst) = inst {
                            break inst;
                        }
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
                line[self.address.col as usize]
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

pub struct Env<'a> {
    code: BorrowedCode<'a>,
    input: &'a mut dyn BufRead,
    output: &'a mut dyn Write,
}

impl<'a> Env<'a> {
    pub fn new<C>(code: C, input: &'a mut dyn BufRead, output: &'a mut dyn Write) -> Self
    where
        C: Into<BorrowedCode<'a>>,
    {
        Env {
            code: code.into(),
            input,
            output,
        }
    }
}

impl<'a> Env<'a> {
    pub fn execute(mut self) -> i32 {
        let mut cursor = Cursor::new();
        let mut storages = StorageSelector::new();
        let mut inst = self.code.get_inst(cursor.address).unwrap();
        loop {
            let mut reverse = false;
            match inst.oper {
                Oper::Nop => {}
                Oper::Halt => return storages.selected().try_pop().unwrap_or(0),
                Oper::Add => {
                    if let Some((a, b)) = storages.selected().binary_op_assign() {
                        *b = b.wrapping_add(a);
                    } else {
                        reverse = true;
                    }
                }
                Oper::Mul => {
                    if let Some((a, b)) = storages.selected().binary_op_assign() {
                        *b = b.wrapping_mul(a);
                    } else {
                        reverse = true;
                    }
                }
                Oper::Sub => {
                    if let Some((a, b)) = storages.selected().binary_op_assign() {
                        *b = b.wrapping_sub(a);
                    } else {
                        reverse = true;
                    }
                }
                Oper::Div => {
                    if let Some((a, b)) = storages.selected().binary_op_assign() {
                        *b = b.wrapping_div(a);
                    } else {
                        reverse = true;
                    }
                }
                Oper::Mod => {
                    if let Some((a, b)) = storages.selected().binary_op_assign() {
                        *b = b.wrapping_rem(a);
                    } else {
                        reverse = true;
                    }
                }
                Oper::WriteChar => {
                    if let Some(value) = storages.selected().try_pop() {
                        write!(
                            self.output,
                            "{}",
                            std::char::from_u32(value as u32).unwrap()
                        )
                        .unwrap();
                    } else {
                        reverse = true;
                    }
                }
                Oper::WriteInt => {
                    if let Some(value) = storages.selected().try_pop() {
                        write!(self.output, "{}", value).unwrap();
                    } else {
                        reverse = true;
                    }
                }
                Oper::Pop => {
                    if storages.selected().try_pop().is_none() {
                        reverse = true;
                    }
                }
                Oper::ReadChar => {
                    let value = self.read_char();
                    storages.selected().push(value);
                }
                Oper::ReadInt => {
                    let value = self.read_int();
                    storages.selected().push(value);
                }
                Oper::Push(v) => storages.selected().push(v as i32),
                Oper::Dup => {
                    reverse = !storages.selected().dup();
                }
                Oper::Swap => {
                    reverse = !storages.selected().swap();
                }
                Oper::Select(select) => storages.select(select),
                Oper::Move(select) => {
                    if let Some(value) = storages.selected().try_pop() {
                        storages.get_storage(select).push(value);
                    } else {
                        reverse = true;
                    }
                }
                Oper::Compare => {
                    if let Some((a, b)) = storages.selected().binary_op_assign() {
                        *b = (a <= *b) as i32;
                    } else {
                        reverse = true;
                    };
                }
                Oper::Cond => match storages.selected().try_pop() {
                    Some(value) if value != 0 => {}
                    _ => reverse = true,
                },
            };

            inst = cursor.advance(&self.code, inst.cursor_control, reverse);
        }
    }

    fn read_char(&mut self) -> i32 {
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

    fn read_int(&mut self) -> i32 {
        let mut line = String::new();
        self.input.read_line(&mut line).expect("Read Error");
        line.trim().parse::<i32>().expect("Parse error")
    }
}

struct StorageSelector {
    select: Select,
    stacks: [Stack; 28],
    queue: Queue,
}

impl StorageSelector {
    fn new() -> Self {
        Self {
            select: Select::Stack(0),
            stacks: Default::default(),
            queue: Default::default(),
        }
    }

    fn select(&mut self, select: Select) {
        self.select = select;
    }

    fn selected(&mut self) -> &mut dyn Storage {
        self.get_storage(self.select)
    }

    fn get_storage(&mut self, select: Select) -> &mut dyn Storage {
        match select {
            Select::Stack(id) => {
                assert!(id < 28);
                &mut self.stacks[id as usize]
            }
            Select::Queue => &mut self.queue,
            Select::Channel => unreachable!(),
        }
    }
}

trait Storage {
    fn push(&mut self, value: i32);
    fn try_pop(&mut self) -> Option<i32>;
    fn binary_op_assign(&mut self) -> Option<(i32, &mut i32)>;
    fn swap(&mut self) -> bool;
    fn dup(&mut self) -> bool;
}

#[derive(Clone, Default)]
struct Stack {
    stack: Vec<i32>,
}

impl Storage for Stack {
    fn push(&mut self, value: i32) {
        self.stack.push(value)
    }

    fn try_pop(&mut self) -> Option<i32> {
        self.stack.pop()
    }

    fn binary_op_assign(&mut self) -> Option<(i32, &mut i32)> {
        let len = self.stack.len();
        if len >= 2 {
            let a = self.stack.pop().unwrap();
            let b = &mut self.stack[len - 2];
            Some((a, b))
        } else {
            None
        }
    }

    fn swap(&mut self) -> bool {
        let len = self.stack.len();
        if len >= 2 {
            self.stack.swap(len - 1, len - 2);
            true
        } else {
            false
        }
    }

    fn dup(&mut self) -> bool {
        if let Some(peek) = self.stack.last().cloned() {
            self.stack.push(peek);
            true
        } else {
            false
        }
    }
}

#[derive(Clone, Default)]
struct Queue {
    queue: VecDeque<i32>,
}

impl Storage for Queue {
    fn push(&mut self, value: i32) {
        self.queue.push_back(value)
    }

    fn try_pop(&mut self) -> Option<i32> {
        self.queue.pop_front()
    }

    fn binary_op_assign(&mut self) -> Option<(i32, &mut i32)> {
        if self.queue.len() >= 2 {
            let a = self.queue.pop_front().unwrap();
            let b = self.queue.pop_front().unwrap();
            self.queue.push_back(b);
            let b = self.queue.back_mut().unwrap();
            Some((a, b))
        } else {
            None
        }
    }

    fn swap(&mut self) -> bool {
        if self.queue.len() >= 2 {
            self.queue.swap(0, 1);
            true
        } else {
            false
        }
    }

    fn dup(&mut self) -> bool {
        if let Some(peek) = self.queue.front().cloned() {
            self.queue.push_front(peek);
            true
        } else {
            false
        }
    }
}

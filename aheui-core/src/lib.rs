use std::collections::VecDeque;
#[cfg(feature = "render")]
use std::fmt::Write as FmtWrite;
use std::io::{BufRead, Write};

#[cfg(feature = "render")]
pub trait Render {
    fn render(&self, prefix: &str) -> String;
}

#[derive(Copy, Clone, Debug)]
pub struct Inst {
    pub cursor_control: CursorControl,
    pub oper: Oper,
}

impl Inst {
    pub fn new(cursor_control: CursorControl, oper: Oper) -> Inst {
        Inst {
            cursor_control,
            oper,
        }
    }
}

#[cfg(feature = "parse")]
impl std::convert::From<char> for Inst {
    fn from(ch: char) -> Inst {
        if ch < '\u{AC00}' || ch > '\u{D7A3}' {
            return Inst::new(CursorControl::Nop, Oper::Nop);
        }
        let index = ch as u32 - 0xAC00;
        let first = index / (28 * 21);
        let second = index % (28 * 21) / 28;
        let last = index % 28;

        let cursor_control = match second {
            0 /* ㅏ */ => CursorControl::Right,
            2 /* ㅑ */ => CursorControl::Right2,
            4 /* ㅓ */ => CursorControl::Left,
            6 /* ㅕ */ => CursorControl::Left2,
            8 /* ㅗ */ => CursorControl::Up,
            12 /* ㅛ */ => CursorControl::Up2,
            13 /* ㅜ */ => CursorControl::Down,
            17 /* ㅠ */ => CursorControl::Down2,
            18 /* ㅡ */ => CursorControl::MirrorV,
            19 /* ㅢ */ => CursorControl::Mirror,
            20 /* ㅣ */ => CursorControl::MirrorH,
            _ => CursorControl::Nop,
        };

        let oper = match first {
            2 /* ㄴ */ => Oper::Div,
            3 /* ㄷ */ => Oper::Add,
            4 /* ㄸ */ => Oper::Mul,
            5 /* ㄹ */ => Oper::Mod,
            6 /* ㅁ */ => match last {
                21 /* ㅇ */ => Oper::WriteInt,
                27 /* ㅎ */ => Oper::WriteChar,
                _ => Oper::Pop,
            },
            7 /* ㅂ */ => push_from_jongseong_index(last),
            8 /* ㅃ */ => Oper::Dup,
            9 /* ㅅ */ => Oper::Select(Select::from_jongseong_index(last)),
            10 /* ㅆ */ => Oper::Move(Select::from_jongseong_index(last)),
            11 /* ㅇ */ => Oper::Nop,
            12 /* ㅈ */ => Oper::Compare,
            14 /* ㅊ */ => Oper::Cond,
            16 /* ㅌ */ => Oper::Sub,
            17 /* ㅍ */ => Oper::Swap,
            18 /* ㅎ */ => Oper::Halt,
            _ => Oper::Nop,
        };

        Inst::new(cursor_control, oper)
    }
}

#[cfg(feature = "parse")]
fn push_from_jongseong_index(index: u32) -> Oper {
    match index {
        0 => Oper::Push(0),
        1 /* ㄱ */ => Oper::Push(2),
        2 /* ㄲ */ => Oper::Push(4),
        3 /* ㄳ */ => Oper::Push(4),
        4 /* ㄴ */ => Oper::Push(2),
        5 /* ㄵ */ => Oper::Push(5),
        6 /* ㄶ */ => Oper::Push(5),
        7 /* ㄷ */ => Oper::Push(3),
        8 /* ㄹ */ => Oper::Push(5),
        9 /* ㄺ */ => Oper::Push(7),
        10 /* ㄻ */ => Oper::Push(9),
        11 /* ㄼ */ => Oper::Push(9),
        12 /* ㄽ */ => Oper::Push(7),
        13 /* ㄾ */ => Oper::Push(9),
        14 /* ㄿ */ => Oper::Push(9),
        15 /* ㅀ */ => Oper::Push(8),
        16 /* ㅁ */ => Oper::Push(4),
        17 /* ㅂ */ => Oper::Push(4),
        18 /* ㅄ */ => Oper::Push(6),
        19 /* ㅅ */ => Oper::Push(2),
        20 /* ㅆ */ => Oper::Push(4),
        21 /* ㅇ */ => Oper::ReadInt,
        22 /* ㅈ */ => Oper::Push(3),
        23 /* ㅊ */ => Oper::Push(4),
        24 /* ㅋ */ => Oper::Push(3),
        25 /* ㅌ */ => Oper::Push(4),
        26 /* ㅍ */ => Oper::Push(4),
        27 /* ㅎ */ => Oper::ReadChar,
        _ => unreachable!(),
    }
}

#[cfg(feature = "render")]
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

#[derive(Copy, Clone, Debug)]
pub enum CursorControl {
    Nop,
    Left,
    Left2,
    Right,
    Right2,
    Up,
    Up2,
    Down,
    Down2,

    MirrorV,
    MirrorH,
    Mirror,
}

#[cfg(feature = "render")]
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

#[derive(Copy, Clone, Debug)]
pub enum Oper {
    Nop,
    Halt,

    Add,
    Mul,
    Sub,
    Div,
    Mod,

    WriteChar,
    WriteInt,
    Pop,

    ReadChar,
    ReadInt,
    Push(u8),

    Dup,
    Swap,

    Select(Select),
    Move(Select),
    Compare,
    Cond,
}

#[cfg(feature = "render")]
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

#[derive(Copy, Clone, Debug)]
pub enum Select {
    Stack(u8),
    Queue,
    Channel,
}

#[cfg(feature = "parse")]
impl Select {
    fn from_jongseong_index(index: u32) -> Select {
        match index {
            21 /* ㅇ */ => Select::Queue,
            27 /* ㅎ */ => Select::Channel,
            _ => Select::Stack(index as u8)
        }
    }
}

#[cfg(feature = "render")]
impl Render for Select {
    fn render(&self, prefix: &str) -> String {
        match self {
            Select::Stack(id) => format!("{}Select::Stack({})", prefix, id),
            Select::Queue => format!("{}Select::Queue", prefix),
            Select::Channel => format!("{}Select::Channel", prefix),
        }
    }
}

#[cfg(feature = "parse")]
#[derive(Debug)]
pub struct OwnedCode {
    pub index: Vec<usize>,
    pub code: Vec<Inst>,
}

#[cfg(feature = "parse")]
impl OwnedCode {
    pub fn parse_lines<'a, I: Iterator<Item = &'a str>>(lines: I) -> OwnedCode {
        let mut result = OwnedCode {
            index: Vec::new(),
            code: Vec::new(),
        };

        for line in lines {
            result.index.push(result.code.len());
            for char in line.chars() {
                result.code.push(Inst::from(char));
            }
        }
        result.index.push(result.code.len());
        result
    }

    pub fn parse(code: &str) -> OwnedCode {
        OwnedCode::parse_lines(code.lines())
    }

    #[cfg(feature = "render")]
    pub fn render_as_borrowed(&self, prefix: &str) -> String {
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

#[cfg(feature = "parse")]
impl<'a> From<&'a OwnedCode> for BorrowedCode<'a> {
    fn from(owned: &'a OwnedCode) -> BorrowedCode<'a> {
        BorrowedCode {
            index: &owned.index,
            code: &owned.code,
        }
    }
}

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

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
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

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct Address {
    pub row: i32,
    pub col: i32,
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
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

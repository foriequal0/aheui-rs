use crate::inst::{CursorControl, Inst, Oper, Select};
use crate::vm::BorrowedCode;

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
            9 /* ㅅ */ => Oper::Select(select_from_jongseong_index(last)),
            10 /* ㅆ */ => Oper::Move(select_from_jongseong_index(last)),
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

fn select_from_jongseong_index(index: u32) -> Select {
    match index {
        21 /* ㅇ */ => Select::Queue,
        27 /* ㅎ */ => Select::Channel,
        _ => Select::Stack(index as u8)
    }
}

#[derive(Debug)]
pub struct OwnedCode {
    pub index: Vec<usize>,
    pub code: Vec<Inst>,
}

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
}

impl<'a> From<&'a OwnedCode> for BorrowedCode<'a> {
    fn from(owned: &'a OwnedCode) -> BorrowedCode<'a> {
        BorrowedCode {
            index: &owned.index,
            code: &owned.code,
        }
    }
}

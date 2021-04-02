#[derive(Copy, Clone)]
#[cfg_attr(feature = "derive_debug", derive(Debug))]
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

#[derive(Copy, Clone)]
#[cfg_attr(feature = "derive_debug", derive(Debug))]
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

#[derive(Copy, Clone)]
#[cfg_attr(feature = "derive_debug", derive(Debug))]
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

#[derive(Copy, Clone)]
#[cfg_attr(feature = "derive_debug", derive(Debug))]
pub enum Select {
    Stack(u8),
    Queue,
    Channel,
}

use crate::inst::Oper;
use crate::vm::{Engine, Env};

pub struct Interpreter;

impl Engine for Interpreter {
    fn step(&mut self, env: &mut Env) -> Option<i32> {
        let inst = env.code.get_inst(env.cursor.address).unwrap();
        let mut reverse = false;
        match inst.oper {
            Oper::Nop => {}
            Oper::Halt => return Some(env.storages.selected().try_pop().unwrap_or(0)),
            Oper::Add => {
                if let Some((a, b)) = env.storages.selected().binary_op_assign() {
                    *b = b.wrapping_add(a);
                } else {
                    reverse = true;
                }
            }
            Oper::Mul => {
                if let Some((a, b)) = env.storages.selected().binary_op_assign() {
                    *b = b.wrapping_mul(a);
                } else {
                    reverse = true;
                }
            }
            Oper::Sub => {
                if let Some((a, b)) = env.storages.selected().binary_op_assign() {
                    *b = b.wrapping_sub(a);
                } else {
                    reverse = true;
                }
            }
            Oper::Div => {
                if let Some((a, b)) = env.storages.selected().binary_op_assign() {
                    *b = b.wrapping_div(a);
                } else {
                    reverse = true;
                }
            }
            Oper::Mod => {
                if let Some((a, b)) = env.storages.selected().binary_op_assign() {
                    *b = b.wrapping_rem(a);
                } else {
                    reverse = true;
                }
            }
            Oper::WriteChar => {
                if let Some(value) = env.storages.selected().try_pop() {
                    write!(env.output, "{}", std::char::from_u32(value as u32).unwrap()).unwrap();
                } else {
                    reverse = true;
                }
            }
            Oper::WriteInt => {
                if let Some(value) = env.storages.selected().try_pop() {
                    write!(env.output, "{}", value).unwrap();
                } else {
                    reverse = true;
                }
            }
            Oper::Pop => {
                if env.storages.selected().try_pop().is_none() {
                    reverse = true;
                }
            }
            Oper::ReadChar => {
                let value = env.read_char();
                env.storages.selected().push(value);
            }
            Oper::ReadInt => {
                let value = env.read_int();
                env.storages.selected().push(value);
            }
            Oper::Push(v) => env.storages.selected().push(v as i32),
            Oper::Dup => {
                reverse = !env.storages.selected().dup();
            }
            Oper::Swap => {
                reverse = !env.storages.selected().swap();
            }
            Oper::Select(select) => env.storages.select(select),
            Oper::Move(select) => {
                if let Some(value) = env.storages.selected().try_pop() {
                    env.storages.get_storage(select).push(value);
                } else {
                    reverse = true;
                }
            }
            Oper::Compare => {
                if let Some((a, b)) = env.storages.selected().binary_op_assign() {
                    *b = (a <= *b) as i32;
                } else {
                    reverse = true;
                };
            }
            Oper::Cond => match env.storages.selected().try_pop() {
                Some(value) if value != 0 => {}
                _ => reverse = true,
            },
        };

        env.cursor.advance(&env.code, inst.cursor_control, reverse);
        None
    }
}

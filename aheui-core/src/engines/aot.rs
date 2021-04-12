use std::collections::HashMap;

use std::iter::FromIterator;

use crate::engines::Interpreter;
use crate::storage::Storage;
use crate::vm::{Cursor, Engine, Env};
use crate::{Address, Step};

pub trait Precompiled {
    fn execute(&self, env: &mut Env) -> Option<Cursor>;
}

pub struct AOT<'a, P> {
    precompiled: P,
    interpreter: Interpreter<'a>,
}

impl<'a, P> AOT<'a, P>
where
    P: Precompiled,
{
    pub fn new(interpreter: Interpreter<'a>, precompiled: P) -> Self {
        Self {
            precompiled,
            interpreter,
        }
    }
}

impl<'a, P> Engine for AOT<'a, P>
where
    P: Precompiled,
{
    fn step(&self, env: &mut Env) -> Option<i32> {
        loop {
            if let Some(cursor) = self.precompiled.execute(env) {
                env.cursor = cursor;
                continue;
            }

            if let Some(exitcode) = self.interpreter.step(env) {
                return Some(exitcode);
            }
        }
    }
}

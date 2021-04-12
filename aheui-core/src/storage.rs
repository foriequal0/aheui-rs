use std::collections::VecDeque;

use crate::inst::Select;

pub struct StorageSelector {
    select: Select,
    stacks: [Stack; 28],
    queue: Queue,
}

impl StorageSelector {
    pub fn new() -> Self {
        Self {
            select: Select::Stack(0),
            stacks: Default::default(),
            queue: Default::default(),
        }
    }

    pub fn select(&mut self, select: Select) {
        self.select = select;
    }

    pub fn selected(&mut self) -> &mut dyn Storage {
        self.get_storage(self.select)
    }

    pub fn get_storage(&mut self, select: Select) -> &mut dyn Storage {
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

pub trait Storage {
    fn len(&self) -> usize;
    fn push(&mut self, value: i32);
    fn try_pop(&mut self) -> Option<i32>;
    fn pop(&mut self) -> i32;
    fn binary_op_assign(&mut self) -> Option<(i32, &mut i32)>;
    fn swap(&mut self) -> bool;
    fn dup(&mut self) -> bool;
}

#[derive(Default)]
struct Stack {
    stack: Vec<i32>,
}

impl Storage for Stack {
    fn len(&self) -> usize {
        self.stack.len()
    }

    fn push(&mut self, value: i32) {
        self.stack.push(value)
    }

    fn try_pop(&mut self) -> Option<i32> {
        self.stack.pop()
    }

    fn pop(&mut self) -> i32 {
        self.stack.pop().unwrap()
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

#[derive(Default)]
struct Queue {
    queue: VecDeque<i32>,
}

impl Storage for Queue {
    fn len(&self) -> usize {
        self.queue.len()
    }

    fn push(&mut self, value: i32) {
        self.queue.push_back(value)
    }

    fn try_pop(&mut self) -> Option<i32> {
        self.queue.pop_front()
    }

    fn pop(&mut self) -> i32 {
        self.queue.pop_front().unwrap()
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

use std::sync::{Mutex, Arc};
use std::collections::VecDeque;
use std::thread;

pub struct CircularBuffer<T> {
    buffer: Mutex<VecDeque<T>>,
    capacity: usize,
}

impl<T> CircularBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        CircularBuffer {
            buffer: Mutex::new(VecDeque::new()),
            capacity,
        }
    }

    pub fn push(&self, item: T) {
        let mut buffer = self.buffer.lock().unwrap();
        if buffer.len() >= self.capacity {
            //println!("lose item");
            buffer.pop_front();
        }
        buffer.push_back(item);
    }

    pub fn pop(&self) -> Option<T> {
        let mut buffer = self.buffer.lock().unwrap();
        buffer.pop_front()
    }

    pub fn size(&self) -> usize {
        let buffer = self.buffer.lock().unwrap();
        buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        let buffer = self.buffer.lock().unwrap();
        buffer.is_empty()
    }

    pub fn clear(&self) {
        let mut buffer = self.buffer.lock().unwrap();
        buffer.clear();
    }
}

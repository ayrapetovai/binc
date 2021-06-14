use crate::number::Number;
use std::collections::VecDeque;

#[derive(Debug)]
pub struct History {
    backward_list: VecDeque<Number>,
    forward_list: VecDeque<Number>,
    max_size: usize,
}

impl History {
    pub fn new(max_size: usize) -> Self {
        History {
            backward_list: VecDeque::new(),
            forward_list: VecDeque::new(),
            max_size,
        }
    }

    pub fn save(&mut self, number: &Number) {
        if self.backward_list.len() + 1 > self.max_size {
            self.backward_list.pop_front();
        }
        self.backward_list.push_back(number.clone());
        if !self.forward_list.is_empty() {
            self.forward_list.clear();
        }
    }

    pub fn backward(&mut self) -> Number {
        if self.backward_list.len() == 1 {
            self.backward_list.back().unwrap().clone()
        } else {
            let number = self.backward_list.pop_back().unwrap();
            self.forward_list.push_back(number);
            self.backward_list.back().unwrap().clone()
        }
    }

    pub fn forward(&mut self) -> Number {
        if self.forward_list.len() == 0 {
            self.backward_list.back().unwrap().clone()
        } else {
            let number = self.forward_list.pop_back().unwrap();
            self.backward_list.push_back(number.clone());
            number
        }
    }
}
use lossyq::spsc::{noloss, Sender, Receiver};
use super::super::common::{Task, Message};
use super::super::elem::gather::Gather;
use super::super::common::Schedule;
use std::collections::VecDeque;
use std::mem;

pub struct Collector {
  overflow: VecDeque<Message<Box<Task + Send>>>,
}

impl noloss::Overflow for Collector {
  type Input = Message<Box<Task + Send>>;

  fn overflow(&mut self, val : &mut Option<Self::Input>) {
    let mut tmp : Option<Self::Input> = None;
    mem::swap(&mut tmp, val);
    match tmp {
      Some(v) => {
        self.overflow.push_back(v);
      },
      None => {}
    }
  }
}

impl Gather for Collector {
  type InputType  = Box<Task + Send>;
  type OutputType = Box<Task + Send>;

  fn process(
          &mut self,
          input_vec:   Vec<&mut Receiver<Message<Self::InputType>>>,
          output:      &mut Sender<Message<Self::OutputType>>) -> Schedule {

    {
      let mut tmp_overflow = Collector { overflow: VecDeque::new() };

      // process the previously overflown items
      loop {
        match self.overflow.pop_front() {
          Some(item) => {
            let mut opt_item : Option<Message<Self::InputType>> = Some(item);
            match noloss::pour(&mut opt_item, output, &mut tmp_overflow) {
              (noloss::PourResult::Overflowed, _) => { break; }
              _ => {}
            }
          },
          None => { break; }
        }
      }

      // process the incoming items
      for input in input_vec {
        for item in input.iter() {
          let mut opt_item : Option<Message<Self::InputType>> = Some(item);
          match noloss::pour(&mut opt_item, output, &mut tmp_overflow) {
            (noloss::PourResult::Overflowed, _) => { break; }
            _ => {}
          }
        }
      }

      // move the newly overflown items in
      self.overflow.append(&mut tmp_overflow.overflow);
    }

    Schedule::Loop
  }
}

pub fn new() -> Collector {
  Collector {
    overflow: VecDeque::new()
  }
}

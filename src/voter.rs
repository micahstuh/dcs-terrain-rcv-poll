use crate::candidate::Candidate;
use std::rc::Rc;

pub struct Voter {
    pub name: String,
    pub votes: Vec<Rc<Candidate>>,
}

impl Voter {
    pub fn new(name: String, votes: Vec<Rc<Candidate>>) -> Voter {
        Voter {
            name,
            votes,
        }
    }

    pub fn vote(&mut self, candidate: Rc<Candidate>) {
        self.votes.push(candidate);
    }
}
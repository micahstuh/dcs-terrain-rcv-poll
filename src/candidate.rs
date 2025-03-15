use std::clone;

pub struct Candidate {
    pub name: String,
}

impl Candidate {
    pub fn new(name: &str) -> Candidate {
        Candidate {
            name: name.to_string(),
        }
    }
}

impl clone::Clone for Candidate {
    fn clone(&self) -> Candidate {
        Candidate {
            name: self.name.clone(),
        }
    }
}

impl PartialEq for Candidate {
    fn eq(&self, other: &Candidate) -> bool {
        self.name == other.name
    }
}
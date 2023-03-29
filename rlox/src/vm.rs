use std::fmt;

#[repr(u8)]
#[derive(Debug)]
pub enum Op {
    Return
}

pub struct Chunk {
    code: Vec<Op>,
    name: String
}

impl Chunk {
    pub fn new(name: String) -> Chunk {
        Chunk {
            code: Vec::new(),
            name
        }
    }

    pub fn add(&mut self, op: Op) {
        self.code.push(op)
    }
}

impl fmt::Debug for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        writeln!(f, "-*-*- {} -*-*-", self.name)?;
        for (idx, op) in self.code.iter().enumerate() {
            write!(f, "{:04}  {:?}", idx, op)?;
        }

        return Ok(());
    }
}

use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub enum EntryFlag {
    Missing,
    Exact,
    Beta,
    Alpha,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct TTEntry {
    pub depth: usize,
    pub eval: i32,
    pub flag: EntryFlag,
}

pub struct TranspositionTable {
    pub tt: Box<[TTEntry]>, //using array = stack overflow
    pub size: usize,
    pub mask: usize,
}

/*
* TT implementation inspired by engine Black Marlin
* the idea of the mask field is that it will be set to
* (2^k - 1), where the capacity of the table = 2^k.
* This means that the mask will have all bits less
* significant than the kth bit set and all others zeroed.
* Mask acts as a filter so that u64 hash key can get
* indexed in a hash table of some size.
*
* Somehow using a hashmap with default capacity still gives
* higher NPS for me tho...
*/

impl Default for TTEntry {
    fn default() -> Self {
        Self {
            depth: 0,
            eval: 0,
            flag: EntryFlag::Missing,
        }
    }
}

impl TTEntry {
    pub fn new(depth: usize, eval: i32, flag: EntryFlag) -> Self {
        Self { depth, eval, flag }
    }

    pub fn zero(&mut self) {
        self.depth = 0;
        self.eval = 0;
        self.flag = EntryFlag::Missing;
    }
}

pub trait TT {
    fn lookup(&self, key: u64, alpha: i32, beta: i32, depth: usize) -> Option<i32>;
    fn write(&mut self, hash: u64, entry: TTEntry);
}

impl TranspositionTable {
    pub fn new_from_n(n: usize) -> Self {
        //creates TT of capacity 2^n
        let capacity: usize = 1 << n;
        // ensure that capacity is a power of two
        TranspositionTable {
            tt: (0..capacity)
                .map(|_| TTEntry::default())
                .collect::<Box<_>>(),
            size: capacity,
            mask: capacity - 1,
        }
    }

    pub fn index(&self, hash_key: u64) -> usize {
        //return index to allocate in table
        (hash_key as usize) & self.mask
    }

    pub fn get(&self, hash: u64) -> Option<TTEntry> {
        let index = self.index(hash);
        let entry = self.tt[index];
        match entry.flag {
            EntryFlag::Missing => None,
            _ => Some(entry),
        }
    }

    pub fn clear(&mut self) {
        self.tt.iter_mut().for_each(|entry| entry.zero());
    }
}

impl TT for TranspositionTable {
    fn lookup(&self, hash_key: u64, alpha: i32, beta: i32, depth: usize) -> Option<i32> {
        if let Some(entry) = self.get(hash_key) {
            if entry.depth >= depth {
                match entry.flag {
                    EntryFlag::Beta => {
                        //lower bound hash entry
                        if entry.eval >= beta {
                            return Some(entry.eval);
                        }
                    }
                    EntryFlag::Alpha => {
                        //upper bound entry
                        if entry.eval <= alpha {
                            return Some(entry.eval);
                        }
                    }
                    EntryFlag::Exact => return Some(entry.eval),
                    //pv entry
                    EntryFlag::Missing => unreachable!(),
                    //as above, the get() function will return none
                    //if the entry is missing
                }
            }
        }
        None
    }

    fn write(&mut self, hash: u64, entry: TTEntry) {
        let index = self.index(hash);
        self.tt[index] = entry;
    }
}

impl TT for HashMap<u64, TTEntry> {
    fn lookup(&self, key: u64, alpha: i32, beta: i32, depth: usize) -> Option<i32> {
        if let Some(entry) = self.get(&key) {
            if entry.depth >= depth {
                match entry.flag {
                    EntryFlag::Beta => {
                        //lower bound hash entry
                        if entry.eval >= beta {
                            return Some(entry.eval);
                        }
                    }
                    EntryFlag::Alpha => {
                        //upper bound entry
                        if entry.eval <= alpha {
                            return Some(entry.eval);
                        }
                    }
                    EntryFlag::Exact => return Some(entry.eval),
                    //pv entry
                    EntryFlag::Missing => unreachable!(),
                    //as above, the get() function will return none
                    //if the entry is missing
                }
            }
        }
        None
    }

    fn write(&mut self, hash: u64, entry: TTEntry) {
        self.insert(hash, entry);
    }
}

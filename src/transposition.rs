use crate::{Move, NULL_MOVE};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub enum EntryFlag {
    Missing,
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct TTEntry {
    pub depth: u8,       //8b
    pub eval: i32,       //32b
    pub flag: EntryFlag, //8b
    pub best_move: Move, //16b
    pub hash_key: u64,
}

pub struct TranspositionTable {
    pub tt: Vec<TTEntry>,
    pub size: usize,
    pub mask: usize,
}

impl TTEntry {
    pub fn new(depth: u8, eval: i32, flag: EntryFlag, best_move: Move, hash_key: u64) -> Self {
        Self {
            depth,
            eval,
            flag,
            best_move,
            hash_key,
        }
    }

    pub fn zero(&mut self) {
        self.depth = 0;
        self.eval = 0;
        self.flag = EntryFlag::Missing;
    }

    fn empty() -> Self {
        Self {
            depth: 0,
            eval: 0,
            flag: EntryFlag::Missing,
            best_move: NULL_MOVE,
            hash_key: 0,
        }
    }
}

pub trait TT {
    fn lookup(&self, key: u64) -> Option<TTEntry>;
    fn write(&mut self, hash: u64, entry: TTEntry);
}

impl TranspositionTable {
    pub fn log2_capacity(n: usize) -> Self {
        let capacity: usize = 1 << n;
        TranspositionTable {
            tt: (0..capacity).map(|_| TTEntry::empty()).collect::<Vec<_>>(),
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
    fn lookup(&self, hash_key: u64) -> Option<TTEntry> {
        if let Some(entry) = self.get(hash_key) {
            if entry.hash_key == hash_key {
                Some(entry)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn write(&mut self, hash: u64, entry: TTEntry) {
        let index = self.index(hash);
        self.tt[index] = entry;
    }
}

impl TT for HashMap<u64, TTEntry> {
    fn lookup(&self, hash_key: u64) -> Option<TTEntry> {
        self.get(&hash_key).copied()
    }

    fn write(&mut self, hash: u64, entry: TTEntry) {
        self.insert(hash, entry);
    }
}

use crate::{Move, NULL_MOVE};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EntryFlag {
    Missing,
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct TTEntry {
    pub depth: usize,
    pub eval: i32,
    pub flag: EntryFlag,
    pub best_move: Move,
}

// TT implementation inspired by engine Black Marlin
// the idea of the mask field is that it will be set to
// (2^k - 1), where the capacity of the table = 2^k.
// This means that the mask will have all bits less
// significant than the kth bit set and all others zeroed.
// Mask acts as a filter so that u64 hash key can get
// indexed in a hash table of some size.

// Somehow using a hashmap with default capacity still gives
// higher NPS for me tho...
pub struct TranspositionTable {
    pub tt: Box<[TTEntry]>, //using array = stack overflow
    pub size: usize,
    pub mask: usize,
}

pub struct LookupResult {
    pub eval: Option<i32>,
    pub best_move: Move,
    pub depth: usize,
    pub flag: EntryFlag,
}

impl Default for TTEntry {
    fn default() -> Self {
        Self {
            depth: 0,
            eval: 0,
            flag: EntryFlag::Missing,
            best_move: NULL_MOVE,
        }
    }
}

impl TTEntry {
    pub fn new(depth: usize, eval: i32, flag: EntryFlag, best_move: Move) -> Self {
        Self {
            depth,
            eval,
            flag,
            best_move,
        }
    }

    pub fn zero(&mut self) {
        self.depth = 0;
        self.eval = 0;
        self.flag = EntryFlag::Missing;
    }
}

pub trait TT {
    fn lookup(
        &self,
        key: u64,
        alpha: i32,
        beta: i32,
        depth: usize,
        search_tt_score: &mut i32,
    ) -> LookupResult;
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
    fn lookup(
        &self,
        hash_key: u64,
        alpha: i32,
        beta: i32,
        depth: usize,
        search_tt_score: &mut i32,
    ) -> LookupResult {
        if let Some(entry) = self.get(hash_key) {
            *search_tt_score = entry.eval;
            if entry.depth >= depth {
                match entry.flag {
                    EntryFlag::LowerBound => {
                        //lower bound hash entry
                        if entry.eval >= beta {
                            return LookupResult {
                                eval: Some(entry.eval),
                                best_move: entry.best_move,
                                depth: entry.depth,
                                flag: EntryFlag::LowerBound,
                            };
                        }
                    }
                    EntryFlag::UpperBound => {
                        //upper bound entry
                        if entry.eval <= alpha {
                            return LookupResult {
                                eval: Some(entry.eval),
                                best_move: entry.best_move,
                                depth: entry.depth,
                                flag: EntryFlag::UpperBound,
                            };
                        }
                    }
                    EntryFlag::Exact => {
                        return LookupResult {
                            eval: Some(entry.eval),
                            best_move: entry.best_move,
                            depth: entry.depth,
                            flag: EntryFlag::Exact,
                        };
                    }
                    //pv entry
                    EntryFlag::Missing => unreachable!(),
                    //as above, the get() function will return none
                    //if the entry is missing
                }

                return LookupResult {
                    eval: None,
                    best_move: entry.best_move,
                    depth: entry.depth,
                    flag: entry.flag,
                };
            } else {
                return LookupResult {
                    eval: None,
                    best_move: entry.best_move,
                    depth: entry.depth,
                    flag: entry.flag,
                };
            }
        }
        LookupResult {
            eval: None,
            best_move: NULL_MOVE,
            depth: 0,
            flag: EntryFlag::Missing,
        }
    }

    fn write(&mut self, hash: u64, entry: TTEntry) {
        let index = self.index(hash);
        self.tt[index] = entry;
    }
}

impl TT for HashMap<u64, TTEntry> {
    fn lookup(
        &self,
        key: u64,
        alpha: i32,
        beta: i32,
        depth: usize,
        search_tt_score: &mut i32,
    ) -> LookupResult {
        if let Some(entry) = self.get(&key) {
            *search_tt_score = entry.eval;
            if entry.depth >= depth {
                match entry.flag {
                    EntryFlag::LowerBound => {
                        //lower bound hash entry
                        if entry.eval >= beta {
                            return LookupResult {
                                eval: Some(entry.eval),
                                best_move: entry.best_move,
                                depth: entry.depth,
                                flag: EntryFlag::LowerBound,
                            };
                        }
                    }
                    EntryFlag::UpperBound => {
                        //upper bound entry
                        if entry.eval <= alpha {
                            return LookupResult {
                                eval: Some(entry.eval),
                                best_move: entry.best_move,
                                depth: entry.depth,
                                flag: EntryFlag::UpperBound,
                            };
                        }
                    }
                    EntryFlag::Exact => {
                        return LookupResult {
                            eval: Some(entry.eval),
                            best_move: entry.best_move,
                            depth: entry.depth,
                            flag: EntryFlag::Exact,
                        };
                    }
                    //pv entry
                    EntryFlag::Missing => unreachable!(),
                    //as above, the get() function will return none
                    //if the entry is missing
                }

                //even if we can't return an evaluation we should still return the best move
                //for the purpose of move-ordering
                return LookupResult {
                    eval: None,
                    best_move: entry.best_move,
                    depth: entry.depth,
                    flag: entry.flag,
                };
            } else {
                //return best move even in the case that we cannot rely on the evaluation for the
                //purpose of move ordering
                return LookupResult {
                    eval: None,
                    best_move: entry.best_move,
                    depth: entry.depth,
                    flag: EntryFlag::Missing,
                };
            }
        }

        LookupResult {
            eval: None,
            best_move: NULL_MOVE,
            depth: 0,
            flag: EntryFlag::Missing,
        }
    }

    fn write(&mut self, hash: u64, entry: TTEntry) {
        self.insert(hash, entry);
    }
}

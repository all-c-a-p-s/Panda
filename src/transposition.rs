use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

use crate::Move;

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum EntryFlag {
    Missing,
    Exact,
    LowerBound,
    UpperBound,
}

pub struct TTRef<'a> {
    tt: &'a TranspositionTable,
}

impl std::ops::Deref for TTRef<'_> {
    type Target = TranspositionTable;
    fn deref(&self) -> &Self::Target {
        self.tt
    }
}

impl<'a> TTRef<'a> {
    #[must_use]
    pub fn new(tt: &'a TranspositionTable) -> Self {
        Self { tt }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(C)]
pub struct TTEntry {
    pub hash_key: u64,   //64b
    pub eval: i32,       //32b
    pub best_move: Move, //16b
    pub depth: u8,       //8b
    pub flag: EntryFlag, //8b = 128b total
}

#[derive(Default)]
pub struct TTEntryInternal {
    data: AtomicU64,
    key: AtomicU64,
}

impl Clone for TTEntryInternal {
    fn clone(&self) -> Self {
        Self {
            data: AtomicU64::new(self.data.load(Relaxed)),
            key: AtomicU64::new(self.key.load(Relaxed)),
        }
    }
}

impl TTEntryInternal {
    pub fn zero(&self) {
        self.data.store(0, Relaxed);
        self.key.store(0, Relaxed);
    }
}

pub struct TranspositionTable {
    pub tt: Vec<TTEntryInternal>,
    pub size: usize,
    pub mask: usize,
}

impl TTEntry {
    const BEST_MOVE_SHIFT: u64 = 32;
    const DEPTH_SHIFT: u64 = 48;
    const FLAG_SHIFT: u64 = 56;
    #[must_use]
    pub fn new(depth: u8, eval: i32, flag: EntryFlag, best_move: Move, hash_key: u64) -> Self {
        Self {
            hash_key,
            eval,
            best_move,
            depth,
            flag,
        }
    }

    fn from_internal(x: TTEntryInternal) -> TTEntry {
        unsafe {
            let bytes: [u64; 2] = [x.key.load(Relaxed), x.data.load(Relaxed)];
            std::mem::transmute(bytes)
        }
    }

    #[allow(clippy::wrong_self_convention)]
    fn to_u64s(&self) -> (u64, u64) {
        let key = self.hash_key;

        let eval_data = u64::from(unsafe { std::mem::transmute::<i32, u32>(self.eval) });
        let bm_data = u64::from(self.best_move.data) << Self::BEST_MOVE_SHIFT;
        let depth_data = u64::from(self.depth) << Self::DEPTH_SHIFT;
        let flag_data = u64::from(self.flag as u8) << Self::FLAG_SHIFT;

        let data = eval_data | bm_data | depth_data | flag_data;

        (data, key)
    }
}

pub trait TT {
    fn lookup(&self, key: u64) -> Option<TTEntry>;
    fn write(&self, hash: u64, entry: TTEntry);
}

impl TranspositionTable {
    #[must_use]
    pub fn with_log2_capacity(n: usize) -> Self {
        let capacity: usize = 1 << n;
        TranspositionTable {
            tt: (0..capacity)
                .map(|_| TTEntryInternal::default())
                .collect::<Vec<_>>(),
            size: capacity,
            mask: capacity - 1,
        }
    }

    #[must_use]
    pub fn in_megabytes(n: usize) -> Self {
        let mbs = n.max(1);
        let x = (mbs as f32).log2() as usize;

        Self::with_log2_capacity(16 + x)
    }

    #[must_use]
    pub fn index(&self, hash_key: u64) -> usize {
        //return index to allocate in table
        (hash_key as usize) & self.mask
    }

    #[must_use]
    pub fn get(&self, hash: u64) -> Option<TTEntry> {
        let index = self.index(hash);
        let entry = TTEntry::from_internal(self.tt[index].clone());
        match entry.flag {
            EntryFlag::Missing => None,
            _ => Some(entry),
        }
    }

    pub fn clear(&self) {
        self.tt.iter().for_each(TTEntryInternal::zero);
    }

    pub fn resize(&mut self, mbs: usize) {
        let x = (mbs.max(1) as f32).log2() as usize;
        self.size = 1 << (x + 16);
        self.mask = self.size - 1;
        self.tt.resize_with(self.size, TTEntryInternal::default);
    }
}

impl TT for TranspositionTable {
    fn lookup(&self, hash_key: u64) -> Option<TTEntry> {
        self.get(hash_key)
            .filter(|&entry| entry.hash_key == hash_key)
    }

    fn write(&self, hash: u64, entry: TTEntry) {
        let index = self.index(hash);
        let (d, k) = entry.to_u64s();
        self.tt[index].data.store(d, Relaxed);
        self.tt[index].key.store(k, Relaxed);
    }
}

#[cfg(test)]
mod tests {
    macro_rules! entryt {
        ($fen: expr, $eval: expr, $mv: expr, $depth: expr, $flag: expr, $idx: expr) => {
            let b = crate::Board::from($fen);
            let h = b.hash_key;
            let m = crate::uci::parse_move($mv, &b);
            let entry = TTEntry::new($depth, $eval, $flag, m, h);

            let internal = TTEntryInternal::default();
            let (d, k) = entry.to_u64s();
            internal.data.store(d, Relaxed);
            internal.key.store(k, Relaxed);
            let entry2 = TTEntry::from_internal(internal);

            assert_eq!(entry, entry2);

            println!("Entryt Position {}: Passed", $idx);
        };
    }

    use super::*;
    use crate::INFINITY;

    #[rustfmt::skip]
    #[test]
    pub fn entry_convesion_test() {
        entryt!("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 20, "e2e4", 0, EntryFlag::Exact, 1);
        entryt!("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 20, "e2e4", 0, EntryFlag::LowerBound, 2);
        entryt!("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 20, "e2e4", 0, EntryFlag::UpperBound, 3);
        entryt!("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 10, "d2d4", 12, EntryFlag::Exact, 4);
        entryt!("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 10, "d2d4", 12, EntryFlag::LowerBound, 5);
        entryt!("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 10, "d2d4", 12, EntryFlag::UpperBound, 6);
        entryt!("r3kb1r/3b1ppp/pq2p3/3pP3/3P4/5N2/PP3PPP/RN1QK2R w KQkq - 0 13", 0, "d1b3", 8, EntryFlag::Exact, 7);
        entryt!("r3kb1r/3b1ppp/pq2p3/3pP3/3P4/5N2/PP3PPP/RN1QK2R w KQkq - 0 13", 0, "d1b3", 8, EntryFlag::LowerBound, 8);
        entryt!("r3kb1r/3b1ppp/pq2p3/3pP3/3P4/5N2/PP3PPP/RN1QK2R w KQkq - 0 13", 0, "d1b3", 8, EntryFlag::UpperBound, 9);
        entryt!("r1b1k1nr/pppp1ppp/2n5/4P3/8/2Q2N2/PqP1PPPP/RN2KB1R b KQkq - 0 8", INFINITY - 1, "b2c1", 1, EntryFlag::Exact, 10);
    }
}

use std::sync::atomic::{AtomicBool, AtomicU64};
use std::time::{Duration, Instant};

use crate::eval::Accumulator;
use crate::read_param;
use crate::search::params;
use crate::search::transposition::{TTRef, TranspositionTable};
use crate::util::types::{Piece, Square};
use crate::{Board, INFINITY, MAX_DEPTH, Move, MoveData, NULL_MOVE, iterative_deepening};

const MIN_MOVE_TIME: usize = 1; //make sure move time is never 0
const MOVE_OVERHEAD: usize = 50;

pub const CORRHIST_SIZE: usize = 16_384;

//returns ideal time window, hard deadline
#[must_use]
pub fn move_time(time: usize, increment: usize, moves_to_go: usize) -> (usize, usize) {
    if time < MOVE_OVERHEAD {
        return ((time / 2).min(MIN_MOVE_TIME), (time / 2).min(MIN_MOVE_TIME));
    }

    let time_until_flag = time - MOVE_OVERHEAD;

    let m = if moves_to_go == 0 { read_param!(TMAN_DEFAULT_MTG) } else { moves_to_go.clamp(2, 40) };

    //note time - increment must be +ve since we got increment last turn
    let average_move_time = time_until_flag / m; // I guess this ignores increment so variable
    // name is a lie
    let ideal_time = average_move_time * read_param!(TMAN_IDEAL_MULT) / 1024 + increment / 2;

    let t = ideal_time.min(time_until_flag);

    let max_time = (2 * t).min((time_until_flag * 3) / 5);

    (t.max(MIN_MOVE_TIME), max_time.max(MIN_MOVE_TIME))
}

#[derive(Copy, Clone)]
pub struct SearchStackEntry {
    pub square_moved_to: Option<Square>,
    pub piece_moved: Option<Piece>,
    pub made_capture: bool,
    pub eval: i32,
}

#[derive(Copy, Clone)]
pub struct SearchInfo {
    pub ss: [SearchStackEntry; MAX_DEPTH],
    pub lmr_table: LMRTable,
    pub nodetable: NodeTable,
    pub piece_history: [[i32; 64]; 12],
    pub square_history: [[[i32; 64]; 64]; 2],
    pub caphist: [[[i32; 5]; 64]; 12],

    pub counter_correlation: [[[i32; 64]; 64]; 2],
    pub followup_correlation: [[[i32; 64]; 64]; 2],

    pub stck: AccumulatorStack,

    pub pawn_corrhist: [[i32; CORRHIST_SIZE]; 2],
    pub knb_corrhist: [[i32; CORRHIST_SIZE]; 2],
    pub krq_corrhist: [[i32; CORRHIST_SIZE]; 2],

    pub killer_moves: [Option<Move>; MAX_DEPTH],
    pub counter_moves: [[Option<Move>; 64]; 12],
    pub excluded: [Option<Move>; MAX_DEPTH],
}

#[derive(Clone, Copy)]
pub struct AccumulatorStack {
    pub accs: [Accumulator; MAX_DEPTH + 1],
    pub idx: usize,
}

impl Default for AccumulatorStack {
    fn default() -> Self {
        let mut accs = [Accumulator::default(); MAX_DEPTH + 1];
        accs[0] = Accumulator::from_startpos();

        Self { accs, idx: 0 }
    }
}

impl AccumulatorStack {
    pub fn partial_push(&mut self) {
        self.accs[self.idx + 1] = self.accs[self.idx];
        self.idx += 1;
    }

    pub fn pop(&mut self) {
        self.idx -= 1;
    }

    /// This function is a bit hacky. Sometimes (often in the uci file), we need to successively
    /// update the root accumulator. The easiest way to do this is just to make the move on the
    /// board normally (which puts it at index 1) and then call this function, which gives the
    /// desired behaviour.
    ///
    /// This should generally be called for any move that gets made on the board
    /// and won't be undone (i.e. not in some type of search).
    pub fn bring_to_front(&mut self) {
        self.accs[0] = self.accs[1];
        self.idx = 0;
    }

    pub fn set_to(&mut self, b: &Board) {
        self.accs[0] = Accumulator::from_board(b);
        self.idx = 0;
    }
}

#[derive(Copy, Clone)]
pub struct LMRTable {
    pub reduction_table: [[[i32; 64]; 64]; 2],
}

#[derive(Clone, Copy)]
pub struct NodeTable {
    table: [[usize; 64]; 64],
}

impl NodeTable {
    pub fn add(&mut self, mv: Move, nodes: usize) {
        self.table[mv.square_from()][mv.square_to()] += nodes;
    }

    #[must_use]
    pub fn get(self, mv: Move) -> usize {
        self.table[mv.square_from()][mv.square_to()]
    }
}

impl Default for NodeTable {
    fn default() -> Self {
        Self { table: [[0; 64]; 64] }
    }
}

impl Default for SearchStackEntry {
    fn default() -> Self {
        Self { eval: -INFINITY, piece_moved: None, square_moved_to: None, made_capture: false }
    }
}

impl Default for LMRTable {
    fn default() -> Self {
        let tb = read_param!(LMR_TACTICAL_BASE) as f64 / 100.0;
        let td = read_param!(LMR_TACTICAL_DIVISOR) as f64 / 100.0;
        let qb = read_param!(LMR_QUIET_BASE) as f64 / 100.0;
        let qd = read_param!(LMR_QUIET_DIVISOR) as f64 / 100.0;
        let mut reduction_table = [[[0; 64]; 64]; 2];

        #[allow(clippy::needless_range_loop)]
        for depth in 0..64 {
            for played in 0..64 {
                reduction_table[0][depth][played] =
                    (1024.0 * (tb + (depth as f64).ln() * (played as f64).ln() / td)) as i32;
                //tactical move
                reduction_table[1][depth][played] =
                    (1024.0 * (qb + (depth as f64).ln() * (played as f64).ln() / qd)) as i32;
                //quiet move
            }
        }

        LMRTable { reduction_table }
    }
}

impl Default for SearchInfo {
    fn default() -> Self {
        Self {
            ss: [SearchStackEntry::default(); MAX_DEPTH],
            lmr_table: LMRTable::default(),
            nodetable: NodeTable::default(),
            piece_history: [[0; 64]; 12],
            square_history: [[[0; 64]; 64]; 2],
            caphist: [[[0; 5]; 64]; 12],

            counter_correlation: [[[0; 64]; 64]; 2],
            followup_correlation: [[[0; 64]; 64]; 2],

            stck: AccumulatorStack::default(),

            pawn_corrhist: [[0; CORRHIST_SIZE]; 2],
            knb_corrhist: [[0; CORRHIST_SIZE]; 2],
            krq_corrhist: [[0; CORRHIST_SIZE]; 2],

            killer_moves: [None; 64],
            counter_moves: [[None; 64]; 12],
            excluded: [None; 64],
        }
    }
}

pub struct Thread<'a> {
    pub pv_length: [usize; 64],
    pub pv: [[Move; MAX_DEPTH]; MAX_DEPTH],
    pub tt: TTRef<'a>,
    pub ply: usize,
    pub nodes: usize,
    pub timer: Timer,
    pub stop: &'a AtomicBool,
    pub moves_fully_searched: usize,
    pub do_pruning: bool,
    pub info: &'a mut SearchInfo,
    pub double_extensions: u8,
    pub seldepth: u8,
}

pub struct Timer {
    pub max_nodes: usize,
    pub end_time: Instant,
}

impl Default for Timer {
    fn default() -> Self {
        Self { max_nodes: 0, end_time: Instant::now() }
    }
}

impl<'a> Thread<'a> {
    pub fn new(
        end_time: Instant,
        max_nodes: usize,
        tt: &'a TranspositionTable,
        info: &'a mut SearchInfo,
        stop: &'a AtomicBool,
    ) -> Self {
        let timer = Timer { max_nodes, end_time };

        Thread {
            pv_length: [0; MAX_DEPTH],
            pv: [[NULL_MOVE; MAX_DEPTH]; MAX_DEPTH],
            tt: TTRef::new(tt),
            ply: 0,
            nodes: 0,
            timer,
            stop,
            moves_fully_searched: 0,
            do_pruning: true,
            info,
            double_extensions: 0,
            seldepth: 0,
        }
    }
}

pub struct Searcher<'a> {
    _nodecount: AtomicU64,
    tt: &'a TranspositionTable,
    info: &'a mut SearchInfo,
}

#[derive(Default)]
pub struct Limits {
    pub max_nodes: Option<usize>,
    pub max_time: Option<usize>,
    pub max_depth: Option<u8>,
}

impl Limits {
    pub fn depth_only(d: u8) -> Self {
        Self { max_nodes: None, max_time: Some(INFINITY as usize), max_depth: Some(d) }
    }

    pub fn time_only(time: usize) -> Self {
        Self { max_nodes: None, max_time: Some(time), max_depth: None }
    }

    pub fn nodes_only(nodes: usize) -> Self {
        Self { max_nodes: Some(nodes), max_time: Some(INFINITY as usize), max_depth: None }
    }

    pub fn time_and_nodes(time: usize, nodes: usize) -> Self {
        Self { max_nodes: Some(nodes), max_time: Some(time), max_depth: None }
    }
}

impl<'a> Searcher<'a> {
    #[must_use]
    pub fn new(tt: &'a TranspositionTable, info: &'a mut SearchInfo) -> Self {
        Self { _nodecount: AtomicU64::new(0), tt, info }
    }
    // this attribute is for threads variable which is unused in datagen mode
    #[allow(unused, clippy::too_many_arguments)]
    pub fn start_search(
        &mut self,
        position: &mut Board,
        time_left: usize,
        inc: usize,
        moves_to_go: usize,
        limits: &Limits,
        threads: usize,
    ) -> MoveData {
        // Soft-limit vs Hard-limit is an idea explained to me by the author of Sirius
        // Soft limit: if you complete an iteration and the time taken > this, exit
        // Hard limit: if you are currently searching (i.e. in the middle of the tree) and
        //             time taken > this, then exit search
        // in practice you should mostly exit at the soft-limit
        let (soft_limit, hard_limit) = if let Some(k) = limits.max_time {
            if k <= MOVE_OVERHEAD {
                (k, k)
            } else {
                let t = MIN_MOVE_TIME.max(k - MOVE_OVERHEAD);
                (t, t)
            }
        } else {
            move_time(time_left, inc, moves_to_go)
        };

        let max_nodes = if let Some(l) = limits.max_nodes { l } else { i32::MAX as usize };
        let max_depth = if let Some(l) = limits.max_depth { l } else { MAX_DEPTH as u8 };

        let start = Instant::now();
        let end_time = start + Duration::from_millis(hard_limit as u64);

        let stop = AtomicBool::new(false);

        let mut main_thread = Thread::new(end_time, max_nodes, self.tt, self.info, &stop);

        //datagen is already multi-threaded so only search on one thread
        #[cfg(feature = "datagen")]
        {
            return iterative_deepening::<false>(
                &mut position.clone(),
                soft_limit,
                hard_limit,
                max_depth,
                &mut main_thread,
            );
        }

        let mut infos = (0..threads.saturating_sub(1))
            .map(|_| {
                let mut info = SearchInfo::default();
                info.stck.set_to(position);
                info
            })
            .collect::<Vec<_>>();

        #[cfg(not(feature = "datagen"))]
        std::thread::scope(|s| {
            let main_handle = s.spawn(|| {
                iterative_deepening::<true>(&mut position.clone(), soft_limit, hard_limit, max_depth, &mut main_thread)
            });

            for info in infos.iter_mut() {
                let mut pos = *position;
                let mut worker = Thread::new(end_time, max_nodes, self.tt, info, &stop);

                s.spawn(move || iterative_deepening::<false>(&mut pos, soft_limit, hard_limit, max_depth, &mut worker));
            }

            main_handle.join().expect("error in main thread")
        })
    }
}

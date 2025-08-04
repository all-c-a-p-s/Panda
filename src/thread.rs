use std::sync::atomic::{AtomicBool, AtomicU64};
use std::time::{Duration, Instant};
use types::{Piece, Square};

use crate::read_param;
use crate::search::{params, CORRHIST_SIZE};
use crate::transposition::{TTRef, TranspositionTable};
use crate::{iterative_deepening, types, Board, Move, MoveData, INFINITY, MAX_PLY, NULL_MOVE};

const MIN_MOVE_TIME: usize = 1; //make sure move time is never 0
const MOVE_OVERHEAD: usize = 50;

//returns ideal time window, hard deadline
#[must_use]
pub fn move_time(time: usize, increment: usize, moves_to_go: usize, _ply: usize) -> (usize, usize) {
    if time < MOVE_OVERHEAD {
        return (
            std::cmp::max(time / 2, MIN_MOVE_TIME),
            std::cmp::max(time / 2, MIN_MOVE_TIME),
        );
    }

    let time_until_flag = time - MOVE_OVERHEAD;

    let m = if moves_to_go == 0 {
        20 //very low compared to what I've seen but fsr this works best for Panda
    } else {
        moves_to_go.clamp(2, 40)
    };

    //note time - increment must be +ve since we got increment last turn
    let average_move_time = time_until_flag / m; // I guess this ignores increment so variable
                                                 // name is a lie
    let ideal_time = (average_move_time * 7) / 10 + increment / 2;
    let t = std::cmp::min(ideal_time, time_until_flag);

    let max_time = std::cmp::min(2 * t, (time_until_flag * 3) / 5);

    (std::cmp::max(t, MIN_MOVE_TIME), max_time)
}

#[derive(Copy, Clone)]
pub struct SearchStackEntry {
    pub previous_piece: Option<Piece>,
    pub previous_square: Option<Square>,
    pub made_capture: bool,
    pub eval: i32,
}

pub struct SearchInfo {
    pub ss: [SearchStackEntry; MAX_PLY],
    pub lmr_table: LMRTable,
    pub history_table: [[i32; 64]; 12],
    pub caphist_table: [[[i32; 5]; 64]; 12],
    pub counter_moves: [[Move; 64]; 12],
    pub followup_moves: [[Move; 64]; 12],
    pub corrhist: [[i32; CORRHIST_SIZE as usize]; 2],
    pub killer_moves: [Option<Move>; MAX_PLY],
    pub excluded: [Option<Move>; MAX_PLY],
}

pub struct LMRTable {
    pub reduction_table: [[[i32; 32]; 32]; 2],
}

impl Default for SearchStackEntry {
    fn default() -> Self {
        Self {
            eval: -INFINITY,
            previous_piece: None,
            previous_square: None,
            made_capture: false,
        }
    }
}

impl Default for LMRTable {
    fn default() -> Self {
        let tb = (read_param!(LMR_TACTICAL_BASE) as f64) / 100.0;
        let td = (read_param!(LMR_TACTICAL_DIVISOR) as f64) / 100.0;
        let qb = (read_param!(LMR_QUIET_BASE) as f64) / 100.0;
        let qd = (read_param!(LMR_QUIET_DIVISOR) as f64) / 100.0;
        let mut reduction_table = [[[0; 32]; 32]; 2];
        for depth in 0..32 {
            for played in 0..32 {
                reduction_table[0][depth][played] =
                    (tb + f64::ln(depth as f64) * f64::ln(played as f64) / td) as i32;
                //tactical move
                reduction_table[1][depth][played] =
                    (qb + f64::ln(depth as f64) * f64::ln(played as f64) / qd) as i32;
                //quiet move
            }
        }

        LMRTable { reduction_table }
    }
}

impl Default for SearchInfo {
    fn default() -> Self {
        Self {
            ss: [SearchStackEntry::default(); MAX_PLY],
            lmr_table: LMRTable::default(),
            history_table: [[0; 64]; 12],
            caphist_table: [[[0; 5]; 64]; 12],
            corrhist: [[0; CORRHIST_SIZE as usize]; 2],
            followup_moves: [[NULL_MOVE; 64]; 12],
            counter_moves: [[NULL_MOVE; 64]; 12],
            killer_moves: [None; 64],
            excluded: [None; 64],
        }
    }
}

pub struct Thread<'a> {
    pub pv_length: [usize; 64],
    pub pv: [[Move; MAX_PLY]; MAX_PLY],
    pub tt: TTRef<'a>,
    pub ply: usize,
    pub nodes: usize,
    pub timer: Timer,
    pub stop: &'a AtomicBool,
    pub moves_fully_searched: usize,
    pub do_pruning: bool,
    pub info: SearchInfo,
    pub double_extensions: u8,
}

pub struct Timer {
    pub max_nodes: usize,
    pub end_time: Instant,
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            max_nodes: 0,
            end_time: Instant::now(),
        }
    }
}

impl<'a> Thread<'a> {
    pub fn new(
        end_time: Instant,
        max_nodes: usize,
        tt: &'a TranspositionTable,
        stop: &'a AtomicBool,
    ) -> Self {
        let timer = Timer {
            max_nodes,
            end_time,
        };

        Thread {
            pv_length: [0; MAX_PLY],
            pv: [[NULL_MOVE; MAX_PLY]; MAX_PLY],
            tt: TTRef::new(tt),
            ply: 0,
            nodes: 0,
            timer,
            stop,
            moves_fully_searched: 0,
            do_pruning: true,
            info: SearchInfo::default(),
            double_extensions: 0,
        }
    }
}

pub struct Searcher<'a> {
    _nodecount: AtomicU64,
    tt: &'a TranspositionTable,
}

impl<'a> Searcher<'a> {
    #[must_use]
    pub fn new(tt: &'a TranspositionTable) -> Self {
        Self {
            _nodecount: AtomicU64::new(0),
            tt,
        }
    }
    //comment is for threads variable which is unused in datagen mode
    #[allow(unused)]
    #[allow(clippy::too_many_arguments)]
    pub fn start_search(
        &self,
        position: &mut Board,
        time_left: usize,
        inc: usize,
        moves_to_go: usize,
        movetime: usize,
        max_nodes: usize,
        threads: usize,
    ) -> MoveData {
        // Soft-limit vs Hard-limit is an idea explained to me by the author of Sirius
        // Soft limit: if you complete an iteration and the time taken > this, exit
        // Hard limit: if you are currently searching (i.e. in the middle of the tree) and
        //             time taken > this, then exit search
        // in practice you should mostly exit at the soft-limit
        let (soft_limit, hard_limit) = match movetime {
            0 => move_time(time_left, inc, moves_to_go, position.ply),

            k => {
                if k <= MOVE_OVERHEAD {
                    let t = std::cmp::max(MIN_MOVE_TIME, k / 2);
                    (t, t)
                } else {
                    let t = std::cmp::max(MIN_MOVE_TIME, k - MOVE_OVERHEAD);
                    (t, t)
                }
            }
        };

        let start = Instant::now();
        let end_time = start + Duration::from_millis(hard_limit as u64);

        let stop = AtomicBool::new(false);

        let mut main_thread = Thread::new(end_time, max_nodes, self.tt, &stop);

        //datagen is already multi-threaded so only search on one thread
        #[cfg(feature = "datagen")]
        {
            return iterative_deepening(
                &mut position.clone(),
                soft_limit,
                hard_limit,
                &mut main_thread,
                false,
            );
        }

        #[cfg(not(feature = "datagen"))]
        std::thread::scope(|s| {
            let main_handle = s.spawn(|| {
                iterative_deepening(
                    &mut position.clone(),
                    soft_limit,
                    hard_limit,
                    &mut main_thread,
                    true,
                )
            });

            for _ in 0..threads - 1 {
                let mut pos = *position;
                let mut worker = Thread::new(end_time, max_nodes, self.tt, &stop);
                s.spawn(move || {
                    iterative_deepening(&mut pos, soft_limit, hard_limit, &mut worker, false)
                });
            }

            main_handle.join().expect("error in main thread")
        })
    }
}

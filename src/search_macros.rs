#[macro_export]
macro_rules! singularity_de {
    ($self_:expr, $pv_node:expr, $excluded_eval:expr, $threshold:expr) => {
        DO_SINGULARITY_DE
            && !$pv_node
            && $excluded_eval < $threshold - read_param!(SINGULARITY_DE_MARGIN)
            && $self_.double_extensions <= 6
    };
}

#[macro_export]
macro_rules! tt_cutoff {
    ($singular:expr, $root:expr, $pv_node:expr, $depth:expr, $entry:expr,
     $beta:expr, $alpha:expr, $cutnode:expr, $in_check:expr) => {
        !$singular
            && !$root
            && (!$pv_node
                && $depth <= $entry.depth
                && match $entry.flag {
                    EntryFlag::Exact => true,
                    EntryFlag::LowerBound => $entry.eval >= $beta,
                    EntryFlag::UpperBound => $entry.eval <= $alpha,
                    EntryFlag::Missing => false,
                }
                || ($cutnode
                    && $entry.eval
                        - read_param!(TT_FUTILITY_MARGIN)
                            * ($depth as i32 - $entry.depth as i32).max(1)
                        >= $beta
                    && $entry.flag != EntryFlag::UpperBound
                    && !$in_check))
    };
}

#[macro_export]
macro_rules! can_static_prune {
    ($self_:expr, $in_check:expr, $singular:expr, $pv_node:expr) => {
        !$in_check && !$singular && $self_.do_pruning && !$pv_node
    };
}

#[macro_export]
macro_rules! can_rfp {
    ($depth:expr, $static_eval:expr, $improving:expr, $beta:expr) => {
        $depth <= read_param!(RFP_DEPTH)
            && $static_eval - (read_param!(RFP_MARGIN) * ($depth - $improving as u8)) as i32
                >= $beta
    };
}

#[macro_export]
macro_rules! can_razor {
    ($depth:expr, $static_eval:expr, $improving:expr, $opponent_captured:expr,
     $opponent_worsening:expr, $alpha:expr) => {
        $depth <= read_param!(MAX_RAZOR_DEPTH)
            && $static_eval
                + read_param!(RAZORING_MARGIN)
                    * ($depth as i32 + $improving as i32
                        - ($opponent_captured || !$opponent_worsening) as i32)
                <= $alpha
    };
}

#[macro_export]
macro_rules! can_nmp {
    ($position:expr, $static_eval:expr, $depth:expr, $beta:expr, $root:expr) => {
        !$position.is_kp_endgame()
            && !$position.last_move_null
            && $static_eval + read_param!(NMP_FACTOR) * $depth as i32 - read_param!(NMP_BASE)
                >= $beta
            && !$root
    };
}

#[macro_export]
macro_rules! do_iir {
    ($pv_node:expr, $cutnode:expr, $depth:expr, $tt_move:expr) => {
        ($pv_node || $cutnode) && $depth >= read_param!(IIR_DEPTH_MINIMUM) && !$tt_move
    };
}

#[macro_export]
macro_rules! maybe_singular {
    ($root:expr, $depth:expr, $singular:expr, $m:expr, $best_move:expr,
     $tt_depth:expr, $tt_bound:expr) => {
        DO_SINGULARITY_EXTENSION
            && !$root
            && $depth >= 8
            && !$singular
            && $m == $best_move
            && $tt_depth >= $depth - 3
            && $tt_bound != EntryFlag::UpperBound
    };
}

#[macro_export]
macro_rules! do_iiw {
    ($pv_node:expr, $tt_hit:expr, $tt_bound:expr, $root:expr, $singular:expr,
     $tt_score:expr, $alpha:expr, $beta:expr) => {
        $pv_node
            && $tt_hit
            && $tt_bound == EntryFlag::Exact
            && !$root
            && !$singular
            && $tt_score >= $alpha
            && $tt_score <= $beta
    };
}

#[macro_export]
macro_rules! should_reduce {
    ($legal:expr, $pv_node:expr, $tt_move:expr, $root:expr, $tactical:expr,
     $depth:expr, $not_mated:expr) => {
        $legal
            > (FULL_DEPTH_MOVES + $pv_node as u8 + !$tt_move as u8 + $root as u8 + $tactical as u8)
            && $depth >= REDUCTION_LIMIT
            && $not_mated
    };
}

#[macro_export]
macro_rules! corrhist_update_allowed {
    ($in_check:expr, $best_move:expr, $position:expr, $hash_flag:expr,
     $best_score:expr, $static_eval:expr) => {
        !($in_check
            || $best_move.is_capture($position)
            || ($hash_flag == EntryFlag::LowerBound && $best_score <= $static_eval)
            || ($hash_flag == EntryFlag::UpperBound && $best_score >= $static_eval))
    };
}

#[macro_export]
macro_rules! should_correct_with_tt {
    ($tt_hit:expr, $static_eval:expr, $tt_score:expr, $tt_bound:expr) => {
        $tt_hit
            && !(($static_eval > $tt_score && $tt_bound == EntryFlag::LowerBound)
                || ($static_eval < $tt_score && $tt_bound == EntryFlag::UpperBound))
    };
}

#[macro_export]
macro_rules! do_lmp {
    ($depth:expr, $legal:expr, $lmp_threshold:expr, $in_check:expr) => {
        $depth <= read_param!(LMP_DEPTH) && $legal > $lmp_threshold && !$in_check
    };
}

#[macro_export]
macro_rules! do_see_pruning {
    ($lmr_depth:expr, $considered:expr, $pv_node:expr) => {
        $lmr_depth <= read_param!(SEE_PRUNING_DEPTH) && $considered > 1 && !$pv_node
    };
}

pub(crate) use can_nmp;
pub(crate) use can_razor;
pub(crate) use can_rfp;
pub(crate) use can_static_prune;
pub(crate) use corrhist_update_allowed;
pub(crate) use do_iir;
pub(crate) use do_iiw;
pub(crate) use do_lmp;
pub(crate) use do_see_pruning;
pub(crate) use maybe_singular;
pub(crate) use should_correct_with_tt;
pub(crate) use should_reduce;
pub(crate) use singularity_de;
pub(crate) use tt_cutoff;

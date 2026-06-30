#[cfg(feature = "stats")]
pub mod stats {
    use std::fmt;
    use std::sync::atomic::{AtomicU32, Ordering};

    pub struct SearchStats {
        pub nodes: AtomicU32,
        pub qnodes: AtomicU32,

        pub pv_nodes: AtomicU32,
        pub cutnodes: AtomicU32,
        pub all_nodes: AtomicU32,

        pub mxd_nodes: AtomicU32,
        pub mdp_cutoffs: AtomicU32,
        pub tt_cutoffs: AtomicU32,
        pub tt_fp_cutoffs: AtomicU32,

        pub rfp_cutoffs: AtomicU32,
        pub razoring_cutoffs: AtomicU32,

        pub nmp_attempts: AtomicU32,
        pub nmp_cutoffs: AtomicU32,

        pub iir_reductions: AtomicU32,

        pub probcut_attempts: AtomicU32,
        pub probcut_cutoffs: AtomicU32,

        pub lmp_skips: AtomicU32,
        pub hp_skips: AtomicU32,
        pub see_skips: AtomicU32,

        pub singularity_checks: AtomicU32,
        pub singularity_exts: AtomicU32,
        pub singularity_dexts: AtomicU32,
        pub singularity_texts: AtomicU32,
        pub multicuts: AtomicU32,
        pub singularity_reductions: AtomicU32,

        pub iaw_entries: AtomicU32,
        pub iaw_pointless: AtomicU32,
        pub iaw_exact_exits: AtomicU32,
        pub iaw_low_exits: AtomicU32,
        pub iaw_high_exits: AtomicU32,
        pub iaw_fails: AtomicU32,

        pub lmr_attempts: AtomicU32,
        pub lmr_full_depth: AtomicU32,
        pub lmr_pv_exits: AtomicU32,

        pub moveloop_entries: AtomicU32,
        pub moves_considered: AtomicU32,
        pub alpha_raises: AtomicU32,
        pub beta_cutoffs: AtomicU32,

        pub qs_moveloop_entries: AtomicU32,
        pub qs_moves_considered: AtomicU32,
        pub qs_stand_pat_cutoffs: AtomicU32,
        pub qs_fp_skips: AtomicU32,
        pub qs_bad_cap_skips: AtomicU32,
        pub qs_alpha_raises: AtomicU32,
        pub qs_beta_cutoffs: AtomicU32,
    }

    impl SearchStats {
        pub const fn new() -> Self {
            Self {
                nodes: AtomicU32::new(0),
                qnodes: AtomicU32::new(0),

                pv_nodes: AtomicU32::new(0),
                cutnodes: AtomicU32::new(0),
                all_nodes: AtomicU32::new(0),

                mxd_nodes: AtomicU32::new(0),
                mdp_cutoffs: AtomicU32::new(0),
                tt_cutoffs: AtomicU32::new(0),
                tt_fp_cutoffs: AtomicU32::new(0),

                rfp_cutoffs: AtomicU32::new(0),
                razoring_cutoffs: AtomicU32::new(0),

                nmp_attempts: AtomicU32::new(0),
                nmp_cutoffs: AtomicU32::new(0),

                iir_reductions: AtomicU32::new(0),

                probcut_attempts: AtomicU32::new(0),
                probcut_cutoffs: AtomicU32::new(0),

                lmp_skips: AtomicU32::new(0),
                hp_skips: AtomicU32::new(0),
                see_skips: AtomicU32::new(0),

                singularity_checks: AtomicU32::new(0),
                singularity_exts: AtomicU32::new(0),
                singularity_dexts: AtomicU32::new(0),
                singularity_texts: AtomicU32::new(0),
                multicuts: AtomicU32::new(0),
                singularity_reductions: AtomicU32::new(0),

                iaw_entries: AtomicU32::new(0),
                iaw_pointless: AtomicU32::new(0),
                iaw_exact_exits: AtomicU32::new(0),
                iaw_low_exits: AtomicU32::new(0),
                iaw_high_exits: AtomicU32::new(0),
                iaw_fails: AtomicU32::new(0),

                lmr_attempts: AtomicU32::new(0),
                lmr_full_depth: AtomicU32::new(0),
                lmr_pv_exits: AtomicU32::new(0),

                moveloop_entries: AtomicU32::new(0),
                moves_considered: AtomicU32::new(0),
                alpha_raises: AtomicU32::new(0),
                beta_cutoffs: AtomicU32::new(0),

                qs_moveloop_entries: AtomicU32::new(0),
                qs_moves_considered: AtomicU32::new(0),
                qs_stand_pat_cutoffs: AtomicU32::new(0),
                qs_fp_skips: AtomicU32::new(0),
                qs_bad_cap_skips: AtomicU32::new(0),
                qs_alpha_raises: AtomicU32::new(0),
                qs_beta_cutoffs: AtomicU32::new(0),
            }
        }

        #[inline]
        pub fn inc(field: &AtomicU32) {
            field.fetch_add(1, Ordering::Relaxed);
        }

        #[inline]
        pub fn add(field: &AtomicU32, n: u32) {
            field.fetch_add(n, Ordering::Relaxed);
        }

        #[inline]
        fn pct(num: u32, den: u32) -> f64 {
            if den == 0 { 0.0 } else { 100.0 * num as f64 / den as f64 }
        }
    }

    pub static STATS: SearchStats = SearchStats::new();

    impl fmt::Display for SearchStats {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            macro_rules! load {
                ($x:ident) => {
                    self.$x.load(Ordering::Relaxed)
                };
            }

            let nodes = load!(nodes);
            let qnodes = load!(qnodes);
            let total_nodes = nodes as u64 + qnodes as u64;

            let moveloop_entries = load!(moveloop_entries);
            let moves_considered = load!(moves_considered);

            let qs_moveloop_entries = load!(qs_moveloop_entries);
            let qs_moves_considered = load!(qs_moves_considered);

            write!(
                f,
                "\
Search stats

nodes:
  main nodes:          {nodes}
  qnodes:              {qnodes}
  total nodes:         {total_nodes}

node types:
  pv nodes:            {:>10}  ({:>6.2}% of main nodes)
  cut nodes:           {:>10}  ({:>6.2}% of main nodes)
  all nodes:           {:>10}  ({:>6.2}% of main nodes)

early exits:
  mxd nodes:           {:>10}  ({:>6.2}% of main nodes)
  mdp cutoffs:         {:>10}  ({:>6.2}% of main nodes)
  tt cutoffs:          {:>10}  ({:>6.2}% of main nodes)
  tt fp cutoffs:       {:>10}  ({:>6.2}% of main nodes)
  rfp cutoffs:         {:>10}  ({:>6.2}% of main nodes)
  razoring cutoffs:    {:>10}  ({:>6.2}% of main nodes)

main move loop:
  entries:             {moveloop_entries:>10}  ({:>6.2}% of main nodes)
  moves considered:    {moves_considered:>10}  ({:>6.2} per moveloop entry)
  alpha raises:        {:>10}  ({:>6.2}% of moveloop entries)
  beta cutoffs:        {:>10}  ({:>6.2}% of moveloop entries)

move pruning / skips:
  lmp skips:           {:>10}  ({:>6.2}% of moveloop entries)
  history skips:       {:>10}  ({:>6.2}% of moveloop entries)
  SEE skips:           {:>10}  ({:>6.2}% of moves considered)

null move pruning:
  attempts:            {:>10}  ({:>6.2}% of main nodes)
  cutoffs:             {:>10}  ({:>6.2}% of attempts, {:>6.2}% of main nodes)

probcut:
  attempts:            {:>10}  ({:>6.2}% of main nodes)
  cutoffs:             {:>10}  ({:>6.2}% of attempts, {:>6.2}% of main nodes)

lmr:
  attempts:            {:>10}  ({:>6.2}% of moves considered)
  researches:          {:>10}  ({:>6.2}% of attempts, {:>6.2}% of moves considered)
  pv exits:            {:>10}  ({:>6.2}% of attempts, {:>6.2}% of moves considered)

singularity:
  checks:              {:>10}  ({:>6.2}% of main nodes)
  single extensions:   {:>10}  ({:>6.2}% of checks)
  double extensions:   {:>10}  ({:>6.2}% of checks)
  triple extensions:   {:>10}  ({:>6.2}% of checks)
  multicuts:           {:>10}  ({:>6.2}% of checks)
  reductions:          {:>10}  ({:>6.2}% of checks)

iaw:
  entries:             {:>10}  ({:>6.2}% of moveloop entries)
  pointless:           {:>10}  ({:>6.2}% of entries)
  exact exits:         {:>10}  ({:>6.2}% of entries)
  low exits:           {:>10}  ({:>6.2}% of entries)
  high exits:          {:>10}  ({:>6.2}% of entries)
  fails:               {:>10}  ({:>6.2}% of entries)

qsearch:
  moveloop entries:   {qs_moveloop_entries:>10}  ({:>6.2}% of qnodes)
  moves considered:    {qs_moves_considered:>10}  ({:>6.2} per qsearch moveloop entry)
  stand-pat cutoffs:   {:>10}  ({:>6.2}% of qnodes)
  fp skips:            {:>10}  ({:>6.2}% of qsearch moves considered)
  bad cap skips:       {:>10}  ({:>6.2}% of qsearch moves considered)
  alpha raises:        {:>10}  ({:>6.2}% of qsearch moveloop entries)
  beta cutoffs:        {:>10}  ({:>6.2}% of qsearch moveloop entries)
",
                load!(pv_nodes),
                Self::pct(load!(pv_nodes), nodes),
                load!(cutnodes),
                Self::pct(load!(cutnodes), nodes),
                load!(all_nodes),
                Self::pct(load!(all_nodes), nodes),
                load!(mxd_nodes),
                Self::pct(load!(mxd_nodes), nodes),
                load!(mdp_cutoffs),
                Self::pct(load!(mdp_cutoffs), nodes),
                load!(tt_cutoffs),
                Self::pct(load!(tt_cutoffs), nodes),
                load!(tt_fp_cutoffs),
                Self::pct(load!(tt_fp_cutoffs), nodes),
                load!(rfp_cutoffs),
                Self::pct(load!(rfp_cutoffs), nodes),
                load!(razoring_cutoffs),
                Self::pct(load!(razoring_cutoffs), nodes),
                Self::pct(moveloop_entries, nodes),
                if moveloop_entries == 0 { 0.0 } else { moves_considered as f64 / moveloop_entries as f64 },
                load!(alpha_raises),
                Self::pct(load!(alpha_raises), moveloop_entries),
                load!(beta_cutoffs),
                Self::pct(load!(beta_cutoffs), moveloop_entries),
                load!(lmp_skips),
                Self::pct(load!(lmp_skips), moveloop_entries),
                load!(hp_skips),
                Self::pct(load!(hp_skips), moveloop_entries),
                load!(see_skips),
                Self::pct(load!(see_skips), moves_considered),
                load!(nmp_attempts),
                Self::pct(load!(nmp_attempts), nodes),
                load!(nmp_cutoffs),
                Self::pct(load!(nmp_cutoffs), load!(nmp_attempts)),
                Self::pct(load!(nmp_cutoffs), nodes),
                load!(probcut_attempts),
                Self::pct(load!(probcut_attempts), nodes),
                load!(probcut_cutoffs),
                Self::pct(load!(probcut_cutoffs), load!(probcut_attempts)),
                Self::pct(load!(probcut_cutoffs), nodes),
                load!(lmr_attempts),
                Self::pct(load!(lmr_attempts), moves_considered),
                load!(lmr_full_depth),
                Self::pct(load!(lmr_full_depth), load!(lmr_attempts)),
                Self::pct(load!(lmr_full_depth), moves_considered),
                load!(lmr_pv_exits),
                Self::pct(load!(lmr_pv_exits), load!(lmr_attempts)),
                Self::pct(load!(lmr_pv_exits), moves_considered),
                load!(singularity_checks),
                Self::pct(load!(singularity_checks), nodes),
                load!(singularity_exts),
                Self::pct(load!(singularity_exts), load!(singularity_checks)),
                load!(singularity_dexts),
                Self::pct(load!(singularity_dexts), load!(singularity_checks)),
                load!(singularity_texts),
                Self::pct(load!(singularity_texts), load!(singularity_checks)),
                load!(multicuts),
                Self::pct(load!(multicuts), load!(singularity_checks)),
                load!(singularity_reductions),
                Self::pct(load!(singularity_reductions), load!(singularity_checks)),
                load!(iaw_entries),
                Self::pct(load!(iaw_entries), moveloop_entries),
                load!(iaw_pointless),
                Self::pct(load!(iaw_pointless), load!(iaw_entries)),
                load!(iaw_exact_exits),
                Self::pct(load!(iaw_exact_exits), load!(iaw_entries)),
                load!(iaw_low_exits),
                Self::pct(load!(iaw_low_exits), load!(iaw_entries)),
                load!(iaw_high_exits),
                Self::pct(load!(iaw_high_exits), load!(iaw_entries)),
                load!(iaw_fails),
                Self::pct(load!(iaw_fails), load!(iaw_entries)),
                Self::pct(qs_moveloop_entries, qnodes),
                if qs_moveloop_entries == 0 { 0.0 } else { qs_moves_considered as f64 / qs_moveloop_entries as f64 },
                load!(qs_stand_pat_cutoffs),
                Self::pct(load!(qs_stand_pat_cutoffs), qnodes),
                load!(qs_fp_skips),
                Self::pct(load!(qs_fp_skips), qs_moves_considered),
                load!(qs_bad_cap_skips),
                Self::pct(load!(qs_bad_cap_skips), qs_moves_considered),
                load!(qs_alpha_raises),
                Self::pct(load!(qs_alpha_raises), qs_moveloop_entries),
                load!(qs_beta_cutoffs),
                Self::pct(load!(qs_beta_cutoffs), qs_moveloop_entries),
            )
        }
    }
}

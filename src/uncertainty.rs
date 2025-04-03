use crate::*;

use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use rand::Rng;
use rayon::prelude::*;
use std::error::Error;

use utils::math;

#[allow(unused)]
const LOSING_SIDE_MATERIAL_WEIGHTS: [f32; 13] = [
    1.0965039, 1.0757979, 1.0569726, 1.059839, 1.0428557, 1.0292687, 1.0110841, 1.0151744,
    1.0125952, 1.0, 0.9591061, 0.891087, 0.80261296,
];
//done in same way as phase score

const MUTATION_PROBABILTY: f32 = 0.06;
const FIRST_GEN_MUTATION_PROBABILITY: f32 = 0.3;
const POPULATION_SIZE: i32 = 50;
const NUM_GENERATIONS: i32 = 100;

#[derive(Clone, PartialEq)]
struct Individual {
    weights: Vec<Vec<f32>>,
    loss: f32,
}

/// This function returns a multiplicative constant which we multiply an evaluation by
/// depending on how confident we are in that evaluation
pub fn confidence_weight(losing_side_phase: usize) -> f32 {
    let mut w = 1.0;

    let phase_index = usize::clamp(losing_side_phase, 0, 12);

    w *= LOSING_SIDE_MATERIAL_WEIGHTS[phase_index];

    w
}

fn tuner_cw(losing_side_phase: usize, weights: &Vec<Vec<f32>>) -> f32 {
    let mat_w = weights[0].clone();

    let mut w = 1.0;

    let phase_index = usize::clamp(losing_side_phase, 0, 12);
    w *= mat_w[phase_index];

    w
}

/// Function which takes in an evaluation and converts it to an expected score from the game
/// This function must:
/// - have range [0, 1]
/// - be S-shaped
/// - be strictly increasing
/// - have a point of inflection at (0, 0.5)
fn wdl(eval: i32) -> f32 {
    math::sigmoid((eval as f32) / 150.0)
}

fn init_weights() -> Individual {
    Individual {
        weights: vec![vec![
            1.06, 1.05, 1.04, 1.06, 1.03, 1.03, 1.02, 1.02, 1.01, 1.01, 0.97, 0.92, 0.80,
        ]],
        loss: 0.0,
    }
}

impl Individual {
    fn compute_loss(&mut self, data: &Vec<(String, f32)>) {
        let mut total_loss = 0.0;
        for elem in data {
            let pos = Board::from(&elem.0);
            let label_wdl = elem.1;

            let static_eval = match pos.side_to_move {
                Colour::White => eval::evaluate(&pos),
                Colour::Black => -eval::evaluate(&pos),
            };

            let winning_side = if static_eval >= 0 {
                Colour::White
            } else {
                Colour::Black
            };

            let losing_phase_score = match winning_side {
                Colour::White => {
                    count(pos.bitboards[BQ]) * 4
                        + count(pos.bitboards[BR]) * 2
                        + count(pos.bitboards[BB])
                        + count(pos.bitboards[BN])
                }
                Colour::Black => {
                    count(pos.bitboards[WQ]) * 4
                        + count(pos.bitboards[WR]) * 2
                        + count(pos.bitboards[WB])
                        + count(pos.bitboards[WN])
                }
            };

            let w = tuner_cw(losing_phase_score, &self.weights);

            let computed_eval = (static_eval as f32 * w) as i32;
            let computed_wdl = wdl(computed_eval);

            let delta = (label_wdl - computed_wdl).abs();
            total_loss += delta;
        }

        self.loss = total_loss;
    }

    #[allow(unused)]
    fn tune_one_parameter(&self, i: usize, j: usize, data: &Vec<(String, f32)>) -> Self {
        //since parameters are combined linearly they can actually be tuned one at a time
        let mut n = self.clone();
        let mut best = n.clone();
        let mut k = 0.74;

        let bar = ProgressBar::new(50);
        bar.set_style(
            ProgressStyle::with_template(
                "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
            )
            .unwrap()
            .progress_chars("##-"),
        );

        while k < 1.25 {
            k += 0.01;
            n.weights[i][j] = k;
            n.compute_loss(data);
            if n.loss < best.loss {
                best = n.clone();
            }

            bar.inc(1);
        }
        bar.finish();
        best
    }

    fn mutate(&self, probability: f32) -> Self {
        let mut n = Self {
            weights: self.weights.clone(),
            loss: 0.0,
        };

        let mut rng = rand::thread_rng();

        for (i, x) in self.weights.iter().enumerate() {
            for (j, w) in x.iter().enumerate() {
                let f = rng.gen::<f32>();

                if f <= probability {
                    let delta = rng.gen_range(0.01..=0.10);
                    let change = w * delta;

                    let up = rng.gen_bool(0.5);

                    let v = if up { w + change } else { w - change };

                    n.weights[i][j] = v;
                }
            }
        }

        n
    }

    fn combine(&self, other: &Self) -> Self {
        let mut x = Self {
            weights: self.weights.clone(),
            loss: 0.0,
        };
        let mut rng = rand::thread_rng();
        for i in 0..self.weights.len() {
            for j in 0..self.weights[i].len() {
                let b = rng.gen_bool(0.5);
                if b {
                    x.weights[i][j] = other.weights[i][j];
                }
            }
        }

        x.mutate(MUTATION_PROBABILTY)
    }
}

#[allow(unused)]
#[derive(Clone)]
enum LabelType {
    LoHi,
    GameResult,
}

fn parse_line(x: &str, method: LabelType) -> (String, f32) {
    let parts = x.split('|').collect::<Vec<&str>>();
    let fen = parts[0].trim();
    let eval = parts[1].trim().parse::<i32>().unwrap();
    let res = parts[2].trim().parse::<f32>().unwrap();

    (
        fen.to_owned(),
        match method {
            LabelType::LoHi => wdl(eval),
            LabelType::GameResult => res,
        },
    )
}

fn load_data(path: &str, method: LabelType) -> Vec<(String, f32)> {
    std::fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(|x| parse_line(x, method.clone()))
        .collect()
}

fn take_sample(data: &Vec<(String, f32)>, size: usize) -> Vec<(String, f32)> {
    let p = size as f32 / data.len() as f32;

    let mut rng = rand::thread_rng();

    let mut res = vec![];

    for x in data {
        let q: f32 = rng.gen();
        if q < p {
            res.push(x.clone());
        }
    }
    res
}

pub fn genetic_algorithm() -> Result<(), Box<dyn Error>> {
    println!("Beginning uncertainty parameter tuning.\n");

    let data = load_data(
        "/Users/seba/rs/Panda/marlinflow/trainer/data/data230325.txt",
        LabelType::LoHi,
    );

    let mut population = vec![init_weights()];
    for _ in 0..POPULATION_SIZE - 1 {
        population.push(init_weights().mutate(FIRST_GEN_MUTATION_PROBABILITY));
    }

    let mut first_sample = None;
    let mut first_avg_loss = None;

    for gen in 0..NUM_GENERATIONS {
        println!(
            "{} Starting generation {} of {}!",
            "INFO:".green().bold(),
            gen + 1,
            NUM_GENERATIONS
        );
        let data = take_sample(&data, 50_000);
        if first_sample.is_none() {
            first_sample = Some(data.clone());
        }
        println!("Sample size: {}", data.len());
        let mut new_population = population.clone();
        //use elitism to avoid "throwing away" a good solution
        for x in &population {
            let mut rng = rand::thread_rng();
            let n1 = rng.gen_range(0..POPULATION_SIZE);
            let child1 = x.combine(&population[n1 as usize]);

            let n2 = rng.gen_range(0..POPULATION_SIZE);
            let child2 = x.combine(&population[n2 as usize]);

            let n3 = rng.gen_range(0..POPULATION_SIZE);
            let child3 = x.combine(&population[n3 as usize]);

            let child7 = x.mutate(MUTATION_PROBABILTY);
            let child8 = x.mutate(MUTATION_PROBABILTY);
            let child9 = x.mutate(MUTATION_PROBABILTY);

            new_population.extend(vec![child1, child2, child3, child7, child8, child9]);
        }

        let bar = ProgressBar::new(POPULATION_SIZE as u64 * 7);
        bar.set_style(
            ProgressStyle::with_template(
                "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
            )
            .unwrap()
            .progress_chars("##-"),
        );

        new_population.par_iter_mut().for_each(|x| {
            let _ = x.compute_loss(&data);
            bar.inc(1);
        });
        bar.finish();
        new_population.sort_by(|a, b| a.loss.partial_cmp(&b.loss).unwrap()); //ascending sort (which is what we want)
        population = new_population[..POPULATION_SIZE as usize].to_vec();

        if first_avg_loss.is_none() {
            first_avg_loss = Some(new_population[0].loss / 50_000.0);
        }

        println!(
            "{} Generation {} of {}: average cost {}! \n",
            "INFO:".green().bold(),
            gen + 1,
            NUM_GENERATIONS,
            (population[0].loss as f32 / data.len() as f32)
        );
    }

    let mut best = population[0].clone();

    println!("RESULTS OF TUNING ARE: ");
    println!("======================\n");

    println!(
        "const LOSING_SIDE_MATERIAL_WEIGHTS: [f32; 13] = {:?};",
        best.weights[0]
    );
    println!(
        "const WINNING_SIDE_KING_SAFETY_WEIGHTS: [f32; 8] = {:?};",
        best.weights[1]
    );
    println!(
        "const WINNING_SIDE_IN_CHECK_WEIGHT: f32 = {};",
        best.weights[2][0]
    );

    println!("\n======================");
    best.compute_loss(&first_sample.expect("didn't find first sample :("));
    println!(
        "Loss on first sample after generation 1: {}",
        first_avg_loss.expect("no first avg loss")
    );
    println!(
        "Loss on first sample after generation {}: {}",
        NUM_GENERATIONS,
        best.loss / 50_000.0
    );

    Ok(())
}
pub fn tune_one_by_one() {
    println!("Beginning one-by-one parameter tuning");
    let data = load_data(
        "/Users/seba/rs/Panda/marlinflow/trainer/data/data230325.txt",
        LabelType::LoHi,
    );

    let data = take_sample(&data, 250_000);

    let mut i1 = init_weights();

    i1.compute_loss(&data);
    let original_loss = i1.loss;

    for i in 0..2 {
        for j in 0..i1.weights[i].len() {
            println!("Currently tuning parameter {}, {}", i, j);
            i1 = i1.tune_one_parameter(i, j, &data);
        }
    }

    let final_loss = i1.loss;

    println!(
        "Original loss: {}\nFinal loss: {}",
        original_loss, final_loss
    );

    println!(
        "const LOSING_SIDE_MATERIAL_WEIGHTS: [f32; 13] = {:?};",
        i1.weights[0]
    );
    println!(
        "const WINNING_SIDE_KING_SAFETY_WEIGHTS: [f32; 8] = {:?};",
        i1.weights[1]
    );
    println!(
        "const WINNING_SIDE_IN_CHECK_WEIGHT: f32 = {};",
        i1.weights[2][0]
    );
}

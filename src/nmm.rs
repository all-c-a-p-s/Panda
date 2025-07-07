use burn::{
    module::Module,
    nn::{
        loss::{MseLoss, Reduction},
        Linear, LinearConfig,
    },
    optim::{AdamConfig, GradientsParams, Optimizer},
    tensor::{backend::AutodiffBackend, Tensor},
};

use crate::*;

// input(896) -> hidden(2) -> output(1)
#[derive(Module, Debug)]
pub struct PolicyNet<B: AutodiffBackend> {
    hidden: Linear<B>,
    output: Linear<B>,
}

pub struct PolicyNetConfig {
    input_size: usize,
    hl_size: usize,
}

impl Default for PolicyNetConfig {
    fn default() -> Self {
        Self {
            input_size: 896,
            hl_size: 2,
        }
    }
}

pub struct TrainingConfig {
    pub model: PolicyNetConfig,
    pub optimizer: AdamConfig,
    pub lr: f64,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            model: PolicyNetConfig::default(),
            optimizer: AdamConfig::new(),
            lr: 0.001,
        }
    }
}

impl<B: AutodiffBackend> PolicyNet<B> {
    pub fn new(device: &B::Device, config: &PolicyNetConfig) -> Self {
        let hidden = LinearConfig::new(config.input_size, config.hl_size).init(device);
        let output = LinearConfig::new(config.hl_size, 1).init(device);

        Self { hidden, output }
    }

    pub fn forward(&self, input: Tensor<B, 1>) -> Tensor<B, 1> {
        let x = self.hidden.forward(input);
        let x = burn::tensor::activation::relu(x);
        let x = self.output.forward(x);
        burn::tensor::activation::tanh(x) //[-1, 1]
    }

    pub fn step(
        &mut self,
        input: Tensor<B, 1>,
        targets: Tensor<B, 1>,
        optim: AdamConfig,
        config: &TrainingConfig,
    ) where
        <B as AutodiffBackend>::InnerBackend: AutodiffBackend,
    {
        let mut optim = optim.init();
        let output = self.forward(input);
        let loss = MseLoss::new().forward(output.clone(), targets.clone(), Reduction::Auto);

        //stalls at runtime here
        let grads = loss.backward();

        *self = optim.step(
            config.lr,
            self.clone(),
            GradientsParams::from_grads(grads, self),
        );
    }
}

pub struct InputState<'a> {
    b: &'a Board,
    m: Move,
}

impl InputState<'_> {
    pub fn into_tensor<B: AutodiffBackend>(&self, device: &B::Device) -> Tensor<B, 1> {
        let mut pieces = [[0f32; 64]; 12];

        for (i, sq) in self.b.pieces_array.iter().enumerate() {
            if let Some(pc) = sq {
                pieces[*pc][i] = 1.0;
            }
        }

        let (mut sq_from, mut sq_to) = ([0f32; 64], [0f32; 64]);
        sq_from[self.m.square_from()] = 1.0;
        sq_to[self.m.square_to()] = 1.0;

        let mut pcsf = pieces.as_flattened().to_vec();
        pcsf.extend_from_slice(&sq_from);
        pcsf.extend_from_slice(&sq_to);

        let vals: [f32; 896] = pcsf.try_into().expect("???");
        Tensor::from_data(vals, device)
    }
}

pub fn make_input_state<'a>(b: &'a Board, m: Move) -> InputState<'a> {
    InputState { b, m }
}

pub fn make_targets<B: AutodiffBackend>(value: f32, device: &B::Device) -> Tensor<B, 1> {
    Tensor::from_data([value; 1], device)
}

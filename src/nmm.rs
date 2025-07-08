use tch::{
    nn, nn::Module, nn::Optimizer, nn::OptimizerConfig, nn::Sequential, nn::VarStore, Device,
    Tensor,
};

use crate::*;

const INPUT_DIM: i64 = 896;
const HL_SIZE: i64 = 2;

fn net(vs: &nn::Path) -> Sequential {
    nn::seq()
        .add(nn::linear(
            vs / "layer1",
            INPUT_DIM,
            HL_SIZE,
            Default::default(),
        ))
        .add_fn(|xs| xs.relu())
        .add(nn::linear(vs, HL_SIZE, 1, Default::default()))
}

pub struct NetConfig {
    pub net: Sequential,
    pub opt: Optimizer,
    pub vs: VarStore,
}

impl NetConfig {
    pub fn new() -> Self {
        let vs = nn::VarStore::new(Device::Cpu);
        let opt = nn::Adam::default()
            .build(&vs, 1e-3)
            .expect("failed to init optimiser");
        Self {
            net: net(&vs.root()),
            vs,
            opt,
        }
    }
}

pub fn update_net(
    net: &mut Sequential,
    opt: &mut Optimizer,
    is: &InputState,
    tgt: f32,
) -> Result<(), Box<dyn std::error::Error>> {
    let t = is.into_tensor();

    let tgt = make_target_tensor(tgt);

    let loss = net.forward(&t).mse_loss(&tgt, tch::Reduction::Mean);
    opt.backward_step(&loss);

    Ok(())
}

pub fn forward_prediction(
    net: &Sequential,
    is: &InputState,
) -> Result<f64, Box<dyn std::error::Error>> {
    let t = is.into_tensor();
    let out = net.forward(&t);
    Ok(out.double_value(&[0]))
}

pub struct InputState<'a> {
    b: &'a Board,
    m: Move,
}

impl InputState<'_> {
    pub fn into_tensor(&self) -> Tensor {
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
        Tensor::from_slice(&vals)
    }
}

pub fn make_input_state<'a>(b: &'a Board, m: Move) -> InputState<'a> {
    InputState { b, m }
}

pub fn make_target_tensor(tgt: f32) -> Tensor {
    Tensor::from_slice(&[tgt])
}

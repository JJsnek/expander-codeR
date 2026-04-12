use expander_codeR::field::F;
use expander_codeR::expander::{build_layer,SamplingMode};
use expander_codeR::encoder::encode_recursive;

use expander_codeR::experiment::*;
use rand::distributions::uniform::SampleBorrow;

fn main(){
    
    //experiments
    let mut results=Vec::new();

    let ns=vec![1024,];
    let ds = vec![3,5,7,9,11];
    let weights =vec![1,2,4,8,16];
    let modes=vec![
        SamplingMode::Random,
        SamplingMode::NonZero,
        SamplingMode::Hybrid,
    ];

    for &mode in &modes{
        for &n in &ns{
            for &d in &ds{
                for &w in &weights{
                    let cfg=ExperimentConfig{
                        n,
                        m:n/2,
                        d,
                        layers:2,
                        trials:1000,
                        weight:2,
                        mode,
                    };

                    println!(
                        "mode={:?}, n={}, d={}, weight={}",
                        mode, n, d, w
                    );

                    let result=run_experiment(&cfg);
                    results.push(result);
                }
            }
        }
    }
    write_csv(&results,"results.csv");
}
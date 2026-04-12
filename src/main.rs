use expander_codeR::field::F;
use expander_codeR::expander::{build_layer,SamplingMode};
use expander_codeR::encoder::encode_recursive;

fn main(){
    let n=1024;
    let m=512;
    let d=3;
    let layer=build_layer(n,m,d,SamplingMode::Random);

    let layers=vec![layer];

    let x:Vec<F>=(0..n).map(|_| F::from(1u64)).collect();

    let y=encode_recursive(x,&layers);

    println!("Output length: {}",y.len());
}
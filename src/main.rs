// Mandelbrot set in rust
//
// This code shows how to calculate the set in serial and parallel.
// More parallel versions will be added in the future.
//
// Written by Willi Kappler, grandor@gmx.de
//
// License: MIT
//
//#![feature(plugin)]
//
//#![plugin(clippy)]

// External crates
extern crate time;
extern crate rayon;

// Internal crates
extern crate mandel_util;
extern crate mandel_method;

// External modules
use time::{now};

// Internal modules
use mandel_util::{parse_arguments, do_run, compiler_version};
use mandel_method::*;

fn main() {
    // For example run with:
    // cargo run --release -- --re1=-2.0 --re2=1.0 --img1=-1.5 --img2=1.5
    //           --max_iter=2048 --img_size=1024 --num_threads=2
    //
    // Or just using the default values:
    // cargo run --release -- --num_threads=2
    //
    // Note that the image size must be a power of two

    let mandel_config = parse_arguments();

    let version = env!("CARGO_PKG_VERSION");

    println!("mandel-rust version: {}", version);
    println!("Number of repetitive runs: {}", mandel_config.num_of_runs);
    println!("Rustc version: {}", compiler_version);

    // Get current date and time once and pass it to the individual runs for the image filename.
    let tm = now();
    let tm = tm.strftime("%Y_%m_%d__%H_%M_%S").unwrap();
    let time_now = format!("{}", &tm);

    // vec! macro expects usize
    let mut image: Vec<u32> = vec![0; (mandel_config.img_size * mandel_config.img_size) as usize];

    do_run("serial", &serial, &mandel_config, &mut image, &time_now);

    do_run("scoped_thread_pool", &scoped_thread_pool_, &mandel_config, &mut image, &time_now);

    // Make sure this is only called once
    match rayon::initialize(rayon::Configuration::new().set_num_threads(mandel_config.num_threads as usize)) {
        Ok(_) => {
            do_run("rayon_join", &rayon_join, &mandel_config, &mut image, &time_now);

            do_run("rayon_par_iter", &rayon_par_iter, &mandel_config, &mut image, &time_now);
        },
        Err(e) => println!("Rayon error: set number of threads failed: {}", e)
    }

    do_run("rust_scoped_pool", &rust_scoped_pool, &mandel_config, &mut image, &time_now);

    do_run("job_steal", &job_steal, &mandel_config, &mut image, &time_now);

    do_run("job_steal_join", &job_steal_join, &mandel_config, &mut image, &time_now);

    // do_run("kirk_crossbeam", &kirk_crossbeam, &mandel_config, &mut image, &time_now);
}

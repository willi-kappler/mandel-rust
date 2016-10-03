#![allow(non_upper_case_globals)]

// External crates
#[macro_use]
extern crate clap;
extern crate num_cpus;
extern crate num;
extern crate time;

// External modules
use clap::App;
use num::complex::Complex64;
use time::{precise_time_ns};

// Rust modules
use std::fs::File;
use std::io::prelude::Write;
use std::io::Result;
use std::io::BufWriter;
use std::fs::OpenOptions;
use std::path::Path;
use std::fs;

// Configuration file, reflects command line options
#[derive(Copy, Clone)]
pub struct MandelConfig {
    pub re1: f64,
    pub re2: f64,
    pub img1: f64,
    pub img2: f64,
    pub x_step: f64,
    pub y_step: f64,
    pub max_iter: u32,
    pub img_size: u32,
    pub write_metadata: bool,
    pub no_ppm: bool,
    pub num_threads: u32,
    pub num_of_runs: u32
}

include!(concat!(env!("OUT_DIR"), "/compiler_version.rs"));

// Parse command line options via clap and returns the responding configuration
pub fn parse_arguments() -> MandelConfig {
    let matches = App::new("mandel_rust")
        .version("0.3")
        .author("Willi Kappler <grandor@gmx.de>")
        .about("Simple mandelbrot written in pure rust")
        .args_from_usage(
            "--re1=[REAL1] 'left real part (default: -2.0)'
             --re2=[REAL2] 'right real part (default: 1.0)'
             --img1=[IMAGINARY1] 'lower part (default: -1.50)'
             --img2=[IMAGINARY2] 'upper part (default: 1.50)'
             --write_metadata 'write metadata like run time into the ppm file (default: off)'
             --no_ppm 'disable creation of the ppm file, just run the calculation (default: off)'
             --bench 'use all available CPUs (default: off), will change in the future'
             --max_iter=[MAX_ITER] 'maximum number of iterations (default: 4096)'
             --img_size=[IMAGE_SIZE] 'size of image in pixel (square, default: 2048, must be a power of two)'
             --num_of_runs=[NUM_OF_RUNS] 'number of repetitive runs (default: 2)'
             --num_threads=[NUMBER_OF_THREADS] 'number of threads to use (default: 2)'")
        .get_matches();

    let re1 = value_t!(matches.value_of("REAL1"), f64).unwrap_or(-2.0);
    let re2 = value_t!(matches.value_of("REAL2"), f64).unwrap_or(1.0);
    let img1 = value_t!(matches.value_of("IMAGINARY1"), f64).unwrap_or(-1.5);
    let img2 = value_t!(matches.value_of("IMAGINARY2"), f64).unwrap_or(1.5);
    let metadata = matches.is_present("write_metadata");
    let bench = matches.is_present("bench");
    let no_ppm = matches.is_present("no_ppm");
    let max_iter = value_t!(matches.value_of("MAX_ITER"), u32).unwrap_or(4096);
    let img_size = value_t!(matches.value_of("IMAGE_SIZE"), u32).unwrap_or(2048);
    let num_of_runs = value_t!(matches.value_of("NUM_OF_RUNS"), u32).unwrap_or(2);
    let num_threads = if bench { num_cpus::get() as u32 } else {
        value_t!(matches.value_of("NUMBER_OF_THREADS"), u32).unwrap_or(2) };

    assert!(re1 < re2);
    assert!(img1 < img2);
    assert!(max_iter > 0);
    assert!(img_size > 0);
    assert!(num_threads > 0);

    println!("Configuration: re1: {:.2}, re2: {:.2}, img1: {:.2}, img2: {:.2}, max_iter: {}, img_size: {}, num_threads: {}",
        re1, re2, img1, img2, max_iter, img_size, num_threads);

    let x_step = (re2 - re1) / (img_size as f64);
    let y_step = (img2 - img1) / (img_size as f64);

    MandelConfig{
        re1: re1,
        re2: re2,
        img1: img1,
        img2: img2,
        x_step: x_step,
        y_step: y_step,
        max_iter: max_iter,
        img_size: img_size,
        write_metadata: metadata,
        no_ppm: no_ppm,
        num_threads: num_threads,
        num_of_runs: num_of_runs
    }
}

// The inner iteration loop of the mandelbrot calculation
// See https://en.wikipedia.org/wiki/Mandelbrot_set
pub fn mandel_iter(max_iter: u32, c: Complex64) -> u32 {
    let mut z: Complex64 = c;

    let mut iter = 0;

    while (z.norm_sqr() <= 4.0) && (iter < max_iter) {
        z = c + (z * z);
        iter = iter + 1;
    }

    iter
}

// Write calculated mandelbrot set as PPM image.
// Add run time information as comment.
fn write_image(file_name: &str, mandel_config: &MandelConfig, time_in_ms: f64, image: &[u32]) -> Result<()> {
    let mut buffer = BufWriter::new(try!(File::create(file_name)));

    try!(buffer.write(b"P3\n"));
    try!(write!(buffer, "# mandelbrot, max_iter: {}\n", mandel_config.max_iter));
    if mandel_config.write_metadata {
        // TODO: add more meta data: date and time, method, ...
        try!(write!(buffer, "# computation time: {} ms\n", time_in_ms));
    }
    try!(write!(buffer, "{0} {0}\n", mandel_config.img_size));
    try!(buffer.write(b"255\n"));

    let mut img_value: u32;

    for y in 0..mandel_config.img_size {
        for x in 0..mandel_config.img_size {

            img_value = image[((y * mandel_config.img_size) + x) as usize];
            if img_value == mandel_config.max_iter {
                try!(buffer.write(b"0 0 0 "));
            } else {
                try!(write!(buffer, "255 {} 0 ", (img_value % 16) * 16));
            }

        }
        try!(buffer.write(b"\n"));
    }

    Ok(())
}

fn write_benchmark_result(method: &str, num_threads: u32,
     time_in_ms: f64, min_time: f64, max_time: f64) -> Result<()> {

    // Check if output folder "plot" is available:

    if !Path::new("plot").exists() {
        // If not, create it!
        println!("Folder 'plot' does not exist, creating it...");
        try!(fs::create_dir("plot"));
    
    }

    let mut buffer = BufWriter::new(try!(
        OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(format!("plot{}{}.txt", std::path::MAIN_SEPARATOR, method))));

    try!(write!(buffer, "{} {} {} {}\n", num_threads, time_in_ms, min_time, max_time));

    Ok(())
}

// Prepares and runs one version of the mandelbrot set calculation.
pub fn do_run(method: &str, mandel_func: &Fn(&MandelConfig, &mut [u32]) -> (),
    mandel_config: &MandelConfig, image: &mut [u32], time_now: &str) {

    let mut repetitive_times = Vec::new();
    let mut min_time = std::f64::MAX;
    let mut max_time = 0.0;

    for _ in 0..mandel_config.num_of_runs {
        let start_time = precise_time_ns();

        mandel_func(mandel_config, image);

        let end_time = precise_time_ns();
        let total_time_in_ms = ((end_time - start_time) as f64) / (1000.0 * 1000.0);

        if total_time_in_ms > max_time {
            max_time = total_time_in_ms;
        }

        if total_time_in_ms < min_time {
            min_time = total_time_in_ms;
        }

        repetitive_times.push(total_time_in_ms);
    }

    let mean_time = repetitive_times.iter().fold(0.0, |sum, t| sum + t) /
        (mandel_config.num_of_runs as f64);

    println!("Time taken for this run ({}): {:.5} ms", method, mean_time);

    write_benchmark_result(&method, mandel_config.num_threads, mean_time,
        min_time, max_time).expect("I/O error while writing benchmark results");

    if !mandel_config.no_ppm {
        let file_name = format!("{}_{}.ppm", method, &time_now);

        write_image(&file_name, &mandel_config, mean_time, &image).expect(
            &format!("I/O error while writing image: '{}'", file_name));
    }
}

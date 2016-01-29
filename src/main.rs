// Mandelbrot set in rust
//
// This code shows how to calculate the set in serial and parallel.
// More parallel versions will be added in the future.
//
// Written by Willi Kappler
//
// License: MIT


#[macro_use]
extern crate clap;

extern crate time;
extern crate num;
extern crate num_cpus;
extern crate scoped_threadpool;
extern crate simple_parallel;
extern crate rayon;
extern crate scoped_pool;
extern crate jobsteal;

// Rust modules
use std::fs::File;
use std::io::prelude::Write;
use std::io::Result;
use std::io::BufWriter;

// External crates
use time::{precise_time_ns, now};
use num::complex::Complex64;
use clap::App;
use rayon::par_iter::*;

// Configuration file, reflects command line options
#[derive(Copy, Clone)]
struct MandelConfig {
    re1: f64,
    re2: f64,
    img1: f64,
    img2: f64,
    x_step: f64,
    y_step: f64,
    max_iter: u32,
    img_size: u32,
    write_metadata: bool,
    no_ppm: bool,
    num_threads: u32
}

// Parse command line options via clap and returns the responding configuration
fn parse_arguments() -> MandelConfig {
    let matches = App::new("mandel_rust")
        .version("0.3")
        .author("Willi Kappler <willi.kappler.gm@gmail.com>")
        .about("Simple mandelbrot written in pure rust")
        .args_from_usage(
            "--re1=[REAL1] 'left real part (default: -2.0)'
             --re2=[REAL2] 'right real part (default: 1.0)'
             --img1=[IMAGINARY1] 'lower part (default: -1.50)'
             --img2=[IMAGINARY2] 'upper part (default: 1.50)'
             --write_metadata 'write metadata like run time into the ppm file (default: off)'
             --no-ppm 'disable creation of the ppm file, just run the calculation (default: off)'
             --bench 'use all available CPUs (default: off), will change in the future'
             --max_iter=[MAX_ITER] 'maximum number of iterations (default: 2048)'
             --img_size=[IMAGE_SIZE] 'size of image in pixel (square, default: 1024, must be a power of two)'
             --num_threads=[NUMBER_OF_THREADS] 'number of threads to use (default: 2)'")
        .get_matches();

    let re1 = value_t!(matches.value_of("REAL1"), f64).unwrap_or(-2.0);
    let re2 = value_t!(matches.value_of("REAL2"), f64).unwrap_or(1.0);
    let img1 = value_t!(matches.value_of("IMAGINARY1"), f64).unwrap_or(-1.5);
    let img2 = value_t!(matches.value_of("IMAGINARY2"), f64).unwrap_or(1.5);
    let metadata = matches.is_present("write_metadata");
    let bench = matches.is_present("bench");
    let no_ppm = matches.is_present("no-ppm");
    let max_iter = value_t!(matches.value_of("MAX_ITER"), u32).unwrap_or(2048);
    let img_size = value_t!(matches.value_of("IMAGE_SIZE"), u32).unwrap_or(1024);
    let num_threads = if bench { num_cpus::get() as u32 } else { value_t!(matches.value_of("NUMBER_OF_THREADS"), u32).unwrap_or(2) };

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
        num_threads: num_threads
    }
}

// The inner iteration loop of the mandelbrot calculation
// See
fn mandel_iter(max_iter: u32, c: Complex64) -> u32 {
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

// The serial version of the mandelbrot set calculation.
fn serial(mandel_config: &MandelConfig, image: &mut [u32]) {
    for y in 0..mandel_config.img_size {
        for x in 0..mandel_config.img_size {
            image[((y * mandel_config.img_size) + x) as usize] =
                mandel_iter(mandel_config.max_iter,
                    Complex64{re: mandel_config.re1 + ((x as f64) * mandel_config.x_step),
                              im: mandel_config.img1 + ((y as f64) * mandel_config.y_step)}
                );
        }
    }
}

// The parallel version of the mandelbrot set calculation, uses scoped_threadpool.
fn scoped_thread_pool_(mandel_config: &MandelConfig, image: &mut [u32]) {
    let mut pool = scoped_threadpool::Pool::new(mandel_config.num_threads);

    pool.scoped(|scope| {
        for (y, slice) in image.chunks_mut(mandel_config.img_size as usize).enumerate() {
            scope.execute(move || {
                for x in 0..mandel_config.img_size {
                    slice[x as usize] =
                    mandel_iter(mandel_config.max_iter,
                        Complex64{re: mandel_config.re1 + ((x as f64) * mandel_config.x_step),
                                  im: mandel_config.img1 + ((y as f64) * mandel_config.y_step)}
                    );
                }
            });
        }
    });
}

// The parallel version of the mandelbrot set calculation, uses simple_parallel.
fn simple_parallel_(mandel_config: &MandelConfig, image: &mut [u32]) {
    let mut pool = simple_parallel::Pool::new(mandel_config.num_threads as usize);

    pool.for_(image.chunks_mut(mandel_config.img_size as usize).enumerate(), |(y, slice)| {
        for x in 0..mandel_config.img_size {
            slice[x as usize] =
            mandel_iter(mandel_config.max_iter,
                Complex64{re: mandel_config.re1 + ((x as f64) * mandel_config.x_step),
                          im: mandel_config.img1 + ((y as f64) * mandel_config.y_step)}
            );
        }
    });
}

// The parallel version of the mandelbrot set calculation, uses rayon join.
fn rayon_join(mandel_config: &MandelConfig, image: &mut [u32]) {
    rayon_helper(mandel_config, image, 0);
}

// Rayon helper function for recursive divide-and-conquer call
fn rayon_helper(mandel_config: &MandelConfig, slice: &mut [u32], y: u32) {
    if slice.len() == (mandel_config.img_size as usize) { // just process one scanline of the mandelbrot image
        for x in 0..mandel_config.img_size {
            slice[x as usize] =
            mandel_iter(mandel_config.max_iter,
                Complex64{re: mandel_config.re1 + ((x as f64) * mandel_config.x_step),
                          im: mandel_config.img1 + ((y as f64) * mandel_config.y_step)}
            );
        }
    } else {
        let mid = slice.len() / 2;
        let (top, bottom) = slice.split_at_mut(mid);
        rayon::join(
            || rayon_helper(mandel_config, top, y),
            || rayon_helper(mandel_config, bottom, y + ((mid as u32) / mandel_config.img_size))
        );
    }
}

// The parallel version of the mandelbrot set calculation, uses rayon par_iter.
fn rayon_par_iter(mandel_config: &MandelConfig, image: &mut [u32]) {

    image.par_iter_mut().enumerate().for_each(
        |(n, pixel)| {
            let y = (n as u32) / mandel_config.img_size;
            let x = (n as u32) - (y * mandel_config.img_size);
            *pixel = mandel_iter(mandel_config.max_iter,
                        Complex64{re: mandel_config.re1 + ((x as f64) * mandel_config.x_step),
                                  im: mandel_config.img1 + ((y as f64) * mandel_config.y_step)}
            );
        });
}

// The parallel version of the mandelbrot set calculation, uses rust scoped pool.
fn rust_scoped_pool(mandel_config: &MandelConfig, image: &mut [u32]) {
    let pool = scoped_pool::Pool::new(mandel_config.num_threads as usize);

    pool.scoped(|scope| {
        for (y, slice) in image.chunks_mut(mandel_config.img_size as usize).enumerate() {
            scope.execute(move || {
                for x in 0..mandel_config.img_size {
                    slice[x as usize] =
                    mandel_iter(mandel_config.max_iter,
                        Complex64{re: mandel_config.re1 + ((x as f64) * mandel_config.x_step),
                                  im: mandel_config.img1 + ((y as f64) * mandel_config.y_step)}
                    );
                }
            });
        }
    });
}

// The parallel version of the mandelbrot set calculation, uses jobsteal.
fn job_steal(mandel_config: &MandelConfig, image: &mut [u32]) {
    let mut pool = jobsteal::make_pool(mandel_config.num_threads as usize).unwrap();

    pool.scope(|scope| {
        for (y, slice) in image.chunks_mut(mandel_config.img_size as usize).enumerate() {
            scope.submit(move |_| {
                for x in 0..mandel_config.img_size {
                    slice[x as usize] =
                    mandel_iter(mandel_config.max_iter,
                        Complex64{re: mandel_config.re1 + ((x as f64) * mandel_config.x_step),
                                  im: mandel_config.img1 + ((y as f64) * mandel_config.y_step)}
                    );
                }
            });
        }
    });
}

// Prepares and runs one version of the mandelbro set calculation
fn do_run(file_name_prefix: &str, mandel_func: &Fn(&MandelConfig, &mut [u32]) -> (),
    mandel_config: &MandelConfig, image: &mut [u32], time_now: &str) {
    let start_time = precise_time_ns();

    mandel_func(mandel_config, image);

    let end_time = precise_time_ns();
    let total_time_in_ms = ((end_time - start_time) as f64) / (1000.0 * 1000.0);

    println!("Time taken for this run ({}): {:.5} ms", file_name_prefix, total_time_in_ms);

    if !mandel_config.no_ppm {
        let file_name = format!("{}_{}.ppm", file_name_prefix, &time_now);

        write_image(&file_name, &mandel_config, total_time_in_ms, &image).expect(
            &format!("Could not open file for writing: '{}'", file_name));
    }
}

fn main() {
    // For example run with:
    // cargo run --release -- --re1=-2.0 --re2=1.0 --img1=-1.5 --img2=1.5 --max_iter=2048 --img_size=1024 --num_threads=2
    //
    // Or just using the default values:
    // cargo run --release -- --num_threads=2
    //
    // Note that the image size must be a power of two

    let mandel_config = parse_arguments();

    // Get current date and time once and pass it to the individual runs for the image filename.
    let tm = now();
    let tm = tm.strftime("%Y_%m_%d__%H_%M_%S").unwrap();
    let time_now = format!("{}", &tm);

    // vec! macro expects usize
    let mut image: Vec<u32> = vec![0; (mandel_config.img_size * mandel_config.img_size) as usize];

    do_run("serial", &serial, &mandel_config, &mut image, &time_now);

    do_run("scoped_thread_pool", &scoped_thread_pool_, &mandel_config, &mut image, &time_now);

    do_run("simple_parallel", &simple_parallel_, &mandel_config, &mut image, &time_now);

    do_run("rayon_join", &rayon_join, &mandel_config, &mut image, &time_now);

    do_run("rayon_par_iter", &rayon_par_iter, &mandel_config, &mut image, &time_now);

    do_run("rust_scoped_pool", &rust_scoped_pool, &mandel_config, &mut image, &time_now);

    do_run("job_steal", &job_steal, &mandel_config, &mut image, &time_now);

}

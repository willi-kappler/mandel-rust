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
extern crate num;
extern crate time;
extern crate scoped_threadpool;
extern crate simple_parallel;
extern crate rayon;
extern crate scoped_pool;
extern crate jobsteal;
extern crate kirk;
extern crate crossbeam;

// Internal crates
extern crate mandel_util;

// External modules
use num::complex::Complex64;
use time::{now};
use rayon::par_iter::*;
use kirk::crew::deque::Options;

// Internal modules
use mandel_util::*;

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

// The parallel version of the mandelbrot set calculation, uses rayon par_iter_mut.
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
    let mut pool = jobsteal::make_pool((mandel_config.num_threads - 1) as usize).unwrap();

    pool.scope(|scope| {
        for (y, slice) in image.chunks_mut(mandel_config.img_size as usize).enumerate() {
            scope.submit(move || {
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

// The parallel version of the mandelbrot set calculation, uses jobsteal with divide-and-conquer strategy.
fn job_steal_join(mandel_config: &MandelConfig, image: &mut [u32]) {
    // Jobsteal uses n + 1 threads (1 main thread + n sub-threads)
    // It is OK to create a Jobsteal pool with zero threads.
    // See https://github.com/willi-kappler/mandel-rust/issues/1
    let mut pool = jobsteal::make_pool((mandel_config.num_threads - 1) as usize).unwrap();

    pool.scope(|scope| {
        job_steal_helper(mandel_config, scope, image, 0);
    })
}

// jobsteal helper for divide and conquer version.
fn job_steal_helper<'a, 'b>(mandel_config: &MandelConfig, spawner: &jobsteal::Spawner<'a, 'b>,
                            slice: &mut [u32], y: u32) {
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
        spawner.join(
            |inner| job_steal_helper(mandel_config, inner, top, y),
            |inner| job_steal_helper(mandel_config, inner, bottom, y + ((mid as u32) / mandel_config.img_size))
        );
    }
}

// The parallel version of the mandelbrot set calculation, uses kirk and crossbeam.
fn kirk_crossbeam(mandel_config: &MandelConfig, image: &mut [u32]) {
    crossbeam::scope(|scope| {
        let mut pool = kirk::Pool::<kirk::Deque<kirk::Task>>::scoped(scope,
            Options{ num_workers: mandel_config.num_threads as usize, .. Options::default()});
        for (y, slice) in image.chunks_mut(mandel_config.img_size as usize).enumerate() {
            pool.push(move || {
                for x in 0..mandel_config.img_size {
                    slice[x as usize] =
                    mandel_iter(mandel_config.max_iter,
                        Complex64{re: mandel_config.re1 + ((x as f64) * mandel_config.x_step),
                                  im: mandel_config.img1 + ((y as f64) * mandel_config.y_step)}
                    );
                }
            })
        }
    });
}

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

    // Get current date and time once and pass it to the individual runs for the image filename.
    let tm = now();
    let tm = tm.strftime("%Y_%m_%d__%H_%M_%S").unwrap();
    let time_now = format!("{}", &tm);

    // vec! macro expects usize
    let mut image: Vec<u32> = vec![0; (mandel_config.img_size * mandel_config.img_size) as usize];

    do_run("serial", &serial, &mandel_config, &mut image, &time_now);

    do_run("scoped_thread_pool", &scoped_thread_pool_, &mandel_config, &mut image, &time_now);

    do_run("simple_parallel", &simple_parallel_, &mandel_config, &mut image, &time_now);

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

    do_run("kirk_crossbeam", &kirk_crossbeam, &mandel_config, &mut image, &time_now);
}

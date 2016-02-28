// External crates
extern crate num;
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
use rayon::par_iter::*;
use kirk::crew::deque::Options;

// Internal modules
use mandel_util::{mandel_iter, MandelConfig};

// The serial version of the mandelbrot set calculation.
pub fn serial(mandel_config: &MandelConfig, image: &mut [u32]) {
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
pub fn scoped_thread_pool_(mandel_config: &MandelConfig, image: &mut [u32]) {
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
pub fn simple_parallel_(mandel_config: &MandelConfig, image: &mut [u32]) {
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
pub fn rayon_join(mandel_config: &MandelConfig, image: &mut [u32]) {
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
pub fn rayon_par_iter(mandel_config: &MandelConfig, image: &mut [u32]) {

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
pub fn rust_scoped_pool(mandel_config: &MandelConfig, image: &mut [u32]) {
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
pub fn job_steal(mandel_config: &MandelConfig, image: &mut [u32]) {
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
pub fn job_steal_join(mandel_config: &MandelConfig, image: &mut [u32]) {
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
pub fn kirk_crossbeam(mandel_config: &MandelConfig, image: &mut [u32]) {
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

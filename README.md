# mandel-rust: Mandelbrot set in rust

This code shows how to calculate the set in serial and parallel using Rust and various libraries.
More parallel versions (with different libraries) will be added in the future.

Written by Willi Kappler, License: MIT - Version 0.3 (2016.01.30)

![mandelbrot set](mandel.png)


Compile with:

    cargo build --release

Run with the default values:

    cargo run --release

Supported command line options:

        --img_size <IMAGE_SIZE>              size of image in pixel (square, default: 1024, must be a power of two)
        --img1 <IMAGINARY1>                  lower part (default: -1.50)
        --img2 <IMAGINARY2>                  upper part (default: 1.50)
        --write_metadata                     write metadata like run time into the ppm file (default: off)
        --no_ppm                             disable creation of the ppm file, just run the calculation (default: off)
        --bench                              use all available CPUs (default: off), will change in the future
        --max_iter <MAX_ITER>                maximum number of iterations (default: 2048)
        --num_threads <NUMBER_OF_THREADS>    number of threads to use (default: 2)
        --re1 <REAL1>                        left real part (default: -2.0)
        --re2 <REAL2>                        right real part (default: 1.0)

The main program runs the calculation 7 times: 1 x single threaded and currently 6 x multi threaded.
It writes the mandelbrot set out as PPM image files. For each method one image file is created.

To check if all the images are equal (and thus that all the computations are correct) you can use this command:

    for i in *.ppm; do md5sum $i; done

Or even better:

    for i in *.ppm; do md5sum $i; done | cut -c1-32 | uniq

(This works only if the flag `--write_metadata` has not been set)

# Used crates:
- Clap: command line parsing
- Time: timing each run
- Num: complex numbers
- Num_cpus: for the `--bench` flag, determine the total number of cpus
- Scoped threadpool: use scope and thread pool
- Simple parallel: use scope and thread pool
- Rayon: using recursive divide-and-conquer call with join, use par_iter_mut
- Rust scoped pool: use scope and thread pool
- Jobsteal: use scope and thread pool, use join (divide-and-conquer)
- Kirk + crossbeam: use scope and thread pool

# Benchmark
Measured on a Transtec server with the following specs:
- RAM: 32 GB
- CPU: 2 x Intel Xeon(R) CPU E5-2620 v3 @ 2.40GHz (12 Cores, with hyper threading 24 cores)
- Operating system: 64 bit Ubuntu Server 14.04
- Rust version: rustc 1.5.0 (3d7cd77e4 2015-12-04)
- Mandel configuration: re1: -2.00, re2: 1.00, img1: -1.50, img2: 1.50, max_iter: 2048, img_size: 1024


![mandelbrot benchmark plot](plot/mandel_bench.png)


(Note: that not all number of cores have been run in the benchmark)

Method | Number of threads | Time taken (in ms)
-------|-------------------|------------------------
serial | 1 | 1703.82371
scoped threadpool | 1 | 2283.06639
scoped threadpool | 8 | 393.57072
scoped threadpool | 24 | 169.02211
simple parallel | 1 | 2508.58119
simple parallel | 8 | 389.50966
simple parallel | 24 | 161.75248
rayon* v0.2 | 24 | 127.69423
rayon par_iter_mut* v0.2 | 24 | 106.66261
rust scoped pool | 1 | 2178.49247
rust scoped pool | 8 | 318.91450
rust scoped pool | 24 | 141.91438
jobsteal | 1 | 1143.25212
jobsteal | 8 | 314.39410
jobsteal | 24 | 135.46289

(*) Note that rayon uses whatever number of cores are available at the moment.

With just using one thread the overhead for both scoped thread pool, rust scoped pool and simple parallel is too high and thus they are slower than the serial version.
Using all cores (including virtual one due to hyper threading) rayon par_iter_mut is the fastest method. It uses explicit work stealing to utilize all the cores more efficiently.
The jobsteal crate also does a good job.

As always take these results with a grain of salt, they just show a general direction.
If in doubt just do run some benchmarks with different crates for your specific code (which is always a good thing to do).

# TODO:
- [ ] Check [ArrayFire](https://github.com/arrayfire/arrayfire-rust)
- [ ] Check [Collenchyma](https://github.com/autumnai/collenchyma)
- [ ] Check [Timely Dataflow](https://github.com/frankmcsherry/timely-dataflow)
- [ ] Check [Crossbeam](https://github.com/aturon/crossbeam)
- [x] Check [rust-scoped-pool](https://github.com/reem/rust-scoped-pool)
- [x] Check [jobsteal](https://github.com/rphmeier/jobsteal)
- [ ] Check [forkjoin](https://github.com/faern/forkjoin)
- [ ] Check [rust-stm](https://github.com/Marthog/rust-stm)
- [ ] Check [kirk](https://github.com/kinghajj/kirk)
- [ ] Use rust-fmt on source code (Thanks to matklad)
- [ ] Check docopt (instead of clap ? Thanks to matklad)

- [ ] Automate benchmark: re-run each test multiple times (user specified command line argument) and take the average
- [ ] Automate benchmark: write all results to text files and make a nice plot

- [ ] Use a bigger image size and a higher number of iterations for the next release

Any feedback is welcome!

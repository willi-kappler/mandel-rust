# mandel-rust: Mandelbrot set in rust

This code shows how to calculate the set in serial and parallel using Rust and various libraries.
More parallel versions (with different libraries) will be added in the future.

Written by Willi Kappler, License: MIT - Version 0.2 (2016.01.24)

![mandelbrot set](mandel.png)


Compile with:

    cargo build --release

Run with the default values:

    cargo run --release

Supported command line options:

        --img_size <IMAGE_SIZE>              size of image in pixel (square, default: 1024, must be a power of two)
        --img1 <IMAGINARY1>                  lower part (default: -1.50)
        --img2 <IMAGINARY2>                  upper part (default: 1.50)
        --max_iter <MAX_ITER>                maximum number of iterations (default: 2048)
        --num_threads <NUMBER_OF_THREADS>    number of threads to use (default: 2)
        --re1 <REAL1>                        left real part (default: -2.0)
        --re2 <REAL2>                        right real part (default: 1.0)

The main program runs the calculation 4 times: 1 x single threaded and currently 3 x multi threaded.
It writes the mandelbrot set out as PPN image files.

# Used crates:
- Clap: command line parsing
- Time: timing each run
- Num: complex numbers
- Scoped_threadpool: manual threading
- Simple_parallel: using parallel for loop
- Rayon: using recursive fork-join (divide-and-conquer) call

# Timging
Measured on a Transtec server with the following specs:
- RAM: 32 GB
- CPU: 2 x Intel Xeon(R) CPU E5-2620 v3 @ 2.40GHz (12 Cores, with hyper threading 24 cores)
- Operating system: 64 bit Ubuntu Server 14.04
- Rust version: rustc 1.5.0 (3d7cd77e4 2015-12-04)
- Mandel configuration: re1: -2.00, re2: 1.00, img1: -1.50, img2: 1.50, max_iter: 2048, img_size: 1024

Method | Number of threads | Time taken (in ms)
-------|-------------------|------------------------
serial | 1 | 1703.823711703.82371
scoped thread pool | 1 | 2283.06639
scoped thread pool | 2 | 1261.97402
scoped thread pool | 3 | 911.55601
scoped thread pool | 4 | 730.70325
scoped thread pool | 5 | 562.70050
scoped thread pool | 6 | 487.89133
scoped thread pool | 7 | 435.70740
scoped thread pool | 8 | 393.57072
scoped thread pool | 10 | 316.86398
scoped thread pool | 12 | 272.54214
scoped thread pool | 14 | 243.76645
scoped thread pool | 16 | 217.38483
scoped thread pool | 20 | 190.96532
scoped thread pool | 24 | 169.02211
simple parallel | 1 | 2508.58119
simple parallel | 2 | 1273.13910
simple parallel | 3 | 871.48674
simple parallel | 4 | 745.04818
simple parallel | 5 | 597.11188
simple parallel | 6 | 512.16674
simple parallel | 7 | 427.89391
simple parallel | 8 | 389.50966
simple parallel | 10 | 332.40532
simple parallel | 12 | 270.28192
simple parallel | 14 | 252.61965
simple parallel | 16 | 216.03580
simple parallel | 20 | 186.54049
simple parallel | 24 | 161.75248
rayon* | 24? | 139.04431

(*) Note that rayon uses whatever number of cores are available at the moment.

# TODO:
- [] Check ArrayFire
- [] Use rust-fmt on source code
- [] Check docopt (instead of clap ?)

Any feedback is welcome!

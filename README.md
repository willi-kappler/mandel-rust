# mandel-rust: Mandelbrot set in rust

This code shows how to calculate the set in serial and parallel.
More parallel versions will be added in the future.

Written by Willi Kappler, License: MIT - Version 0.1 (2015.12.28)

Compile with:

    cargo build --release

Run with the default values:

    cargo run --release
  
Supported command line options:

        --img_size <IMAGE_SIZE>              size of image in pixel (square, default: 1024)
        --img1 <IMAGINARY1>                  lower part (default: -1.50)
        --img2 <IMAGINARY2>                  upper part (default: 1.50)
        --max_iter <MAX_ITER>                maximum number of iterations (default: 2048)
        --num_threads <NUMBER_OF_THREADS>    number of threads to use (default: 2)
        --re1 <REAL1>                        left real part (default: -2.0)
        --re2 <REAL2>                        right real part (default: 1.0)

The main program runs the calculation three times: 1 x single threaded and currently 2 x multi threaded.
It writes the mandelbrot set out as PPN image files.

Feedback is welcome

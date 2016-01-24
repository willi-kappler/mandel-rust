#!/bin/bash

gnuplot <<PLOT
    set terminal png size 640,480
    set output "mandel_bench.png"
    set view map
    set xlabel "number of cores"
    set ylabel "time [ms]"
    set style data points
    set title "mandelbrot benchmark"
    set xtics 0,2,24
    set xrange [0:25]

    plot "serial.txt" title "serial", \
         "scoped_thread_pool.txt" title "scoped thread pool", \
         "simple_parallel.txt" title "simple parallel", \
         "rayon.txt" title "rayon"

PLOT

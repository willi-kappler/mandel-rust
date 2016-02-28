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
    set yrange [0:*]

    filenames = "job_steal_join job_steal kirk_crossbeam rayon_join rayon_par_iter rust_scoped_pool scoped_thread_pool serial simple_parallel"
    plot for [file in filenames] file.".txt" title file with errorbars
PLOT

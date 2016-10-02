#!/bin/bash

gnuplot <<PLOT
    set terminal png noenhanced size 800,600
    set output "mandel_bench1.png"
    set view map
    set xlabel "number of cores"
    set ylabel "time [ms]"
    set style data points
    set title "mandelbrot benchmark full"
    set xtics 0,2,24
    set mxtics 2
    set xrange [0:25]

    set yrange [500:*]
    set ytics 0,500
    set mytics 2

    filenames = "job_steal_join job_steal rayon_join rayon_par_iter rust_scoped_pool scoped_thread_pool serial"
    plot for [file in filenames] file.".txt" using 1:(\$1>0?\$2:1/0):3:4 title file with errorbars

    set output "mandel_bench2.png"
    set title "mandelbrot benchmark close up"
    set xrange [9:25]
    set yrange [500:1600]
    set xtics 10,2,24
    set ytics 0,100
    plot for [file in filenames] file.".txt" using 1:(\$1>9?\$2:1/0):3:4 title file with errorbars

PLOT

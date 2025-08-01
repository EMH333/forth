# Rust Forth

This was a short project of mine to create a basic implementation of Forth in Rust as a learning exercise. It is able to handle a fair number of forth "words" and is fairly fast and performant. There is clearly performance, memory, maintainability, and correctness left on the table, which I may fix at a later date.

I was pleasantly surprised at how straightforward it was to build out the initial interpreter. In the future, this may be a helpful example project to practice optimizing interpreters/compilers.

My goal was to beat the [Gforth](https://www.gnu.org/software/gforth/) implementation for the fizzbuzz program on my local machine. The Gforth implementation is able to handle about 30MiB/s as measured via `./gforth benchmark.forth | pv > /dev/null`.
My implementation (while implementing significantly fewer features) is able to achieve around 1.22GiB/s as measured via `./forth ../../test_files/fizzbuzz3.forth | pv > /dev/null` on a Ryzen 7900X (or ~70MiB/s with `test_files/benchmark.forth`). Compiled with flags: `RUSTFLAGS="-Ctarget-cpu=native" cargo build --profile optrelease`

One of the primary ways I was able to achieve this is by aggressively inlining function calls. More work is needed for this to maintain program correctness, but the basic implementation is working.

Profiling can be done with
```
perf record -g -p `pidof ./forth`
```
and then analyzing the data with `perf report -i perf.data`

If two arguments are provided to the program, it will attempt to output a C++ program from the forth code. When compiled, this will often result in a much faster program.
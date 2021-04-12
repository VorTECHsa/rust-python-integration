# Rust-Python Integration

An example for calling Rust code from Python, passing NumPy data and leveraging threads. This code is related to [this blog post](https://www.vortexa.com/insight/integrating-rust-into-python).

The code has been tested using Rust 1.51.0 and Python 3.7.9 on macOS Catalina.

## What the code does

This example takes some GPS signals from ships, compares them with five example polygons, and works out which signal is in which polygon. Rust is leveraged for speed and parallelism, but the real point of this code is to show how it can all be tied together using the [PyO3](https://github.com/PyO3/pyo3) Rust/Python bindings.

See this blog post LINK HERE for a detailed description.

## The example data

This is held in the data directory, and contains five geojson polygons along with a gzipped CSV file containing 42626 AIS signals.

## The Python code

Two files, `single.py` and `threaded.py` contain essentially the same code, but call two different Rust methods to demonstrate single-threaded and multi-threaded behaviour, the output results are identical. The dependencies are defined in `requirements.txt`.

The Python does the following:

* Loads the AIS data into Pandas, multiplying the records 1024x to get a good quantity of test data.
* Loads the five example geojson polygons
* Initiates profiling using [PyInstrument](https://github.com/joerick/pyinstrument)
* Instantiates the Rust `Engine` class, passing in the geojson as an array
* Extracts the coordinates as NumPy arrays
* Calls Rust to compare all the AIS data with all the polygons, getting a NumPy array back
* Plugs the results back into Pandas
* Stops profiling
* Prints some stats and the result distribution

Note the number returned from Rust is the polygon number, the index in the geojson array originally passed to Rust, or -1 if no result is found. Most signals do not match the polygons but the math still needs to be done.

## The Rust code

* Defines a constructor for type `Engine` which takes the geojson array from Python, parses it and stores the geometry
* The `pip_1` method compares one coordinate pair with the polygons, returning one result
* The `pip_n` method compares a NumPy array pair of coordinates with the polygons, calculating a `Vec<isize>` result before converting that to NumPy. It uses a single thread for the processing, and coordinates can be extracted from NumPy as processing proceeds.
* The `pip_n_threaded` method extracts the NumPy coordinates into `Vec<f64>` types, and then uses rayon to processes these in parallel. Finally the results are packed back into a `Vec<i32>` before being converted into the NumPy result.
* There is also code to define the Python module using Rust and PyO3

Note the use of PyO3 attributes.

## Building

[Install Rust](https://www.rust-lang.org/tools/install), and then run:

`cargo build  -- release`

Once built:

* On Linux, copy `target/release/libpip.so` to `pip.so`
* On macOS, copy `target/release/libpip.dylib` to `pip.so`
* On Windows (not tested), copy `target\release\libpip.dll` to `pip.pyd`

---

MIT License

Copyright (c) 2021 Vortexa Ltd

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.


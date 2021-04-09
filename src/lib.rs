/*
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
*/

use pyo3::prelude::*;
use rayon::prelude::*;

use geojson::quick_collection;
use geo_types::{Geometry, Coordinate, Point};
use geojson::GeoJson;
use geo::algorithm::contains::Contains;

use numpy::{PyArray, PyReadonlyArray1, ToPyArray};

#[pyclass]
struct Engine {
    // One Geometry per input GeoJson string
    polygons: Vec<Geometry<f64>>
}

#[pymethods]
impl Engine {
    /// Main constructor called by python, takes a geojson string array
    #[new]
    fn new(geometry: Vec<&str>) -> Self {
        // We use i32 to return the polygon number, ensure we don't have too many polygons
        if geometry.len() > i32::MAX as usize {
            panic!("Too many input polygons");
        }
        
        // Vec for our polygons to test
        let mut polygons = Vec::new();

        for geometry_str in &geometry {
            // Parse the GeoJson
            let geojson = geometry_str.parse::<GeoJson>().expect("Could not decode geojson");

            // Create a GeometryCollection from the contents
            let geometry_collection = quick_collection(&geojson).expect("Could not create GeometryCollection");

            // Ensure it contains only one geometry
            if geometry_collection.len() != 1 {
                panic!("Multiple polygons found in one geojson string, not supported");
            }

            // Store the polygon
            polygons.push(geometry_collection[0].clone());
        }

        println!("Built Engine with {} polygons", polygons.len());

        Engine { polygons }
    }

    /// Method for testing a single point against all our polygons
    /// 
    /// Returns the number of the polygon which had a hit, or -1 if no hit
    /// 
    /// Note this method can be called by Python directly
    fn pip_1(&self, lat: f64, lon: f64) -> i32 {
        let point = Point(Coordinate{y: lat, x: lon});

        // Iterate through our polygons, stopping at the first hit
        let result = self.polygons.iter()
            .position(|polygon| polygon.contains(&point));

        // Return the hit number, or -1 if nothing found
        match result {
            Some(index) => index as i32,
            None => -1
        }
    }

    /// Method for testing NumPy arrays of coordinates against all our polygons,
    /// using a single thread
    /// 
    /// Returns a NumPy array of polygon indexes, -1 values mean no match found
    /// 
    /// Type signature means two 1-dimenstional 64-bit floating point NumPy arrays in,
    /// and one 1-dimensional 32-bit signed integer NumPy array out
    fn pip_n<'py>(&self,
        py: Python<'py>,
        lat_array: PyReadonlyArray1<f64>,
        lon_array: PyReadonlyArray1<f64>) -> PyResult<&'py PyArray<i32, ndarray::Dim<[usize; 1]>>> {

        // Ensure the latitude and longitude counts match
        if lat_array.len() != lon_array.len() {
            panic!("Input arrays different lengths");
        }

        // Create a Vec for our results
        let mut results = Vec::with_capacity(lat_array.len());

        // Loop over all the coordinates
        for n in 0..lat_array.len() {
            let lat = *lat_array.get([n]).expect("Error extracting lat coordinate");
            let lon = *lon_array.get([n]).expect("Error extracting lon coordinate");

            // Check if we're in any polygon
            let result = self.pip_1(lat, lon);

            // Store the result
            results.push(result);
        }

        // Convert all results back into a NumPy array
        Ok(results.to_pyarray(py))
    }


    /// Method for testing NumPy arrays of coordinates against all our polygons,
    /// using multiple threads
    /// 
    /// Returns a NumPy array of polygon indexes, -1 values mean no match found
    /// 
    /// Type signature means two 1-dimenstional 64-bit floating point NumPy arrays in,
    /// and one 1-dimensional 32-bit signed integer NumPy array out
    fn pip_n_threaded<'py>(&self,
        py: Python<'py>,
        lat_array: PyReadonlyArray1<f64>,
        lon_array: PyReadonlyArray1<f64>) -> PyResult<&'py PyArray<i32, ndarray::Dim<[usize; 1]>>> {

        // Ensure the latitude and longitude counts match
        if lat_array.len() != lon_array.len() {
            panic!("Input arrays different lengths");
        }

        // Using one thread (as we're accessing Python data), convert the incoming
        // coordinate NumPy arrays into a Vec of tuples, with the index, latitude
        // and longitude values per entry.
        let input: Vec<(usize, f64, f64)> = (0..lat_array.len()).into_iter()
            .map(|n| (n,
                *lat_array.get([n]).expect("Error extracting lat coordinate"),
                *lon_array.get([n]).expect("Error extracting lon coordinate")))
            .collect();

        // Using threads, in parallel process all the coordinates checking against
        // all the polygons. Remember which result was for which coordinate index,
        // and filter out the entries with no match (result < 0)
        let shuffled: Vec<(usize, i32)> = input.into_par_iter()
            .map(|(n, lat, lon)| (n, self.pip_1(lat, lon)))
            .filter(|(_, result)| *result >= 0)
            .collect();

        // Create a Vec for all the results, and pre-populate it with -1,
        // i.e. no result found
        let mut results = Vec::with_capacity(lat_array.len());
        results.resize(lat_array.len(), -1);

        // Loop over the results where we did find something, setting the value
        // in the results Vec at the correct index
        for (n, result) in shuffled {
            results[n] = result;
        }

        // Return the results Vec as a NumPy array
        Ok(results.to_pyarray(py))
    }
}

/// Implements the Python module pip, registers the class Engine
#[pymodule]
fn pip(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Engine>()?;

    Ok(())
}
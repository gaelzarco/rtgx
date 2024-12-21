use std::fs::File;
use std::io::{BufRead, BufReader};

/// Stores 3 vertex values (x, y, z)
#[derive(Debug)]
pub struct Vertex(pub f32, pub f32, pub f32);

/// Stores 2D coordinates, x and y in that order
/// #[derive(Debug)]
/// pub struct Coordinate(pub i32, pub i32);

/// Load Geometry into memory
///
/// Takes in a file path and returns a tuple of vectors
///
/// The first is a vector of geometric vertex coordinates
/// The second is a matrix of face element coordinate vectors
///
/// This fuction will panic if the file_path specified cannot be found or the file cannot be opened.
pub fn load_obj(file_path: &str) -> (Vec<Vertex>, Vec<Vec<usize>>) {
    let mut vertices: Vec<Vertex> = vec![];
    let mut faces = Vec::new();

    let file = match File::open(file_path) {
        Ok(f) => f,
        Err(_) => panic!("ERROR: Could not find/open specified file"),
    };
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.unwrap();
        let mut parts = line.split_whitespace();

        match parts.next() {
            Some("v") => {
                let x = parts.next().unwrap().parse::<f32>().unwrap();
                let y = parts.next().unwrap().parse::<f32>().unwrap();
                let z = parts.next().unwrap().parse::<f32>().unwrap();

                vertices.push(Vertex(x, y, z));
            }
            Some("f") => {
                // Face vertex indices start from 1 so sub 1
                // Retrieves the vertex coordinate of the first value in each vert y, texture coor y, and normal y pair of a triangle face
                let face: Vec<usize> = parts
                    .map(|part| part.split('/').next().unwrap().parse::<usize>().unwrap() - 1)
                    .collect();

                faces.push(face);
            }
            _ => {}
        }
    }

    (vertices, faces)
}

/// Transform 3D coordinates to 2D space
///
/// Takes in specified vertex and the canvas width/height to transform coordinates to 2D space
///
/// Translates normalized x and y vertex coordinates to match 2D origin and scales them to resolution
pub fn three_to_canvas(v: &Vertex, width: usize, height: usize) -> (i32, i32) {
    let x = ((v.0 + 1.0) * width as f32 / 2.0) as i32;
    let y = ((v.1 + 1.0) * height as f32 / 2.0) as i32;
    (x, y)
}

/// Draws a 2D triangle onto the screen
///
/// Bubblesorts triangle coordinates and plots them on screen
///
/// Interpolates values between triangle edges to fill in the shape using line()
///
/// This function panics if 0 integer division is possible
pub fn triangle(
    mut v0: (i32, i32),
    mut v1: (i32, i32),
    mut v2: (i32, i32),
    canvas: &mut Vec<u32>,
    width: usize,
    height: usize,
    color: u32,
) {
    // Bubblesort coordinates by y-axis (ascending)
    if v0.1 > v1.1 {
        std::mem::swap(&mut v0, &mut v1);
    }
    if v0.1 > v2.1 {
        std::mem::swap(&mut v0, &mut v2);
    }
    if v1.1 > v2.1 {
        std::mem::swap(&mut v1, &mut v2);
    }

    // Total height of triangle
    let total_height = v2.1 - v0.1;

    // Draw triangle
    for y in v0.1..=v2.1 {
        // Determine which half we are in. (v0, v1) is bottom half. Top half is (v1, v2).
        let second_half = y > v1.1 || v1.1 == v0.1;
        // Segment height for the current half
        let segment_height = if second_half {
            v2.1 - v1.1
        } else {
            v1.1 - v0.1
        };

        if segment_height == 0 {
            panic!("ERROR: Segment height is 0 which results in division by 0\nWhile drawing triangle\nv0({}, {}) v1({}, {}) v2({}, {})", v0.0, v0.1, v1.0, v1.1, v2.0, v2.1);
            // Prevent division by zero
        }

        // These factors enable liner interpolation to calculate endpoints of current line
        // Determines horizontal pos of point on line that connects to top v0 and bottom v2 vertices 
        let alpha = (y - v0.1) as f32 / total_height as f32;
        // Determines horizontal pos of point on line that connects curr half (v0 -> v1) or (v1 -> v2)
        let beta = if second_half {
            (y - v1.1) as f32 / segment_height as f32
        } else {
            (y - v0.1) as f32 / segment_height as f32
        };

        // Point on line connecting top (v0) and bottom (v2) vertices for curr y
        let mut a = ((v0.0 as f32 + (v2.0 - v0.0) as f32 * alpha) as i32, y);
        // Point on line connecting vertices of curr half for curr y
        let mut b = if second_half {
            ((v1.0 as f32 + (v2.0 - v1.0) as f32 * beta) as i32, y)
        } else {
            ((v0.0 as f32 + (v1.0 - v0.0) as f32 * beta) as i32, y)
        };

        // Keep a on left
        if a.0 > b.0 {
            std::mem::swap(&mut a, &mut b);
        }

        // Draw horizontal line using crate::line
        crate::line(a.0, a.1, b.0, b.1, canvas, width, height, color);
    }
}

use crate::video_pipeline::Frame;
use std::collections::VecDeque;

use super::double_thresholding::{MeasuredPixel, Strength};

pub fn eight_conn_edge_tracker_hysteris(mut pixels: Vec<Vec<MeasuredPixel>>) -> Frame {
    let height = pixels.len() as i32;
    let width = pixels[0].len() as i32;
    let mut output_data = vec![0u8; (width * height) as usize];
    let mut visited = vec![vec![false; width as usize]; height as usize];
    let mut queue = VecDeque::new();

    // First pass: Add all strong pixels to queue and mark them with their original value
    for y in 0..height {
        for x in 0..width {
            if pixels[y as usize][x as usize].weight == Strength::Strong {
                queue.push_back((x, y));
                visited[y as usize][x as usize] = true;
                let idx = (y * width + x) as usize;
                // Use original pixel value for strong pixels
                output_data[idx] = pixels[y as usize][x as usize].value as u8;
            }
        }
    }

    let neighbors = [
        (-1, -1),
        (0, -1),
        (1, -1),
        (-1, 0),
        (1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ];

    while let Some((x, y)) = queue.pop_front() {
        for (dx, dy) in neighbors.iter() {
            let new_x = x + dx;
            let new_y = y + dy;

            if new_x < 0 || new_x >= width || new_y < 0 || new_y >= height {
                continue;
            }

            if visited[new_y as usize][new_x as usize] {
                continue;
            }

            let pixel = &pixels[new_y as usize][new_x as usize];
            if pixel.weight == Strength::Weak {
                visited[new_y as usize][new_x as usize] = true;
                queue.push_back((new_x, new_y));

                let idx = (new_y * width + new_x) as usize;
                // Use the original pixel value for weak pixels
                output_data[idx] = pixel.value as u8;
            }
        }
    }

    // We can drop pixels row by row as we process them
    for row in pixels.drain(..) {
        drop(row);
    }
    // Explicitly drop the outer vector
    drop(pixels);

    Frame {
        data: output_data,
        width,
        height,
        channels: 1,
    }
}

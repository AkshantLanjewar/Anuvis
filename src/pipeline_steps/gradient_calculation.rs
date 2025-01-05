use crate::video_pipeline::Frame;
use std::f32::consts::PI;

#[derive(Clone)]
pub struct PixelGradient {
    pub magnitude: f32,
    pub direction: f32,
}

impl PixelGradient {
    pub fn new(mag: f32, dir: f32) -> Self {
        Self {
            magnitude: mag,
            direction: dir,
        }
    }
}

pub struct SobelOperator {
    kernel_x: [[i32; 3]; 3],
    kernel_y: [[i32; 3]; 3],
}

impl SobelOperator {
    pub fn new() -> Self {
        Self {
            // Sobel kernels for x and y directions
            kernel_x: [[-1, 0, 1], [-2, 0, 2], [-1, 0, 1]],
            kernel_y: [[-1, -2, -1], [0, 0, 0], [1, 2, 1]],
        }
    }

    fn apply_kernels(&self, frame: &Frame, x: i32, y: i32) -> (i32, i32) {
        let mut gx = 0;
        let mut gy = 0;

        // Apply both kernels simultaneously
        for ky in 0..3 {
            for kx in 0..3 {
                // Get the pixel value from the 3x3 neighborhood
                if let Some((value, _, _)) =
                    frame.get_pixel(x + (kx as i32 - 1), y + (ky as i32 - 1))
                {
                    gx += value * self.kernel_x[ky][kx];
                    gy += value * self.kernel_y[ky][kx];
                }
            }
        }

        (gx, gy)
    }

    pub fn calculate_gradient(frame: &Frame) -> Vec<Vec<PixelGradient>> {
        // Ensure the frame is grayscale
        if frame.channels != 1 {
            panic!("Frame must be grayscale (1 channel) for Sobel operator");
        }

        let height = frame.height as usize;
        let width = frame.width as usize;

        // Initialize output gradients matrix
        let mut gradients = vec![vec![PixelGradient::new(0.0, 0.0); width]; height];

        // Create Sobel operator with kernels
        let sobel = SobelOperator::new();

        // Calculate gradients for each pixel (excluding borders)
        for y in 1..height - 1 {
            for x in 1..width - 1 {
                let (gx, gy) = sobel.apply_kernels(frame, x as i32, y as i32);

                // Calculate magnitude and direction
                let magnitude = ((gx * gx + gy * gy) as f32).sqrt();

                // Calculate direction in radians, handle division by zero
                let direction = if gx == 0 {
                    if gy == 0 {
                        0.0
                    } else {
                        PI / 2.0 * gy.signum() as f32
                    }
                } else {
                    (gy as f32).atan2(gx as f32)
                };

                gradients[y][x] = PixelGradient::new(magnitude, direction);
            }
        }

        gradients
    }
}

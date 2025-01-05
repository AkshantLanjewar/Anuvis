use crate::video_pipeline::Frame;

use super::gradient_calculation::PixelGradient;

pub struct GradNonMaxSuppression {}

impl GradNonMaxSuppression {
    pub fn suppress(gradients: Vec<Vec<PixelGradient>>) -> Frame {
        let height = gradients.len() as i32;
        let width = gradients[0].len() as i32;

        // create output buffer for final image
        let size = (width * height) as usize;
        let mut output_data = vec![0u8; size];

        // Process all pixels except borders
        for y in 1..(height - 1) {
            for x in 1..(width - 1) {
                let current = &gradients[y as usize][x as usize];

                // Normalize angle to 0-180 degrees
                // We use modulo to handle negative angles and angles > 180
                let mut angle = current.direction % 180.0;
                if angle < 0.0 {
                    angle += 180.0;
                }

                // Initialize neighbor magnitudes
                let mut neighbor1_mag = 0.0;
                let mut neighbor2_mag = 0.0;

                // Select neighbors based on gradient direction
                // Horizontal edge (angle ~ 0° or 180°)
                if (0.0 <= angle && angle < 22.5) || (157.5 <= angle && angle <= 180.0) {
                    neighbor1_mag = gradients[y as usize][(x - 1) as usize].magnitude;
                    neighbor2_mag = gradients[y as usize][(x + 1) as usize].magnitude;
                }
                // Diagonal edge (angle ~ 45°)
                else if 22.5 <= angle && angle < 67.5 {
                    neighbor1_mag = gradients[(y - 1) as usize][(x + 1) as usize].magnitude;
                    neighbor2_mag = gradients[(y + 1) as usize][(x - 1) as usize].magnitude;
                }
                // Vertical edge (angle ~ 90°)
                else if 67.5 <= angle && angle < 112.5 {
                    neighbor1_mag = gradients[(y - 1) as usize][x as usize].magnitude;
                    neighbor2_mag = gradients[(y + 1) as usize][x as usize].magnitude;
                }
                // Diagonal edge (angle ~ 135°)
                else if 112.5 <= angle && angle < 157.5 {
                    neighbor1_mag = gradients[(y - 1) as usize][(x - 1) as usize].magnitude;
                    neighbor2_mag = gradients[(y + 1) as usize][(x + 1) as usize].magnitude;
                }

                // If current pixel is local maximum, keep its magnitude
                let idx = (y * width + x) as usize;
                if current.magnitude >= neighbor1_mag && current.magnitude >= neighbor2_mag {
                    // Convert f32 magnitude to u8 for the output Frame
                    // Clamp value between 0 and 255
                    output_data[idx] = (current.magnitude.min(255.0).max(0.0)) as u8;
                }
                // Otherwise, magnitude remains 0 (suppressed)
            }
        }

        // create and return the frame
        Frame {
            data: output_data,
            width,
            height,
            channels: 1,
        }
    }
}

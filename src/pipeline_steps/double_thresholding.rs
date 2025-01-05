use crate::video_pipeline::Frame;

#[derive(PartialEq)]
pub enum Strength {
    Strong,
    Weak,
    Suppressed,
}

pub struct MeasuredPixel {
    pub weight: Strength,
    pub value: i32,
}

pub struct DoubleThresholder {
    pub min: i32,
    pub max: i32,
}

impl DoubleThresholder {
    pub fn new(min: i32, max: i32) -> Self {
        Self { min, max }
    }

    pub fn threshold(&self, frame: Frame) -> Vec<Vec<MeasuredPixel>> {
        let mut result = Vec::with_capacity(frame.height as usize);

        for y in 0..frame.height {
            let mut row = Vec::with_capacity(frame.width as usize);

            for x in 0..frame.width {
                // Get pixel value - we'll use the first channel since we expect grayscale
                let pixel = frame
                    .get_pixel(x, y)
                    .map(|(v, _, _)| v) // Take first channel value
                    .unwrap_or(0); // Default to 0 if out of bounds

                // Determine strength based on thresholds
                let weight = if pixel <= self.min {
                    Strength::Suppressed
                } else if pixel >= self.max {
                    Strength::Strong
                } else {
                    Strength::Weak
                };

                // Create measured pixel
                let measured = MeasuredPixel {
                    weight,
                    value: pixel,
                };

                row.push(measured);
            }

            result.push(row);
        }

        result
    }
}

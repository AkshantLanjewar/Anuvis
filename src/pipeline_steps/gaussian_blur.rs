use crate::frame_pipeline::PipelineStep;
use crate::video_pipeline::Frame;

use std::io;

#[derive(Debug)]
pub enum BlurError {
    InvalidSigma(String),
    InvalidDimensions(String),
    ProcessingError(String),
    EmptyInput(String),
}

impl std::fmt::Display for BlurError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlurError::InvalidSigma(msg) => write!(f, "Invalid sigma value: {}", msg),
            BlurError::InvalidDimensions(msg) => write!(f, "Invalid dimensions: {}", msg),
            BlurError::ProcessingError(msg) => write!(f, "Processing error: {}", msg),
            BlurError::EmptyInput(msg) => write!(f, "Empty input: {}", msg),
        }
    }
}

impl std::error::Error for BlurError {}

impl From<BlurError> for io::Error {
    fn from(error: BlurError) -> Self {
        io::Error::new(io::ErrorKind::Other, error.to_string())
    }
}

pub struct GaussianBlur {
    output_dir: String,
    kernel: Vec<f32>,
    radius: usize,
}

impl GaussianBlur {
    /// Create a new GaussianBlur step
    ///
    /// # Arguments
    /// * `output_dir` - The directory to store debug output and intermediate results
    /// * `sigma` - The standard deviation of the Gaussian kernel (determines how smooth the blur is)
    pub fn new(output_dir: &str, sigma: f32) -> Result<Self, BlurError> {
        // calculate the kernel radius
        let radius = (3.0 * sigma).ceil() as usize;
        let size = 2 * radius + 1;
        let mut kernel = Vec::with_capacity(size);

        // Calculate kernel values
        let two_sigma_sq = 2.0 * sigma * sigma;
        let mut sum = 0.0;

        for i in 0..size {
            let x = (i as i32 - radius as i32) as f32;
            let g = (-x * x / two_sigma_sq).exp();
            kernel.push(g);
            sum += g;
        }

        // Check for numerical stability
        if sum.abs() < f32::EPSILON {
            return Err(BlurError::ProcessingError(
                "Kernel sum too close to zero".to_string(),
            ));
        }

        // Normalize kernel
        for val in kernel.iter_mut() {
            *val /= sum;
        }

        Ok(Self {
            output_dir: output_dir.to_string(),
            kernel,
            radius,
        })
    }

    #[inline(always)]
    fn horizontal_pass(
        &self,
        input: &[u8],
        output: &mut [u8],
        width: usize,
        height: usize,
    ) -> Result<(), BlurError> {
        if input.len() != width * height {
            return Err(BlurError::InvalidDimensions(format!(
                "Input length {} does not match dimensions {}x{}",
                input.len(),
                width,
                height
            )));
        }

        for y in 0..height {
            let row = y * width;
            for x in 0..width {
                let mut sum = 0.0;
                for (i, &k) in self.kernel.iter().enumerate() {
                    let src_x = x.saturating_add(i).saturating_sub(self.radius);
                    if src_x >= width {
                        continue;
                    }
                    sum += input[row + src_x] as f32 * k;
                }
                output[row + x] = sum.clamp(0.0, 255.0) as u8;
            }
        }

        Ok(())
    }

    #[inline(always)]
    fn vertical_pass(
        &self,
        input: &[u8],
        output: &mut [u8],
        width: usize,
        height: usize,
    ) -> Result<(), BlurError> {
        if input.len() != width * height {
            return Err(BlurError::InvalidDimensions(format!(
                "Input length {} does not match dimensions {}x{}",
                input.len(),
                width,
                height
            )));
        }

        for x in 0..width {
            for y in 0..height {
                let mut sum = 0.0;
                for (i, &k) in self.kernel.iter().enumerate() {
                    let src_y = y.saturating_add(i).saturating_sub(self.radius);
                    if src_y >= height {
                        continue;
                    }
                    sum += input[src_y * width + x] as f32 * k;
                }
                output[y * width + x] = sum.clamp(0.0, 255.0) as u8;
            }
        }

        Ok(())
    }
}

impl PipelineStep for GaussianBlur {
    fn process(&self, frame: &mut Frame, _frame_count: u32) -> io::Result<()> {
        // Input validation
        if frame.width <= 0 || frame.height <= 0 {
            return Err(BlurError::InvalidDimensions(format!(
                "Invalid frame dimensions: {}x{}",
                frame.width, frame.height
            ))
            .into());
        }

        if frame.data.is_empty() {
            return Err(BlurError::EmptyInput("Frame data is empty".to_string()).into());
        }

        // Convert to grayscale
        frame.to_grayscale();

        let width = frame.width as usize;
        let height = frame.height as usize;

        // Create temporary buffers that will be dropped at end of scope
        let mut temp = vec![0u8; width * height];

        // Process horizontal pass
        self.horizontal_pass(&frame.data, &mut temp, width, height)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

        // Update frame data
        frame.data.clear();
        frame.data.reserve_exact(width * height);
        frame.data.resize(width * height, 0);

        // Process vertical pass
        self.vertical_pass(&temp, &mut frame.data, width, height)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

        Ok(())
    }

    fn name(&self) -> &str {
        "GaussianBlur"
    }
}

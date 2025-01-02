use crate::frame_pipeline::PipelineStep;
use crate::video_pipeline::Frame;
use super::gaussian_blur::GaussianBlur;

use std::io;

pub struct CannyEdgeDetection {
    /// Directory to store debug output and intermediate results
    output_dir: String,
    gaussian: GaussianBlur
}

impl CannyEdgeDetection {
    pub fn new(output_dir: &str) -> io::Result<Self> {
        let gaussian = GaussianBlur::new(output_dir, 3.0)?;

        Ok(Self {
            output_dir: output_dir.to_string(),
            gaussian: gaussian
        })
    }
}

impl PipelineStep for CannyEdgeDetection {
    fn process(&self, frame: Frame, frame_count: u32) -> io::Result<Frame> {
        // create internal frame pipeline
        let nframe = self.gaussian.process(frame, frame_count)?;
        //let nframe = frame;

        Ok(nframe)
    }

    fn name(&self) -> &str {
        "CannyEdgeDetection"
    }
}

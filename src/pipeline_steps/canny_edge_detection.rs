use crate::frame_pipeline::PipelineStep;
use crate::video_pipeline::Frame;
use super::double_thresholding::DoubleThresholder;
use super::eight_conn_edge_tracker::eight_conn_edge_tracker_hysteris;
use super::gaussian_blur::GaussianBlur;
use super::gradient_calculation::SobelOperator;
use super::non_max_suppression::GradNonMaxSuppression;

use std::io;

pub struct CannyEdgeDetection {
    /// Directory to store debug output and intermediate results
    output_dir: String,
    gaussian: GaussianBlur,
}

impl CannyEdgeDetection {
    pub fn new(output_dir: &str) -> io::Result<Self> {
        let gaussian = GaussianBlur::new(output_dir, 3.0)?;

        Ok(Self {
            output_dir: output_dir.to_string(),
            gaussian: gaussian,
        })
    }
}

impl PipelineStep for CannyEdgeDetection {
    fn process(&self, frame: &mut Frame, frame_count: u32) -> io::Result<()> {
        // create internal frame pipeline
        // step 1, gaussian noise reduction
        self.gaussian.process(frame, frame_count)?;
        // step 2, calculate gradients
        let gradients = SobelOperator::calculate_gradient(
            frame
        );
        // step 3, non max suppression of gradients back into a frame
        *frame = GradNonMaxSuppression::suppress(gradients);
        // step 4, double thresholding
        let thresholder = DoubleThresholder::new(10, 40);
        let thresholded = thresholder.threshold(frame); // NOTE: remove the clone
        // step 5, hysteria edge tracking
        *frame = eight_conn_edge_tracker_hysteris(thresholded);
        
        //let nframe = frame;

        Ok(())
    }

    fn name(&self) -> &str {
        "CannyEdgeDetection"
    }
}

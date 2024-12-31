use crate::video_pipeline::Frame;
use std::{io, path::PathBuf};

// trait for a step within the frame pipeline
/// A trait representing a single step in a machine vision processing pipeline.
/// Each step takes a frame as input, processes it, and returns a modified frame.
pub trait PipelineStep {
    /// Process a single frame, applying this step's machine vision algorithm
    ///
    /// # Arguments
    /// * `frame` - The input frame to process
    ///
    /// # Returns
    /// * `io::Result<Frame>` - The processed frame or an error
    fn process(&self, frame: &Frame) -> io::Result<Frame>;

    /// Get the name of this pipeline step for debugging and logging
    fn name(&self) -> &str;
}

/// A pipeline that runs a sequence of machine vision processing steps on video frames.
/// Supports debugging output of intermediate results between steps.
pub struct FramePipeline {
    /// The ordered sequence of processing steps to apply
    steps: Vec<Box<dyn PipelineStep>>,
    /// Directory to store debug output and intermediate results
    output_dir: String,
    /// Whether to save debug output after each step
    debug: bool,
}

impl FramePipeline {
    pub fn new(output_dir: &str) -> io::Result<Self> {
        // Create directory if it doesn't exist, or clean it if it does
        if std::path::Path::new(output_dir).exists() {
            // Remove all contents of directory
            for entry in std::fs::read_dir(output_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    std::fs::remove_dir_all(path)?;
                } else {
                    std::fs::remove_file(path)?;
                }
            }
        } else {
            // Create new directory
            std::fs::create_dir_all(output_dir)?;
        }

        Ok(Self {
            steps: Vec::new(),
            output_dir: output_dir.to_string(),
            debug: false,
        })
    }

    pub fn add_step<T: PipelineStep + 'static>(&mut self, step: T) {
        self.steps.push(Box::new(step));
    }

    pub fn set_debug(&mut self, debug: bool) {
        self.debug = debug;
    }

    pub fn process_frame(&mut self, frame: &Frame, frame_count: u32) -> io::Result<Frame> {
        let mut current_frame = frame.clone();

        // Create frame-specific output directory
        let frame_dir = PathBuf::from(&self.output_dir)
            .join(format!("frame_{:08}_output", frame_count));

        std::fs::create_dir_all(&frame_dir)?;

        // Process through each step
        for (index, step) in self.steps.iter().enumerate() {
            if self.debug {
                println!("Executing step {}: {}", index + 1, step.name());
            }

            // Process the frame through this step
            current_frame = step.process(&current_frame)?;

            // If in debug mode, save intermediate results
            if self.debug {
                let debug_path = frame_dir.join(format!(
                    "debug_step_{}_{}_{:08}.png",
                    index + 1,
                    step.name(),
                    frame_count
                ));

                current_frame.save(&debug_path)?;
            }
        }

        // Save the final processed frame
        let frame_path = frame_dir.join(format!("frame_{:08}.png", frame_count));

        current_frame.save(&frame_path)?;

        Ok(current_frame)
    }
}

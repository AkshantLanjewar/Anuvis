use gst::prelude::*;
use gstreamer as gst;
use gstreamer_app::{self as gst_app, AppSink};
use std::thread;
use std::time::Duration;
use std::io;
use std::path::PathBuf;
use std::fs;
use image::{ImageBuffer, Rgb};

#[derive(Debug, Clone)]
pub struct Frame {
    pub data: Vec<u8>,
    pub width: i32,
    pub height: i32,
    pub channels: i32,
}

impl Frame {
    pub fn get_pixel(&self, x: i32, y: i32) -> Option<(i32, i32, i32)> {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            return None;
        }

        // Calculate position in the data array
        // Each pixel is 3 bytes (RGB), and we need to convert 2D coordinates to 1D array index
        let index = (y * self.width * 3 + x * 3) as usize;

        if index + 2 >= self.data.len() {
            return None;
        }

        Some((
            self.data[index] as i32,     // Red
            self.data[index + 1] as i32, // Green
            self.data[index + 2] as i32, // Blue
        ))
    }

    pub fn print_pixel(&self, x: i32, y: i32) {
        match self.get_pixel(x, y) {
            Some((r, g, b)) => println!(
                "Pixel at ({}, {}): RGB({}, {}, {}) - Hex: #{:02X}{:02X}{:02X}",
                x, y, r, g, b, r as u8, g as u8, b as u8
            ),
            None => println!("Pixel position ({}, {}) is out of bounds", x, y),
        }
    }

    pub fn save(&self, path: &PathBuf) -> io::Result<()> {
        // Ensure we have RGB data
        if self.channels != 3 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Expected RGB frame (3 channels), got {} channels", self.channels)
            ));
        }

        // Convert to ImageBuffer
        let img_buffer = ImageBuffer::<Rgb<u8>, _>::from_raw(
            self.width as u32,
            self.height as u32,
            self.data.clone()
        ).ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData,
            "Failed to create image buffer"
        ))?;

        // Save as PNG
        img_buffer.save(path).map_err(|e| io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to save image: {}", e)
        ))
    }
}

// gstreamer pipeline to handle video processing frame by frame
pub struct VideoPipeline {
    // gstreamer pipeline to handle video processing
    pipeline: gst::Pipeline,

    // gstreamer appsink to handle video output
    appsink: gst_app::AppSink,

    // width of the video
    width: i32,

    // height of the video
    height: i32,
}

impl VideoPipeline {
    // create a new video pipeline
    pub fn new(input: &str) -> Result<Self, gst::glib::Error> {
        // init gstreamer if not already initialized
        gst::init()?;

        // create pipleine
        let pipeline = gst::Pipeline::new();

        // create elements
        let src = gst::ElementFactory::make_with_name("filesrc", None).map_err(|_e| {
            gst::glib::Error::new(
                gst::LibraryError::Failed,
                "Failed to create filesrc element",
            )
        })?;
        let demux = gst::ElementFactory::make_with_name("matroskademux", None).map_err(|_e| {
            gst::glib::Error::new(
                gst::LibraryError::Failed,
                "Failed to create demuxer element",
            )
        })?;
        let decode = gst::ElementFactory::make_with_name("decodebin", None).map_err(|_e| {
            gst::glib::Error::new(
                gst::LibraryError::Failed,
                "Failed to create decodebin element",
            )
        })?;
        let convert = gst::ElementFactory::make_with_name("videoconvert", None).map_err(|_e| {
            gst::glib::Error::new(
                gst::LibraryError::Failed,
                "Failed to create convert element",
            )
        })?;

        // build the caps for RGB only, no size specified
        let caps = gst::Caps::builder("video/x-raw")
            .field("format", &"RGB")
            .build();

        // create appsink
        let sink: AppSink = gst_app::AppSink::builder()
            .name("appsink")
            .caps(&caps)
            .build();

        // set the video file path
        src.set_property("location", input);

        // add elements to pipeline
        pipeline
            .add_many(&[&src, &demux, &decode, &convert, &sink.upcast_ref()])
            .map_err(|_e| {
                gst::glib::Error::new(
                    gst::LibraryError::Failed,
                    "Failed to add elements to pipeline",
                )
            })?;

        // link elements
        src.link(&demux).map_err(|_e| {
            gst::glib::Error::new(gst::LibraryError::Failed, "Failed to link src to demux")
        })?;
        convert.link(&sink).map_err(|_e| {
            gst::glib::Error::new(gst::LibraryError::Failed, "Failed to link convert to sink")
        })?;

        // connect to demux pad-added signal
        let decode_weak = decode.downgrade();
        demux.connect_pad_added(move |_demux, pad| {
            if let Some(decode) = decode_weak.upgrade() {
                let sink_pad = decode.static_pad("sink").unwrap();
                let _ = pad.link(&sink_pad);
            }
        });

        // Connect to decoder's pad-added signal
        let convert_weak = convert.downgrade();
        decode.connect_pad_added(move |_, src_pad| {
            if let Some(convert) = convert_weak.upgrade() {
                let sink_pad = convert.static_pad("sink").unwrap();
                let _ = src_pad.link(&sink_pad);
            }
        });

        // Create pipeline instance
        let mut pipeline = VideoPipeline {
            pipeline,
            appsink: sink,
            width: 0,
            height: 0,
        };

        // Start pipeline temporarily to get video info
        pipeline
            .pipeline
            .set_state(gst::State::Playing)
            .map_err(|_e| {
                gst::glib::Error::new(
                    gst::LibraryError::Failed,
                    "Failed to set pipeline state to playing",
                )
            })?;

        // Wait for up to 5 seconds for the first frame
        let start_time = std::time::Instant::now();
        let timeout = Duration::from_secs(5);

        while start_time.elapsed() < timeout {
            match pipeline
                .appsink
                .try_pull_sample(gst::ClockTime::from_mseconds(100))
            {
                Some(sample) => {
                    if let Some(caps) = sample.caps() {
                        let structure = caps.structure(0).unwrap();
                        pipeline.width = structure.get::<i32>("width").unwrap();
                        pipeline.height = structure.get::<i32>("height").unwrap();

                        break;
                    }
                }
                None => {
                    thread::sleep(Duration::from_millis(100));
                }
            }
        }

        if pipeline.width == 0 || pipeline.height == 0 {
            return Err(gst::glib::Error::new(
                gst::LibraryError::Failed,
                "Failed to get video info",
            ));
        }

        // reset pipeline state
        pipeline.pipeline.set_state(gst::State::Null).map_err(|_e| {
            gst::glib::Error::new(
                gst::LibraryError::Failed,
                "Failed to set pipeline state to null",
            )
        })?;

        Ok(pipeline)
    }

    pub fn start(&self) -> Result<(), gst::glib::Error> {
        self.pipeline.set_state(gst::State::Playing).map_err(|_e| {
            gst::glib::Error::new(gst::LibraryError::Failed, "Failed to start pipeline")
        })?;

        Ok(())
    }

    pub fn stop(&self) -> Result<(), gst::glib::Error> {
        self.pipeline.set_state(gst::State::Null).map_err(|_e| {
            gst::glib::Error::new(gst::LibraryError::Failed, "Failed to stop pipeline")
        })?;

        Ok(())
    }

    pub fn get_dimensions(&self) -> (i32, i32) {
        (self.width, self.height)
    }

    pub fn next_frame(&self) -> Option<Frame> {
        self.appsink
            .try_pull_sample(gst::ClockTime::from_seconds(5))
            .map(|sample| {
                let buffer = sample.buffer().unwrap();
                let map = buffer.map_readable().unwrap();

                Frame {
                    data: map.as_slice().to_vec(),
                    width: self.width,
                    height: self.height,
                    channels: 3, // RGB format
                }
            })
    }
}

impl Drop for VideoPipeline {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

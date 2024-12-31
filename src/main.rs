mod frame_pipeline;
mod video_pipeline;
use clap::Parser;

// handle command line arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, help = "Input file to process", required = true)]
    input: String,

    #[arg(short, long, help = "Output directory", required = true)]
    output: String,
}

fn main() {
    // parse command line arguments
    let args = Args::parse();

    // create video pipeline
    let pipeline = video_pipeline::VideoPipeline::new(&args.input).unwrap();

    // create frame pipeline
    let mut frame_pipeline = frame_pipeline::FramePipeline::new(&args.output).unwrap();

    // start pipeline
    pipeline.start().unwrap();

    // process frames
    let mut frame_count = 0;
    while let Some(frame) = pipeline.next_frame() {
        if frame_count % 100 == 0 {
            println!("Frame: {}, with channels: {}", frame_count, frame.channels);
            frame.print_pixel(10, 10);

            // process frame
            frame_pipeline.process_frame(&frame, frame_count).unwrap();
        }

        frame_count += 1;
    }
}

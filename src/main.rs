mod frame_pipeline;
mod pipeline_steps;
mod video_pipeline;
mod host;

use clap::Parser;
use host::ux_loop::launch_ux_loop;

// handle command line arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, help = "Input file to process", required_unless_present = "ui")]
    input: Option<String>,

    #[arg(short, long, help = "Output directory", required_unless_present = "ui")]
    output: Option<String>,

    #[arg(long, default_value_t = false, help = "Launch the application UI")]
    ui: bool,

    #[arg(
        long,
        default_value_t = false,
        help = "Enable development mode with hot reloading"
    )]
    dev: bool,
}

#[tokio::main]
async fn main() {
    // parse command line arguments
    let args = Args::parse();

    if args.ui {
        println!("Launching UI");
        launch_ux_loop(args.dev).await.unwrap();
    } else {
        println!("Running in CLI mode");
        let input = args.input.as_ref().unwrap();
        let output = args.output.as_ref().unwrap();

        // create video pipeline
        let pipeline = video_pipeline::VideoPipeline::new(&input).unwrap();

        // create frame pipeline
        let mut frame_pipeline = frame_pipeline::FramePipeline::new(&output).unwrap();

        // add canny edge detection step
        let edge_detection =
            pipeline_steps::canny_edge_detection::CannyEdgeDetection::new(&output)
            .unwrap();

        frame_pipeline.add_step(edge_detection);

        // start pipeline
        pipeline.start().unwrap();

        // process frames
        let mut frame_count = 0;
        while let Some(mut frame) = pipeline.next_frame() {
            if frame_count % 100 == 0 {
                println!("Frame: {}, with channels: {}", frame_count, frame.channels);
                frame.print_pixel(10, 10);

                // process frame
                frame_pipeline
                    .process_frame(&mut frame, frame_count)
                    .unwrap();
            }

            frame_count += 1;
        }

        //stop pipeline
        pipeline.stop().unwrap();
    }
}

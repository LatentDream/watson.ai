use objc_id::Id;
use crate::utils::filesys::local_data_dir_path;
use screencapturekit_sys::os_types::base::BOOL;
use screencapturekit_sys::{
    cm_sample_buffer_ref::CMSampleBufferRef, content_filter::UnsafeContentFilter,
    content_filter::UnsafeInitParams, shareable_content::UnsafeSCShareableContent,
    stream::UnsafeSCStream, stream_configuration::UnsafeStreamConfiguration,
    stream_error_handler::UnsafeSCStreamError, stream_output_handler::UnsafeSCStreamOutput,
};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::ops::Deref;
use std::path::PathBuf;

const MAX_CHANNELS: usize = 2;

struct StoreAudioHandler {}
struct ErrorHandler;

impl UnsafeSCStreamError for ErrorHandler {
    fn handle_error(&self) {
        log::error!("ERROR in SCStream");
    }
}

impl UnsafeSCStreamOutput for StoreAudioHandler {
    fn did_output_sample_buffer(&self, sample: Id<CMSampleBufferRef>, _of_type: u8) {
        let audio_buffers = sample.get_av_audio_buffer_list();

        let base_path = local_data_dir_path().join("tmp");

        for (i, buffer) in audio_buffers.into_iter().enumerate() {
            if i > MAX_CHANNELS {
                log::warn!("Audio recording with screen capture: more than two channels detected, only storing first two");
                break; // max two channels for now
            }
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .append(true) // Use append mode
                .open(base_path.join(PathBuf::from(format!("output{}.raw", i))))
                .expect("failed to open file");
            if let Err(e) = file.write_all(buffer.data.deref()) {
                log::error!("failed to write SCStream buffer to file: {:?}", e);
            }
        }
    }
}

pub fn init() -> Id<UnsafeSCStream> {
    // Don't record the screen
    let display = UnsafeSCShareableContent::get()
        .unwrap()
        .displays()
        .into_iter()
        .next()
        .unwrap();
    let width = display.get_width();
    let height = display.get_height();
    let filter = UnsafeContentFilter::init(UnsafeInitParams::Display(display));

    let config = UnsafeStreamConfiguration {
        width,
        height,
        captures_audio: BOOL::from(true),
        excludes_current_process_audio: BOOL::from(true),
        ..Default::default()
    };

    let stream = UnsafeSCStream::init(filter, config.into(), ErrorHandler);
    stream.add_stream_output(StoreAudioHandler {}, 1);
    stream
}

pub fn start_capture(stream: &Id<UnsafeSCStream>) {
    let base_path = local_data_dir_path()
        .join("tmp");
    for i in 0..MAX_CHANNELS {
        let output_file = PathBuf::from(format!("output{}.raw", i));
        let output_path = base_path.join(output_file);
        if output_path.exists() {
            fs::remove_file(output_path).unwrap();
        }
    }
    stream.start_capture().expect("Failed to start capture");
}

pub fn stop_capture(stream: &Id<UnsafeSCStream>) {
    stream.stop_capture().expect("Failed to stop capture");
}

pub fn convert_to_wav(output_path: &str) {
    // TODO: convert to wav
    // ffmpeg -f f32le -ar 48000 -ac 1 -i output0.raw -f f32le -ar 48000 -ac 1 -i output1.raw -filter_complex "[0:a][1:a]amerge=inputs=2" -ac 2 output.wav
    let base_path = local_data_dir_path().join("tmp");
    let output_0 = base_path.join(PathBuf::from(format!("output{}.raw", 0)));
    let output_1 = base_path.join(PathBuf::from(format!("output{}.raw", 1)));
    let ffmpegcommand = tauri::api::process::Command::new_sidecar("ffmpeg")
        .expect("failed to create `ffmpeg` binary command")
        .args([
            "-y",
            "-f",
            "f32le",
            "-ar",
            "48000",
            "-ac",
            "1",
            "-i",
            &output_0.to_string_lossy(),
            "-f",
            "f32le",
            "-ar",
            "48000",
            "-ac",
            "1",
            "-i",
            &output_1.to_string_lossy(),
            "-filter_complex",
            "[0:a][1:a]amerge=inputs=2",
            "-ac",
            "2",
            &output_path,
        ])
        .output()
        .expect("failed to execute process");
    log::info!("[FFMPG] status: {:?}", ffmpegcommand.status);
    log::info!("[FFMPG] stdout: {:?}", String::from(&ffmpegcommand.stdout));
    if !ffmpegcommand.status.success() {
        log::error!("[FFMPG] stderr: {:?}", String::from(&ffmpegcommand.stderr));
        panic!("FFMPEG failed to merge the raw audio files from screen capture kit");
    }
    log::info!("[FFMPG] COMPLETED - {}", output_path);
}

pub fn pause_capture(stream: &Id<UnsafeSCStream>) {
    stream.start_capture().expect("Failed to pause capture");
}

pub fn resume_capture(stream: &Id<UnsafeSCStream>) {
    stream.stop_capture().expect("Failed to resume capture");
}

extern crate cpal;
use log;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound::WavWriter;
use cpal::Device;
use std::fs::File;
use std::io::BufWriter;
use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc::channel;
use super::cpal_utils;
use tauri::Window;
use crate::utils::event::EventPayload;


pub fn record(output_path: &str, window: Window, device_name: Option<String>) -> Result<cpal_utils::RecordingCommunicationChannel, anyhow::Error> {

    // Create channel to communicate with the thread
    let (sender, receiver) = channel();

    // Get device and config
    let host: cpal::Host = cpal::default_host();
    let input_device = match device_name {
        Some(device_name) => {
            log::info!("[Microphone] Recording request on: {}", device_name);
            let device = host.input_devices()?.find(|device| device.name().unwrap_or("Unknown Device".to_string()) == device_name);
            if device.is_none() {
                log::error!("[Microphone] Device \"{}\" not found on client device", device_name);
                log::warn!(" [Microphone] Current avalaible devices: {}", host.input_devices()?.map(|device| device.name().unwrap_or("Unknown Device".to_string())).collect::<Vec<String>>().join(", "));
                panic!();
            } else {
                device.unwrap()
            }
        },
        None => {
            host.default_input_device().expect("[Microphone] No output devices available")
        }
    };
    let device_name = input_device.name().unwrap_or("Unknown Device".to_string());
    log::info!("[Microphone] Recording from input device: {}", device_name);
    let config = input_device.default_input_config().expect("Failed to get default config");
    log::info!("[Microphone] Default config: {:?}", config);
    let spec: hound::WavSpec = cpal_utils::wav_spec_from_config(&config);
    let writer: WavWriter<BufWriter<File>> = hound::WavWriter::create(output_path, spec)?;
    let writer = Arc::new(Mutex::new(Some(writer)));

    //? Begin recording 
    log::info!("[Microphone] Begin recording...");
    let writer_2 = writer.clone();
    let err_fn = move |err: cpal::StreamError| {
        let error_msg = err.to_string();
        log::error!("[Microphone] an error occurred on stream: {}", error_msg);
        window.emit("ERROR", EventPayload { message: format!("[Microphone Error]: {}", error_msg).into() }).unwrap();

    };
    let writer_clone = writer.clone();

    //? Spawn a new thread for recording
    let _recording_thread = thread::spawn(move || {
        let stream = match config.sample_format() {
            cpal::SampleFormat::I8 => input_device.build_input_stream(
                &config.into(),
                move |data, _: &_| cpal_utils::write_input_data::<i8, i8>(data, &writer_2),
                err_fn,
                None,
            ),
            cpal::SampleFormat::I16 => input_device.build_input_stream(
                &config.into(),
                move |data, _: &_| cpal_utils::write_input_data::<i16, i16>(data, &writer_2),
                err_fn,
                None,
            ),
            cpal::SampleFormat::I32 => input_device.build_input_stream(
                &config.into(),
                move |data, _: &_| cpal_utils::write_input_data::<i32, i32>(data, &writer_2),
                err_fn,
                None,
            ),
            cpal::SampleFormat::F32 => input_device.build_input_stream(
                &config.into(),
                move |data, _: &_| cpal_utils::write_input_data::<f32, f32>(data, &writer_2),
                err_fn,
                None,
            ),
            sample_format => {
                log::error!("[Microphone] Unsupported sample format '{:?}'", sample_format);
                return;
            }
        }
        .expect("Failed to create input stream");

        let _ = stream.play();

        // Process messages from the main thread
        while let Ok(message) = receiver.recv() {
            match message {
                cpal_utils::RecordingMessage::Pause => {
                    log::info!("[Microphone] Pause recording");
                    let _ = stream.pause();

                }
                cpal_utils::RecordingMessage::Resume => {
                    log::info!("[Microphone] Resume recording");
                    let _ = stream.play();
                }
                cpal_utils::RecordingMessage::Stop => {
                    log::info!("[Microphone] Stop recording");
                    break;
                }
            }
        }

        // std::thread::sleep(std::time::Duration::from_secs(10));
        drop(stream);
        writer_clone.lock().unwrap().take().unwrap().finalize().expect("Failed to finalize writer");
        log::info!("[Microphone] Recording complete!");
    });

    return Ok(cpal_utils::RecordingCommunicationChannel {
        sender,
        target_device: device_name
    });
}

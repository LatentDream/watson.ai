extern crate cpal;
use chrono::Utc;
use log;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound::WavWriter;
use tauri::Window;
use std::fs::File;
use std::io::BufWriter;
use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc::channel;
use crate::utils::event::EventPayload;
use super::cpal_utils;


pub fn record(outptu_path: &str, window: Window, device_name: Option<String>) -> Result<cpal_utils::RecordingCommunicationChannel, anyhow::Error> {

    // Notification setup
    let starting_time = Utc::now().time();
    let mut last_notification = Utc::now().time();

    // Create channel to communicate with the thread
    let (sender, receiver) = channel();

    // Get device and config
    let host: cpal::Host = cpal::default_host();
    let output_device = match device_name {
        Some(device_name) => {
            log::info!("[Speaker] Recording request on: {}", device_name);
            let device = host.output_devices()?.find(|device| device.name().unwrap_or("Unknown Device".to_string()) == device_name);
            if device.is_none() {
                log::error!("[Speaker] Device \"{}\" not found on client device", device_name);
                log::warn!(" [Speaker] Current avalaible devices: {}", host.output_devices()?.map(|device| device.name().unwrap_or("Unknown Device".to_string())).collect::<Vec<String>>().join(", "));
                panic!();
            } else {
                device.unwrap()
            }
        },
        None => {
            host.default_output_device().expect("[Speaker] No output devices available")
        }
    };

    // TODO: Fix this for MacOS - Custom Driver ?
    #[cfg(target_family = "unix")] // for Apple products
    {
        let current_output_device = host.default_output_device().expect("Failed to get default input device").name().unwrap_or("Unknown Device".to_string());
        if !current_output_device.contains("Watson") {
            window.emit("ERROR", EventPayload { message: format!("MacOS - Recording with BlackHole multi-output failed. Ensure that the BlackHole audio device is selected and this one is renamed `Watson`").into() }).unwrap();
        }
    }

    let device_name = output_device.name().unwrap_or("Unknown Device".to_string());
    log::info!("[Speaker] Recording from output device: {}", device_name);
    let config = output_device.default_output_config().expect("Failed to get default config");
    log::info!("[Speaker] Default config: {:?}", config);
    let spec: hound::WavSpec = cpal_utils::wav_spec_from_config(&config);
    let writer: WavWriter<BufWriter<File>> = hound::WavWriter::create(outptu_path, spec)?;
    let writer = Arc::new(Mutex::new(Some(writer)));

    //? Begin recording 
    log::info!("[Speaker] Begin recording...");
    let writer_2 = writer.clone();
    let err_fn = move |err: cpal::StreamError| {
        let error_msg = err.to_string();
        log::error!("[Speaker] an error occurred on stream: {}", error_msg);
        window.emit("ERROR", EventPayload { message: format!("[Speaker Error]: {}", error_msg).into() }).unwrap();
    };
    let writer_clone = writer.clone();

    //? Spawn a new thread for recording
    let _recording_thread = thread::spawn(move || {
        let stream = match config.sample_format() {
            cpal::SampleFormat::I8 => output_device.build_input_stream(
                &config.into(),
                move |data, _: &_| cpal_utils::write_input_data_and_notification::<i8, i8>(data, &writer_2, starting_time, &mut last_notification),
                err_fn,
                None,
            ),
            cpal::SampleFormat::I16 => output_device.build_input_stream(
                &config.into(),
                move |data, _: &_| cpal_utils::write_input_data_and_notification::<i16, i16>(data, &writer_2, starting_time, &mut last_notification),
                err_fn,
                None,
            ),
            cpal::SampleFormat::I32 => output_device.build_input_stream(
                &config.into(),
                move |data, _: &_| cpal_utils::write_input_data_and_notification::<i32, i32>(data, &writer_2, starting_time, &mut last_notification),
                err_fn,
                None,
            ),
            cpal::SampleFormat::F32 => output_device.build_input_stream(
                &config.into(),
                move |data, _: &_| cpal_utils::write_input_data_and_notification::<f32, f32>(data, &writer_2, starting_time, &mut last_notification),
                err_fn,
                None,
            ),
            sample_format => {
                log::error!("[Speaker] Unsupported sample format '{:?}'", sample_format);
                return;
            }
        }
        .expect("Failed to create input stream");

        let _ = stream.play();

        // Process messages from the main thread
        while let Ok(message) = receiver.recv() {
            match message {
                cpal_utils::RecordingMessage::Pause => {
                    log::info!("[Speaker] Pause recording");
                    let _ = stream.pause();

                }
                cpal_utils::RecordingMessage::Resume => {
                    log::info!("[Speaker] Resume recording");
                    let _ = stream.play();
                }
                cpal_utils::RecordingMessage::Stop => {
                    log::info!("[Speaker] Stop recording");
                    break;
                }
            }           
        }

        // std::thread::sleep(std::time::Duration::from_secs(10));
        drop(stream);
        writer_clone.lock().unwrap().take().unwrap().finalize().expect("Failed to finalize writer");
        log::info!("[Speaker] Recording complete!");
    });

    return Ok(cpal_utils::RecordingCommunicationChannel {
        sender,
        target_device: device_name
    });

}



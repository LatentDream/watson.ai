extern crate cpal;
use crate::utils::event::EventPayload;
use chrono::Utc;
use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{FromSample, Sample};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufWriter;
use std::sync::{Arc, Mutex};
use tauri::api::notification::Notification;
use tauri::window;
use tauri::Window;
use ts_rs::TS;

pub enum RecordingMessage {
    Pause,
    Resume,
    Stop,
}

pub struct RecordingCommunicationChannel {
    pub sender: std::sync::mpsc::Sender<RecordingMessage>,
    pub target_device: String,
}

fn sample_format(format: cpal::SampleFormat) -> hound::SampleFormat {
    if format.is_float() {
        hound::SampleFormat::Float
    } else {
        hound::SampleFormat::Int
    }
}

pub fn wav_spec_from_config(config: &cpal::SupportedStreamConfig) -> hound::WavSpec {
    hound::WavSpec {
        channels: config.channels() as _,
        sample_rate: config.sample_rate().0 as _,
        bits_per_sample: (config.sample_format().sample_size() * 8) as _,
        sample_format: sample_format(config.sample_format()),
    }
}

pub type WavWriterHandle = Arc<Mutex<Option<hound::WavWriter<BufWriter<File>>>>>;

pub fn write_input_data<T, U>(input: &[T], writer: &WavWriterHandle)
where
    T: Sample,
    U: Sample + hound::Sample + FromSample<T>,
{
    if let Ok(mut guard) = writer.try_lock() {
        if let Some(writer) = guard.as_mut() {
            for &sample in input.iter() {
                let sample: U = U::from_sample(sample);
                writer.write_sample(sample).ok();
            }
        }
    }
}

pub fn write_input_data_and_notification<T, U>(
    data: &[T],
    writer_2: &WavWriterHandle,
    starting_time: chrono::NaiveTime,
    last_notif: &mut chrono::NaiveTime,
) where
    T: Sample,
    U: Sample + hound::Sample + FromSample<T>,
{
    let now = Utc::now().time();
    if (now - *last_notif).num_minutes() >= 15 {
        let diff = (now - starting_time).num_minutes();
        #[cfg(target_family = "windows")]
        show_recording_status_notification(diff as u32);
        *last_notif = now;
    }
    write_input_data::<T, U>(data, &writer_2)
}

#[cfg(target_family = "windows")]
fn show_recording_status_notification(min_since_started: u32) {
    log::info!("Show notification");
    let context: tauri::Context<tauri::utils::assets::EmbeddedAssets> = tauri::generate_context!();
    let result = Notification::new(&context.config().tauri.bundle.identifier)
        .title("Recording in Progress")
        .body(format!(
            "It's been {} minutes since the begenning of the recording",
            min_since_started
        ))
        .show();
    match result {
        Ok(_) => {}
        Err(e) => {
            log::error!("Error while showing notification: {:?}", e);
        }
    }
}

#[derive(Clone, Deserialize, Serialize, TS)]
#[ts(export, export_to = "../src/bindings/")]
pub struct RecordingDevices {
    pub input_device_name: String,
    pub output_device_name: String,
}

#[derive(Clone, Deserialize, Serialize, TS)]
#[ts(export, export_to = "../src/bindings/")]
pub struct AudioDevice {
    pub name: String,
    pub is_default: bool,
}

#[derive(Clone, Deserialize, Serialize, TS)]
#[ts(export, export_to = "../src/bindings/")]
pub struct AvailableDevices {
    pub input_devices: Vec<AudioDevice>,
    pub output_devices: Vec<AudioDevice>,
}

#[allow(unreachable_code)]
pub fn get_available_audio_devices(window: Window) -> Result<AvailableDevices, anyhow::Error> {
    let host = cpal::default_host();

    // TODO: Change to cpal with integration cpal <> screen capture kit is done
    #[cfg(target_os = "macos")]
    {
        let input_devices = host.input_devices().expect("Failed to get audio devices");
        let default_input_device = host
            .default_input_device()
            .expect("Failed to get default input device")
            .name()
            .unwrap_or("Unknown Device".to_string());

        // TODO: check if permission for screen capture recording is granted
        let default_output_device = "System Audio".to_string();
        let output_device = AudioDevice { name: default_output_device, is_default: true };

        let devices = AvailableDevices {
            input_devices: input_devices
                .map(|d| AudioDevice {
                    name: d.name().unwrap_or("Unknown Device".to_string()),
                    is_default: d.name().unwrap_or("Unknown Device".to_string())
                        == default_input_device,
                })
                .collect(),
            output_devices: vec!(output_device),
        };

        return Ok(devices);
    }

    let input_devices = host.input_devices().expect("Failed to get audio devices");
    let default_input_device = host
        .default_input_device()
        .expect("Failed to get default input device")
        .name()
        .unwrap_or("Unknown Device".to_string());
    let output_devices = host.output_devices().expect("Failed to get audio devices");
    let default_output_device = host
        .default_output_device()
        .expect("Failed to get default output device")
        .name()
        .unwrap_or("Unknown Device".to_string());
    let devices = AvailableDevices {
        input_devices: input_devices
            .map(|d| AudioDevice {
                name: d.name().unwrap_or("Unknown Device".to_string()),
                is_default: d.name().unwrap_or("Unknown Device".to_string())
                    == default_input_device,
            })
            .collect(),
        output_devices: output_devices
            .map(|d| AudioDevice {
                name: d.name().unwrap_or("Unknown Device".to_string()),
                is_default: d.name().unwrap_or("Unknown Device".to_string())
                    == default_output_device,
            })
            .collect(),
    };
    return Ok(devices);
}

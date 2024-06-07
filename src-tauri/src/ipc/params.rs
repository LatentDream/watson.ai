
//* Params types used in the IPC methods.
//*
//* The current best practice is to follow a single argument type, called "params" for all method (JSON-RPC's style).
//*

use serde::Deserialize;

use crate::audio::cpal_audio::cpal_utils;

#[derive(Deserialize)]
pub struct CreateParams<D> {
	pub data: D,
}

#[derive(Deserialize)]
pub struct UpdateParams<D> {
	pub id: String,
	pub data: D,
}

#[derive(Deserialize)]
pub struct GetParams {
	pub id: String,
}

#[derive(Deserialize)]
pub struct GetTranscriptParams {
	pub path: String,
	pub language: String
}

#[derive(Deserialize)]
pub struct DeleteParams {
	pub id: String,
}

#[derive(Deserialize)]
pub struct QueryParams {
	pub query: String,
}

#[derive(Deserialize)]
pub struct GetRecordingStartParams {
	pub recording_devices: cpal_utils::RecordingDevices,
}

//* `ipc` module and sub-modules are all Rust constructs necessary for the WebView to Rust Tauri IPC calls.
//*
//* At a high level it follows the "JSON-RPC 2.0" format
//*   - method_name - Will be the Tauri command function name)
//*   - params - Tauri commands will have one params argument by designed, called params (and state arguments)
//*   - response - Will be a IpcResponse with the JSON-RPC 2.0 result/error format back.
//*
//* The benefits of following the JSON-RPC 2.0 style is that it is simple, clean, and allows to wire the frontend to a
//* JSON-RPC 2.0 cloud backend easily.
//*
//* Notes:
//*   - This module re-exports the appropriate sub-module constructs as their hierarchy is irrelevant to callers.

mod params;
mod meeting;
mod response;
mod recording;
mod setting;
mod crm;
mod session;

// --- re-exports
pub use params::*;
pub use meeting::*;
pub use response::*;
pub use recording::*;
pub use setting::*;
pub use crm::*;
pub use session::*;

use ts_rs::TS;
use serde::Serialize;


/// For now, all mutation queries will return an {id} struct.
/// Note: Keep it light, and client can do a get if needed.
#[derive(TS, Serialize, Clone)]
#[ts(export, export_to = "../src/bindings/")]
pub struct ModelMutateResultData {
	pub id: String,
}

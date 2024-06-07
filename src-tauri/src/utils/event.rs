
// For sending events to the frontend
#[derive(Clone, serde::Serialize)]
pub struct EventPayload {
  pub message: String,
}

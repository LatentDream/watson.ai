
use serde::Serialize;
use std::fmt::Debug;

#[derive(Serialize)]
struct IpcError {
	message: String,
}

#[derive(Serialize)]
pub struct IpcSimpleResult<D> where	D: Serialize {
	pub data: D
}

#[derive(Serialize)]
pub struct IpcResponse<D> where	D: Serialize, {
	error: Option<IpcError>,
	result: Option<IpcSimpleResult<D>>,
}

impl<D, E> From<Result<D, E>> for IpcResponse<D> where D: Serialize, E: Debug {
	fn from(res: Result<D, E>) -> Self {
		match res {
			Ok(data) => IpcResponse {
				error: None,
				result: Some(IpcSimpleResult { data }),
			},
			Err(err) => IpcResponse {
				error: Some(IpcError {
					message: format!("{:?}", err)
				}),
				result: None,
			},
		}
	}
}

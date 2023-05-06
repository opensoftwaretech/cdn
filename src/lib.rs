use std::{ffi::OsStr, path::Path};

use serde::Serialize;

#[derive(Serialize)]
pub struct Response<T> {
	pub status: u16,
	pub message: String,
	pub data: Option<T>,
}

pub fn get_extension_from_filename(filename: &str) -> Option<&str> {
	Path::new(filename).extension().and_then(OsStr::to_str)
}

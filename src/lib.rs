use redis_macros::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};
use std::{ffi::OsStr, path::Path};

#[derive(Serialize, Default)]
pub struct Response<'a, T> {
	pub status: u16,
	pub message: &'a str,
	pub data: Option<T>,
}

pub const EXPIRE_TIME: usize = 60 * 10; // 10 minutes

pub fn get_extension_from_filename(filename: &str) -> Option<&str> {
	Path::new(filename).extension().and_then(OsStr::to_str)
}

#[derive(Serialize, Deserialize, FromRedisValue, ToRedisArgs, Clone)]
pub struct CachedFile {
	pub path: String,
	pub content_type: String,
}

impl CachedFile {
	pub fn new(path: String, content_type: String) -> Self {
		Self { path, content_type }
	}
}

pub async fn get_redis_conn(redis: redis::Client) -> redis::aio::Connection {
	match redis.get_async_connection().await {
		Ok(conn) => conn,
		Err(err) => {
			log::error!("Error connecting to redis: {}", err);
			std::process::exit(1);
		}
	}
}

pub fn get_mime_type(path: String) -> String {
	let kind = infer::get_from_path(path)
		.expect("file read successfully")
		.expect("file type is known");

	return kind.mime_type().to_string();
}

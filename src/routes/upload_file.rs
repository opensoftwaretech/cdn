use actix_multipart::form::{tempfile::TempFile, MultipartForm};
use actix_web::{Error, HttpResponse, Responder};
use cdn::{get_extension_from_filename, Response};
use serde::Serialize;
use snowflake::SnowflakeIdGenerator;

#[derive(Debug, MultipartForm)]
pub struct UploadForm {
	#[multipart(rename = "file")]
	files: Vec<TempFile>,
}

#[derive(Debug, Serialize)]
pub struct FileData {
	id: i64,
	ext: String,
}

pub async fn route(
	MultipartForm(form): MultipartForm<UploadForm>,
) -> Result<impl Responder, Error> {
	let mut id_generator = SnowflakeIdGenerator::new(1, 1);
	let mut files: Vec<FileData> = Vec::new();

	for f in form.files {
		let file_name = f.file_name.clone().unwrap();
		let file_ext = get_extension_from_filename(&file_name.as_str()).unwrap();
		let file_id = id_generator.real_time_generate();

		let upload_dir = std::env::var("UPLOADS_DIR").unwrap_or_else(|_| "/tmp/uploads".to_string());

		let path = format!("{}/{}.{}", upload_dir, file_id, file_ext);

		f.file.persist(&path).unwrap();
		files.push(FileData {
			id: file_id,
			ext: file_ext.to_string(),
		});
	}

	Ok(HttpResponse::Ok().json(Response::<Vec<FileData>> {
		status: 200,
		message: "Upload success",
		data: Some(files),
	}))
}

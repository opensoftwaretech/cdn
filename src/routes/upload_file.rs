use actix_multipart::form::{tempfile::TempFile, MultipartForm};
use actix_web::{web, Error, HttpResponse, Responder};
use cdn::{get_extension_from_filename, Response};
use serde::Serialize;
use snowflake::SnowflakeIdGenerator;

use crate::prisma;

#[derive(Debug, MultipartForm)]
pub struct UploadForm {
	// TODO: create a custom midleware to personalize the file limit
	#[multipart(rename = "file", limit = "10 MiB")]
	files: Vec<TempFile>,
}

#[derive(Debug, Serialize)]
pub struct FileData {
	id: i64,
	ext: String,
	path: String,
	size: i32,
}

pub async fn route(
	MultipartForm(form): MultipartForm<UploadForm>,
	client: web::Data<prisma::PrismaClient>,
) -> Result<impl Responder, Error> {
	let mut id_generator = SnowflakeIdGenerator::new(1, 1);
	let mut files: Vec<FileData> = Vec::new();

	for f in form.files {
		let file_name = f.file_name.clone().unwrap();
		let file_size = f.file.as_file().metadata().unwrap().len();
		let file_ext = get_extension_from_filename(&file_name.as_str()).unwrap();
		let file_id = id_generator.real_time_generate();

		let upload_dir =
			std::env::var("UPLOADS_DIR").unwrap_or_else(|_| "/tmp/uploads".to_string());
		let temp_file_path = f.file.path();
		let path = format!("{}/{}.{}", upload_dir, file_id, file_ext);

		log::debug!("Moving file from {} to {}", path, temp_file_path.display());
		match std::fs::rename(temp_file_path, &path) {
			Ok(_) => {
				files.push(FileData {
					id: file_id,
					ext: file_ext.to_string(),
					path,
					size: file_size as i32,
				});
			}
			Err(_) => {
				HttpResponse::InternalServerError().finish();
			}
		}
	}

	let _ = client
		.file()
		.create_many(
			files
				.iter()
				.map(|file| {
					let kind = infer::get_from_path(file.path.clone())
						.expect("file read successfully")
						.expect("file type is known");

					prisma::file::create_unchecked(
						file.id,
						file.path.clone(),
						file.size,
						kind.mime_type().to_string(),
						vec![],
					)
				})
				.collect(),
		)
		.exec()
		.await;

	Ok(HttpResponse::Ok().json(Response::<Vec<FileData>> {
		status: 200,
		message: "Upload success",
		data: Some(files),
	}))
}

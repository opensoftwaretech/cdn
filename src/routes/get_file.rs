use actix_web::{web, HttpResponse};

use crate::prisma;

pub async fn route(client: web::Data<prisma::PrismaClient>, id: web::Path<String>) -> HttpResponse {
	let (id, _): (i64, String) = match id.into_inner().split_once('.') {
		Some((id, ext)) => (id.parse::<i64>().unwrap(), ext.to_string()),
		None => return HttpResponse::BadRequest().body("Invalid file id"),
	};

	let file = match client
		.file()
		.find_unique(prisma::file::UniqueWhereParam::IdEquals(id))
		.exec()
		.await
	{
		Ok(file) => file.unwrap(),
		Err(_) => return HttpResponse::NotFound().body("File not found"),
	};

	let file_content = match web::block(|| std::fs::read(file.path)).await {
		Ok(file_content) => file_content.unwrap(),
		Err(_) => return HttpResponse::InternalServerError().body("Failed to read file"),
	};

	return HttpResponse::Ok()
		.content_type(file.content_type)
		.body(file_content);
}

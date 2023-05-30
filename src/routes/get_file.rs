use crate::prisma;
use actix_web::{web, Error, HttpResponse, Responder};
use cdn::{get_redis_conn, CachedFile, EXPIRE_TIME};
use redis::AsyncCommands;

pub async fn route(
	client: web::Data<prisma::PrismaClient>,
	redis: web::Data<redis::Client>,
	id: web::Path<String>,
) -> Result<impl Responder, Error> {
	let (id, _): (i64, String) = match id.into_inner().split_once('.') {
		Some((id, ext)) => (id.parse::<i64>().unwrap(), ext.to_string()),
		None => return Ok(HttpResponse::BadRequest().body("Invalid file id")),
	};

	let mut conn = get_redis_conn(redis.get_ref().clone()).await;

	let file: CachedFile = match conn.get::<String, CachedFile>(id.to_string()).await {
		Ok(file) => file,
		Err(_) => {
			let db_file = match client
				.file()
				.find_unique(prisma::file::UniqueWhereParam::IdEquals(id))
				.exec()
				.await
			{
				Ok(file) => file.unwrap(),
				Err(_) => return Ok(HttpResponse::NotFound().body("File not found")),
			};

			let cached_file = CachedFile::new(db_file.path, db_file.content_type);
			conn.set_ex::<String, CachedFile, String>(
				id.to_string(),
				cached_file.clone(),
				EXPIRE_TIME,
			)
			.await
			.unwrap();

			cached_file
		}
	};

	let file_content = match web::block(|| std::fs::read(file.path)).await {
		Ok(file_content) => file_content.unwrap(),
		Err(_) => return Ok(HttpResponse::InternalServerError().body("Failed to read file")),
	};

	return Ok(HttpResponse::Ok()
		.content_type(file.content_type)
		.body(file_content));
}

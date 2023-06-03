use actix_cors::Cors;
use actix_multipart::form::tempfile::TempFileConfig;
use actix_web::{
	get,
	http::{header, StatusCode},
	middleware, web, App, HttpResponse, HttpServer, Responder,
};

mod middlewares;
#[cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
#[rustfmt::skip]
#[allow(clippy::all)]
mod prisma;
mod routes;
use cdn::Response;

#[get("/")]
async fn main_route() -> impl Responder {
	HttpResponse::Ok().json(Response::<()> {
		message: "Hello, world!",
		status: StatusCode::OK.as_u16(),
		..Default::default()
	})
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	dotenv::dotenv().ok();
	env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

	let uploads_dir = std::env::var("UPLOADS_DIR").unwrap_or_else(|_| "/tmp/uploads".to_string());
	let port = std::env::var("API_PORT")
		.unwrap_or_else(|_| "8080".to_string())
		.parse()
		.expect("API_PORT must be a number");
	let redis_url =
		std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

	let db_client = match prisma::PrismaClient::_builder().build().await {
		Ok(client) => web::Data::new(client),
		Err(e) => {
			log::error!("Failed to connect to database: {}", e);
			std::process::exit(1);
		}
	};

	let redis = match redis::Client::open(redis_url) {
		Ok(client) => web::Data::new(client),
		Err(e) => {
			log::error!("Failed to connect to redis: {}", e);
			std::process::exit(1);
		}
	};

	log::info!("Uploads dir: {:?}", uploads_dir);

	let server = HttpServer::new(move || {
		App::new()
			.app_data(db_client.clone())
			.app_data(redis.clone())
			.app_data(TempFileConfig::default().directory(&uploads_dir))
			.wrap(
				Cors::default()
					.allow_any_origin()
					.allowed_methods(vec!["POST", "GET", "DELETE"])
					.allowed_headers(vec![
						header::AUTHORIZATION,
						header::ACCEPT,
						header::CONTENT_TYPE,
					])
					.allowed_header(header::CONTENT_TYPE)
					.supports_credentials()
					.max_age(3600),
			)
			.wrap(middleware::Compress::default())
			.wrap(middleware::Logger::default())
			.service(main_route)
			.configure(routes::init_routes)
	});

	log::info!("Starting server at http://localhost:{:?}", port);

	server.bind(("0.0.0.0", port))?.run().await
}

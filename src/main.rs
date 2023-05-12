use actix_cors::Cors;
use actix_files::Files;
use actix_multipart::form::tempfile::TempFileConfig;
use actix_web::{get, http::header, middleware, web, App, HttpServer, Responder};

mod routes;
use routes::*;

#[get("/")]
async fn main_route() -> impl Responder {
	format!("Hello World!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

	let port = std::env::var("API_PORT")
		.unwrap_or_else(|_| "8080".to_string())
		.parse()
		.expect("API_PORT must be a number");

	let server = HttpServer::new(move || {
		App::new()
			.app_data(TempFileConfig::default().directory("/uploads"))
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
			.service(web::scope("/file").route("/upload", web::post().to(upload_file::route)))
			.service(Files::new("/f", "/uploads/").show_files_listing())
	});

	log::info!("Starting server at http://localhost:{:?}", port);

	server.bind(("0.0.0.0", port))?.run().await
}

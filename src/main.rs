use actix_cors::Cors;
use actix_web::{get, http::header, middleware, App, HttpServer, Responder};

#[get("/")]
async fn main_route() -> impl Responder {
	format!("Hello World!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	dotenv::dotenv().ok();

	let port = std::env::var("API_PORT")
		.unwrap_or_else(|_| "8080".to_string())
		.parse()
		.expect("API_PORT must be a number");

	let server = HttpServer::new(move || {
		App::new()
			.wrap(
				Cors::default()
					.allow_any_origin()
					.allowed_methods(vec!["POST", "GET", "DELETE"])
					.allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
					.allowed_header(header::CONTENT_TYPE)
					.supports_credentials()
					.max_age(3600),
			)
			.wrap(middleware::Compress::default())
			.wrap(middleware::Logger::default())
			.service(main_route)
	});

	println!("Starting server at http://localhost:{:?}", port);

	server.bind(("127.0.0.1", port))?.run().await
}

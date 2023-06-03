use actix_web::web;

pub mod get_file;
pub mod upload_file;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
	cfg.service(upload_file::route);
	cfg.service(get_file::route);
}

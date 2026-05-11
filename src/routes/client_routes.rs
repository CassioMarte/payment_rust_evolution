use actix_web::web;
use crate::handler::client_handler;

pub fn config(cfg: &mut web::ServiceConfig) {
  cfg.service(web::resource("/clients"))
     .route(web::post().to(client_handler::create_client))
     .route(web::get().to(client_handler::find_all_clients))
     .service(web::resource("/clients/{id"))
     .route(web::get().to(client_handler::find_client_by_id))
     .route(web::put().to(client_handler::update_client))
     .route(web::delete().to(client_handler::delete_client))
}
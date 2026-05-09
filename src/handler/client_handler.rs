use actix_web::{web, HttpResponse, Responder};
use uuid::Uuid;
use std::sync::Arc;

use crate::errors::ApiError;
use crate::models::client::{CreateClientDto, UpdateClientDto};
use crate::services::client_service::ClientService;

pub async fn create_client(
    client_service: web::Data<Arc<ClientService>>,
    new_client_dto: web::Json<CreateClientDto>,
) -> Result<HttpResponse, ApiError> {
    let client = client_service.create(new_client_dto.into_inner()).await?;
    Ok(HttpResponse::Created().json(client))
}

pub async fn find_all_clients(
   client_service: web::Data<Arc<ClientService>>,
) -> Result<HttpResponse, ApiError> {
    let clients = client_service.find_all().await?;
    Ok(HttpResponse::Ok().json(clients))
}

pub async fn find_client_by_id(
   client_service: web::Data<Arc<ClientService>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let client_id = path.into_inner();
    let client = client_service.find_by_id(client_id).await?;
    Ok(HttpResponse::Ok().json(client))
}

pub async fn update_client(
    client_service: web::Data<Arc<ClientService>>,
    path: web::Path<Uuid>,
    updated_client: web::Json<UpdateClientDto>,
) -> Result<HttpResponse, ApiError> {
    let client_id = path.into_inner();

    let client = client_service.update_client(
        client_id, 
        updated_client.into_inner()
      ).await?;

    Ok(HttpResponse::Ok().json(client))
}

pub async fn delete_client(
    client_service: web::Data<Arc<ClientService>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let client_id = path.into_inner();
    
    client_service.delete_client(client_id).await?;

    Ok(HttpResponse::NoContent().finish())
}
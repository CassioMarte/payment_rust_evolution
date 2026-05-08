# Handler

````
// web           -> extratores (Json, Path, Data)
// HttpResponse  -> construtor de respostas HTTP
// Responder     -> trait para tipos que podem ser resposta
use actix_web::{web, HttpResponse, Responder};
use uuid::Uuid;
// Arc -> Para acessar o serviço compartilhado com segurança
use std::sync::Arc; 
use crate::errors::ApiError;
use crate::models::client::{CreateClientDto, UpdateClientDto};
use crate::services::client_service::ClientService;

pub async fn create_client_handler(
    // web::Data extrai o serviço que injetamos no main.rs
    // web::Data<Arc<ClientService>>
    // web::Data -> injetado pelo Actix automaticamente (vem do app_data),
    // Arc       -> compartilhado entre threads com segurança
    // ClientService -> o service que contém as regras de negócio
    client_service: web::Data<Arc<ClientService>>,

    // web::Json<CreateClientDto>
    // web::Json -> desserializa o corpo da requisição em CreateClientDto
    // se o JSON for inválido -> Actix retorna 400 antes de entrar no handler
    new_client_dto: web::Json<CreateClientDto>,

) -> Result<HttpResponse, ApiError> {

    let client = client_service
        .create_client(
            new_client_dto.into_inner() // extrai CreateClientDto de dentro do web::Json
        )
        .await?; // ? propaga ApiError para o Actix se falhar
                 // Actix chama ApiError::error_response() automaticamente

    // 201 Created — recurso criado com sucesso
    // .json(client) -> serializa Client para JSON no corpo da resposta
    Ok(HttpResponse::Created().json(client))
}


pub async fn get_all_clients_handler(
    client_service: web::Data<Arc<ClientService>>,
) -> Result<HttpResponse, ApiError> {

    let clients = client_service.get_all_clients().await?;

    // 200 Ok com lista de clients em JSON
    // lista vazia [] também é 200 — não é erro
    Ok(HttpResponse::Ok().json(clients))
}


pub async fn get_client_by_id_handler(
    client_service: web::Data<Arc<ClientService>>,

    // web::Path<Uuid> → extrai o UUID da URL
    // ex: GET /clients/550e8400-e29b-41d4-a716-446655440000
    //                  ↑ esse valor vira Uuid automaticamente
    path: web::Path<Uuid>,

) -> Result<HttpResponse, ApiError> {

    // .into_inner() → extrai o Uuid de dentro do web::Path
    let client_id = path.into_inner();

    let client = client_service.get_client_by_id(client_id).await?;
    // ? aqui pode propagar:
    // ApiError::NotFound    → 404 (cliente não existe)
    // ApiError::DatabaseError → 500 (banco falhou)

    Ok(HttpResponse::Ok().json(client))
}

pub async fn update_client_handler(
    client_service: web::Data<Arc<ClientService>>,

    // UUID vem da URL → /clients/{uuid}
    path: web::Path<Uuid>,

    // Dados atualizados vêm do corpo da requisição
    // UpdateClientDto tem campos Option → cliente envia só o que quer mudar
    updated_client_dto: web::Json<UpdateClientDto>,

) -> Result<HttpResponse, ApiError> {

    let client_id = path.into_inner();

    let client = client_service
        .update_client(
            client_id,
            updated_client_dto.into_inner() // extrai UpdateClientDto do web::Json
        )
        .await?;
    // ? pode propagar:
    // ApiError::NotFound → 404 (cliente não existe)
    // ApiError::Conflict → 409 (email duplicado)
    // ApiError::DatabaseError → 500

    Ok(HttpResponse::Ok().json(client))
}


// ═══════════════════════════════════════════════
// DELETE
// ═══════════════════════════════════════════════
pub async fn delete_client_handler(
    client_service: web::Data<Arc<ClientService>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {

    let client_id = path.into_inner();

    // delete_client retorna Result<(), ApiError>
    // () = vazio — delete não precisa retornar dados
    client_service.delete_client(client_id).await?;
    // ? pode propagar:
    // ApiError::NotFound → 404 (cliente não existe)
    // ApiError::DatabaseError → 500

    // 204 No Content — deletou com sucesso
    // .finish() → sem corpo na resposta (correto para DELETE)
    Ok(HttpResponse::NoContent().finish())
}
````


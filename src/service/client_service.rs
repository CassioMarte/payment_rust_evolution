use std::sync::Arc;
use uuid::Uuid;

use crate::errors::ApiError;
use crate::models::client::{Client, CreateClientDto, UpdateClientDto};
use crate::traits::client_repository::ClientRepository;

pub struct ClientService{
  repository: Arc<dyn ClientRepository>
}

impl ClientService{
  pub fn new(repository: Arc<dyn ClientRepository)-> Self{
    ClientService { repository }
  }

  pub async fn create(&self, new_client:CreateClientDto) -> Result<Client, ApiError> {
    // OBS: a validação de entrada ja ocorre no CreateClientDto via #[validate]
    self.repository.create(new_client).await?
  }

  pub async fn find_all(&self) -> Result<Vec<Client>, ApiError> {
        self.repository.find_all().await
  }

  pub async fn find_by_id(&self, id:Uuid) -> Result<Option<Client>, ApiError>{
    self.repository.find_by_id(id).await?
        .ok_or_else(|| ApiError::NotFound(
          format!("Client with ID {} ​​not found", id)
        ))
  }

  pub async fn update_client(&self, id: Uuid, updated_client: UpdateClientDto) -> Result<Client, ApiError> {
        // A validação de entrada já ocorre no UpdateClientDto via #[validate]
        self.repository.update(id, updated_client).await?
            .ok_or_else(|| ApiError::NotFound(format!("Client with ID {} ​​not found for update", id)))
    }

   pub async fn delete_client(&self, id: Uuid) -> Result<(), ApiError> {
        let deleted = self.repository.delete(id).await?;
        if deleted {
            Ok(())
        } else {
            Err(ApiError::NotFound(format!("Client with ID {} ​​not found for deletion", id)))
        }
    }
}


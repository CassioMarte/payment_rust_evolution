use async_trait::async_trait;
use uuid::Uuid;

use crate::model::client_model::{Client, CreateClientDto, UpdateClientDto};
use crate::errors::ApiError;

#[async_trait]
pub trait ClientRepository: Send + Sync{
  async fn create(&self, new_client: CreateClientDto) -> Result<Client, ApiError>;
  async fn find_all(&self) -> Result<Vec<Client>, ApiError>;
  async fn find_by_id(&self, id: Uuid) -> Result<Client, ApiError>;
  async fn update(&self, id: Uuid, updated_client: UpdateClientDto) -> Result<Client, ApiError>;
  async fn delete(&self, id: Uuid) -> Result<bool, ApiError>;
}
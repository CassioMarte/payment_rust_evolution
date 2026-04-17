use async_trait::async_trait;
use sqlx::{query, query_as, PgPool, Postgres};
use uuid::Uuid;
use chrono::Utc;

use crate::errors::ApiError;
use crate::models::client::{Client, CreateClientDto, UpdateClientDto, ClientName, ClientEmail, ClientAddress, PlanType};
use crate::traits::client_repository::ClientRepository;

pub struct SqlxClientRepository {
    pool: PgPool,
}

impl SqlxClientRepository {
  pub fn new(pool: PgPool) -> Self {
        SqlxClientRepository { pool }
    }
}

impl ClientRepository for SqlxClientRepository{
    async fn create(&self, new_client: CreateClientDto) -> Result<Client, ApiError> {
        let client = query_as::<Postgres, Client>(
            "INSERT INTO clients (name, email, address, plan, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *"
        )
        .bind(new_client.name.0)
        .bind(new_client.email.0)
        .bind(new_client.address.0)
        .bind(new_client.plan)
        .bind(Utc::now().naive_utc())
        .bind(Utc::now().naive_utc())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.is_unique_violation() {
                    return ApiError::Conflict(format!("Email já cadastrado: {}", new_client.email.0));
                }
            }
            ApiError::DatabaseError(format!("Falha ao criar cliente: {}", e))
        })?;

        Ok(client)
    }

    async fn find_all(&self) -> Result<Vec<Client>, ApiError>{
       let clients = query_as::<Postgres, Client>(
        "SELECT * FROM clients"
       )
       .fetch_all(&self.pool)
       .await
       .map_err(|e| ApiError::DatabaseError(format!("Failed to fetch clients: {}", e)))?;

       Ok(clients)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Client>, ApiError> {
        let client = query_as::<Postgres, Client>("SELECT * FROM clients WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| ApiError::DatabaseError(format!("Failed to fetch client: {}", e)))?;

        Ok(client)
    }


}
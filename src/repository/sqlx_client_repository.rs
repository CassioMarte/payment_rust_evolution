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

#[async_trait]
impl ClientRepository for SqlxClientRepository {
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

    async fn find_all(&self) -> Result<Vec<Client>, ApiError> {
        let clients = query_as::<Postgres, Client>("SELECT * FROM clients")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| ApiError::DatabaseError(format!("Falha ao buscar clientes: {}", e)))?;

        Ok(clients)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Client>, ApiError> {
        let client = query_as::<Postgres, Client>("SELECT * FROM clients WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| ApiError::DatabaseError(format!("Falha ao buscar cliente por ID: {}", e)))?;

        Ok(client)
    }

    async fn update(&self, id: Uuid, updated_client: UpdateClientDto) -> Result<Option<Client>, ApiError> {
        let mut query_builder = String::from("UPDATE clients SET updated_at = NOW()");
        let mut binds: Vec<Box<dyn sqlx::Encode<'_, Postgres> + Send + Sync>> = Vec::new();
        let mut param_count = 2; 

        if let Some(name) = updated_client.name {
            query_builder.push_str(&format!(", name = ${}", param_count));
            binds.push(Box::new(ClientName(name)));
            param_count += 1;
        }
        if let Some(email) = updated_client.email {
            query_builder.push_str(&format!(", email = ${}", param_count));
            binds.push(Box::new(ClientEmail(email)));
            param_count += 1;
        }
        if let Some(address) = updated_client.address {
            query_builder.push_str(&format!(", address = ${}", param_count));
            binds.push(Box::new(ClientAddress(address)));
            param_count += 1;
        }
        if let Some(plan) = updated_client.plan {
            query_builder.push_str(&format!(", plan = ${}", param_count));
            binds.push(Box::new(plan));
            param_count += 1;
        }

        if param_count == 2 { 
            return self.find_by_id(id).await.map(|c| c.map(|client| client));
        }

        query_builder.push_str(&format!(" WHERE id = ${} RETURNING *", param_count));
        binds.push(Box::new(id));

        let mut query = sqlx::query_as::<Postgres, Client>(&query_builder);
        for bind in binds {
            query = query.bind(bind);
        }

        let client = query
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| {
                if let sqlx::Error::Database(db_err) = &e {
                    if db_err.is_unique_violation() {
                        return ApiError::Conflict(format!("Email já cadastrado: {}", updated_client.email.clone().unwrap_or_default()));
                    }
                }
                ApiError::DatabaseError(format!("Falha ao atualizar cliente: {}", e))
            })?;

        Ok(client)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, ApiError> {
        let result = query("DELETE FROM clients WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| ApiError::DatabaseError(format!("Falha ao deletar cliente: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }
}

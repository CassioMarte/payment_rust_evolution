use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{NaiveDateTime, Utc};
use validator::Validate;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Validate)]
pub struct ClientName(
 // #[validate(length(min = 1, message = "Client name cannot be empty"))]
  #[validate(length(min = 3, max = 100, message = "Client name must be between 3 and 100 characters."))]
  pub String,
);

impl From<String> for ClientName {
    fn from(value: String) -> Self {
        ClientName(value)
    }
}

impl fmt::Display for ClientName {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Validate)]
pub struct ClientEmail(
  #[validate(email(message = "Email must be a valid email address"))]
  pub String,
);

impl From<String> for ClientEmail {
    fn from(email: String) -> Self {
        ClientEmail(email)
    }
}

impl fmt::Display for ClientEmail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Validate)]
pub struct ClientAddress(
  #[validate(length(min = 5, max = 200, message = "Client address must be between 5 and 200 characters."))]
  pub String,
);

impl From<String> for ClientAddress {
    fn from(address: String) -> Self {
        ClientAddress(address)
    }
}

impl fmt::Display for ClientAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "plan_type", rename_all = "lowercase")]
pub enum PlanType {
    Diaria,
    Mensal,
    Trimestral,
    Semestral,
    Anual,
}

impl fmt::Display for PlanType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlanType::Diaria => write!(f, "diaria"),
            PlanType::Mensal => write!(f, "mensal"),
            PlanType::Trimestral => write!(f, "trimestral"),
            PlanType::Semestral => write!(f, "semestral"),
            PlanType::Anual => write!(f, "anual"),
        }
    }
}


// Model Principal do Cliente
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
pub struct Client {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub address: String,
    pub plan: PlanType,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Validate)]
pub struct CreateClientDto{
    #[validate]
    pub name: ClientName,
    #[validate]
    pub email: ClientEmail,
    #[validate]
    pub address: ClientAddress,
    #[validate]
    pub plan: PlanType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Validate)]
pub struct UpdateClientDto {
    #[validate(length(min = 3, max = 100, message = "Nome deve ter entre 3 e 100 caracteres"))]
    pub name: Option<String>,
    #[validate(email(message = "Email inválido"))]
    pub email: Option<String>,
    #[validate(length(min = 5, max = 200, message = "Endereço deve ter entre 5 e 200 caracteres"))]
    pub address: Option<String>,
    pub plan: Option<PlanType>,
}
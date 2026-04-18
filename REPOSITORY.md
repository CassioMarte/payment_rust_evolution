# Repository

Neste projeto criamos em repository: 
client_repository.rs  -> O contrato
                         O que o repository deve ter e fazer
                         Não sabe como fazer só define o quê.
                         Ex: 
                          ````
                          async fn create(&self, new_client: CreateClientDto) -> Result<Client, ApiError>;
                          ````

sqlx_client_repository.rs -> A implementação 
                             Como fazer usando SQLX + PostgreSQL
                             Sabe como fazer escreve o SQL de verdade
                            EX:
                            ````
                            pub async create(pool: $self, new_client: CreateClienDto)-> Result<Client, ApiErro>{
                              let client = query_as::<Postgres, Client>(
                                "INSERT INTO ..."
                              )

                              OK(client)
                            }
                            ````
               

- Porque da separação se o codigo fica mais verboso? 

O contrato existe pra quando o projeto cresce. É uma decisão de arquitetura, não de funcionamento. O código sem o contrato roda igual — ele só fica mais difícil de testar, de trocar, e de manter com o tempo.
É o tipo de coisa que parece burocracia no início e você agradece 6 meses depois.

Então a resposta honesta é:
Se você nunca vai testar, nunca vai trocar de banco, e trabalha sozinho, pode usar direto e não perde quase nada.

- Porque do Repository se posso fazer tudo no service ou tudo no handler?

ex:
````
pub struct ClientService {
    pool: PgPool, // ← o service CONHECE o banco
}

impl ClientService {
    pub async fn create_client(&self, dto: CreateClientDto) -> Result<Client, ApiError> {
        sqlx::query_as("INSERT INTO clients...")
            .fetch_one(&self.pool) // ← o service FALA com o banco diretamente
            .await
    }
}
````
O service virou duas coisas ao mesmo tempo: lógica de negócio + acesso a dados. Isso viola o Princípio da Responsabilidade Única.


Com separação — desacoplado:
O service não sabe qual banco usa ele só conhece o contrato
ex: 
````
pub struct ClientService{
  repository: Arc<dyn ClientRepository> // contrato
}
// Para trocar de banco → cria novo arquivo de implementação
// o service, handler, routes → não mudam nada

impl ClientService {
    pub fn new(repository: Arc<dyn ClientRepository>) -> Self {
        ClientService { repository }
    }

    pub async fn create_client(&self, new_client: CreateClientDto) -> Result<Client, ApiError> {
        // A validação de entrada já ocorre no DTO (CreateClientDto) via #[validate]
        // e no Newtype Pattern. Se chegou aqui, os dados básicos são válidos.
        self.repository.create(new_client).await
    }
  }
````


### Contrato

obs: 
#[async_trait] 
// O Rust tem uma limitação:
// traits NÃO suportam async fn nativamente

async_trait resolve isso
````
#[async_trait]
pub trait ClientRepository {
    async fn create(&self, ...) -> Result<Client, ApiError>;
    // Funciona! A macro transforma por baixo
}
````

Você escreve:
````
#[async_trait]
pub trait ClientRepository {
    async fn create(&self, ...) -> Result<Client, ApiError>;
}

// A macro transforma em:
pub trait ClientRepository {
    fn create(&self, ...) -> Pin<Box<dyn Future<Output = Result<Client, ApiError>> + Send>>;
    // Future = o tipo que representa uma operação assíncrona
    // Pin<Box<...>> = necessário para o Rust gerenciar o Future na memória
}
// Você não precisa escrever isso — a macro faz por você
````

````
use async_trait::async_trait;
use uuid::Uuid;

use crate::model::client_model::{
    Client, CreateClientDto, UpdateClientDto
}

use crate::errors::ApiErros;

// #[async_trait] → macro que permite async fn neste trait
// pub trait → contrato público que qualquer struct pode implementar
// Send + Sync → garante que pode ser usado entre threads com segurança
// Send  = pode ser ENVIADO para outra thread
// Sync  = pode ser ACESSADO por múltiplas threads ao mesmo tempo
// necessário porque o Actix usa múltiplas threads

#[async_trait]
pub trait ClientRepository: Send + Sync{

    // Cada linha é uma PROMESSA:
    // "Quem implementar este trait DEVE ter estas funções"

    async fn create(&self, new_client: CreateClientDto) -> Result<Client, ApiError>;

    async fn find_all(&self) -> Result<Vec<Client>, ApiError>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Client>>, ApiError>;

    async fn update(&self, id: Uuid, update_client: UpdateClientDto) -> Result<Option<Client>, ApiError>;
    
     async fn delete(&self, id: Uuid) -> Result<bool, ApiError>;

}
````


````
use async_trait::async_trait;
use sqlx::{query, query_as, PgPool, Postgres};
use uuid::Uuid;
use chrono::Utc;

use crate::errors::ApiError;
use crate::models::client::{
    Client, CreateClientDto, UpdateClientDto,
    ClientName, ClientEmail, ClientAddress, PlanType
};

use crate::traits::client_repository::ClientRepository;

pub struct SqlxClientRepository {
    pool: PgPool, // ← a conexão com o banco
}

impl SqlxClientRepository {
    pub fn new(pool:PgPool) -> Self {
        SqlxClientRepository {pool}
    }
}

#[async_trait]
impl ClientRepository for SqlxClientRepository{

    async fn create(&self, new_client: CreateClientDto) -> Result<Client, ApiError> {
        let client = query_as::<Postgres, Client>(
            "INSERT INTO clients (name, email, address, plan, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6) RETURNING *"
        )
        // .0 acessa o String dentro do Value Object
        // ClientName("João").0 → "João"
        .bind(new_client.name.0)
        .bind(new_client.email.0)
        .bind(new_client.address.0)
        .bind(new_client.plan)
        .bind(Utc::now().naive_utc()) // created_at
        .bind(Utc::now().naive_utc()) // updated_at
        .fetch_one(&self.pool)
        .await
        // map_err → transforma sqlx::Error em ApiError
        // com tratamento específico para email duplicado
        .map_err(|e| {
            // Verifica se é erro de violação de unicidade (email duplicado)
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.is_unique_violation() {
                    // erro específico com mensagem clara
                    return ApiError::Conflict(
                        format!("Email já cadastrado: {}", new_client.email.0)
                    );
                }
            }
            // qualquer outro erro do banco
            ApiError::DatabaseError(format!("Falha ao criar cliente: {}", e))
        })?;

        Ok(client)
    }

    async fn find_all(&self) -> Result<Vec<Client>, ApiError> {
        let clients = query_as::<Postgres, Client>("SELECT * FROM clients")
            .fetch_all(&self.pool)
            .await
            // map_err simples — só um tipo de erro possível aqui
            .map_err(|e| ApiError::DatabaseError(
                format!("Falha ao buscar clientes: {}", e)
            ))?;

        Ok(clients)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Client>, ApiError> {
        let client = query_as::<Postgres, Client>(
            "SELECT * FROM clients WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool) // Option<Client> — não erro se não encontrar
        .await
        .map_err(|e| ApiError::DatabaseError(
            format!("Falha ao buscar cliente por ID: {}", e)
        ))?;

        Ok(client)
    }

}
````
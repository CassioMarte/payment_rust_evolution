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

    // Update a parte mais complexa
    // sql dinâmico baseado nos campos recebidos
    async fn update(
        &self,
        id: Uuid,
        updated_client: UpdateClientDto
    ) -> Result<Option<Client>, ApiError> {

        // Começa com a base do SQL — updated_at sempre atualiza
        let mut query_builder = String::from(
            "UPDATE clients SET updated_at = NOW()"
        )
        
         // Vec de valores a fazer bind — dinâmico pois não sabe quais campos virão
        // Box<dyn sqlx::Encode> = qualquer tipo que o SQLx saiba bindар
        let mut binds: Vec<Box<dyn sqlx::Encode<'_, Postgres> + Send + Sync>> = Vec::new();

        // Começa em 2 pois $1 será reservado para o WHERE id = $1 no final
        let mut param_count = 2;


        // Adiciona só os campos que vieram preenchidos (Some)
        // None = cliente não quer mudar esse campo → não inclui no SQL
        if let Some(name) = updated_client.name{
            // ex: ", name = $2"
            query_builder.push_str(&format!(
                ", name = ${}", param_count
            ))
            
            // empacota o valor para bind posterior
            binds.push(Box::new(ClientName(name)))
            param_count += 1; // próximo parâmetro será $3
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


        // Se nenhum campo foi enviado além de updated_at
        // param_count ainda é 2 → nada foi adicionado
        // Retorna o cliente atual sem fazer UPDATE desnecessário
        if param_count == 2{
            return self.find_by_id(id).await
                .map(|c| c.map(|client| client));
                // map(|c| c) → se Ok(Some(client)) retorna Ok(Some(client))
                // se Ok(None) retorna Ok(None)
        }

        // Fecha o SQL com o WHERE e o RETURNING
        // ex final: "UPDATE clients SET updated_at = NOW(), name = $2 WHERE id = $3 RETURNING *"
        query_builder.push_str(&format!(" WHERE id = ${} RETURNING *", param_count));

        // O id é o ÚLTIMO bind
        binds.push(Box::new(id));

        // Constrói a query com todos os binds dinâmicos
        let mut query = sqlx::query_as::<Postgres, Client>(&query_builder);
        for bind in binds {
            query = query.bind(bind); // adiciona cada valor em ordem
        }

         let client = query
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| {
                // Verifica email duplicado no update também
                if let sqlx::Error::Database(db_err) = &e {
                    if db_err.is_unique_violation() {
                        return ApiError::Conflict(format!(
                            "Email já cadastrado: {}",
                            // unwrap_or_default → se email for None usa ""
                            updated_client.email.clone().unwrap_or_default()
                        ));
                    }
                }
                ApiError::DatabaseError(format!("Falha ao atualizar cliente: {}", e))
            })?;

        Ok(client)

        // Olhar logo abaixo explicação visual:
    }

    async fn delete(&self, id: Uuid) -> Result<bool, ApiError> {
        let result = query("DELETE FROM clients WHERE id = $1")
            .bind(id)
            // execute → não retorna linhas — só quantidade afetada
            .execute(&self.pool)
            .await
            .map_err(|e| ApiError::DatabaseError(
                format!("Falha ao deletar cliente: {}", e)
            ))?;

        // rows_affected() → quantas linhas foram deletadas
        // > 0 = encontrou e deletou → true
        // = 0 = não encontrou → false
        Ok(result.rows_affected() > 0)
    }
}
````

- explicação visual:

1- Cliente envia: { "name": "João" }
  Só name veio (email e address são None)

  O Sql construuído fica:
  "UPDATE clients SET 
        update_at = NOW(), 
        name= $2
      where id = $3
      RETURNING *   
   "
   .bind $2 = "João"
   .bind $3 = uuid

2- Cliente envia: { "name": "João", "email": "joao@email.com" }

    O SQL construído fica:
      "UPDATE clients SET 
            updated_at = NOW(), 
            name = $2,
            email = $3
           WHERE id = $4
           RETURNING *"
    bind $2 = "João"
    bind $3 = "joao@email.com"
    bind $4 = uuid
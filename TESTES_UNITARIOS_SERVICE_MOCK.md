# Testes Unitarios Service (Regras de negocio):

Agora vamos para o nível Sênior: Mocks.
O nosso ClientService depende do ClientRepository. Para testar o Service sem precisar do Postgres, vamos criar um "Repositório de Mentira" (Mock) usando a crate mockall.
Isso permite testar cenários difíceis, como: "O que acontece se o banco de dados retornar um erro específico no meio da criação?".
Vou atualizar o src/traits/client_repository.rs para habilitar o Mock.

````
// O service DEPENDE do repository
pub struct ClientService {
    repository: Arc<dyn ClientRepository>, // ← precisa disso para funcionar
}

// Para testar o service você precisa de um repository
// O único repository real que existe é o SqlxClientRepository
// Que precisa de banco de dados

// PROBLEMA:
// Teste unitário + banco de dados = lento, frágil, complexo
// Queremos testar o SERVICE — não o banco!


// ❌ SEM mock — teste depende do banco
#[tokio::test]
async fn test_create_client() {
    let pool = PgPool::connect(&url).await?; // ← precisa do banco!
    let repo = SqlxClientRepository::new(pool);
    let service = ClientService::new(Arc::new(repo));
    // lento, precisa de banco rodando, pode falhar por problema de infra
}

// ✅ COM mock — teste isolado, sem banco
#[tokio::test]
async fn test_create_client() {
    let mock_repo = MockClientRepository::new(); // ← fake do banco
    let service = ClientService::new(Arc::new(mock_repo));
    // rápido, sem dependências, sempre funciona
}
````

Mock = um repository FALSO que simula o banco
Você controla o que ele retorna em cada teste

- Repository REAL:
create() → vai no banco → INSERT → retorna Client

- Repository MOCK (falso):
create() → você diz o que retornar → retorna o que você mandar
sem banco, sem SQL, sem internet — puro Rust

- Versão 1 —> mock!{} manual

````
// client_repository.rs — trait simples no arquivo de repository
#[async_trait]
pub trait ClientRepository: Send + Sync {
    async fn create(&self, ...) -> Result<Client, ApiError>;
    // ...
}


// Você escreve o mock do zero no arquivo de teste
use mockall::mock

mock! {
    pub ClientRepositoryMock {} // cria a struct

    #[async_trait]
    impl ClientRepository for ClientRepositoryMock {
         // repete TODAS as assinaturas manualmente
        async fn create(&self, new_client: CreateClientDto) -> Result<Client, ApiError>;
        async fn find_all(&self) -> Result<Vec<Client>, ApiError>;
        async fn find_by_id(&self, id: Uuid) -> Result<Option<Client>, ApiError>;
        async fn update(&self, id: Uuid, updated_client: UpdateClientDto) -> Result<Option<Client>, ApiError>;
        async fn delete(&self, id: Uuid) -> Result<bool, ApiError>;
    }
}

// OBS: se mudar qualquer coisa trait ClientRepository: Send + Sync tem que mudar aqui impl ClientRepository for ClientRepositoryMock também
````

- Versão 2 —> #[cfg_attr(test, mockall::automock)]

````
// cfg_attr -> aplica o atributo APENAS na condição
// test     -> a condição: só em `cargo test`
// mockall::automock -> gera MockClientRepository automaticamente

#[cfg_attr(test, mockall::automock)] //-> em modo de test gera mock automaticamente
#[async_trait]
pub trait ClientRepository: Send + Sync {
    async fn create(&self, ...) -> Result<Client, ApiError>;
    ...

}


// No arquivo de teste — só importa e usa
use crate::traits::client_repository::MockClientRepository; // ← já existe!
// sem precisar escrever mock!{} manualmente
````

Quando você adiciona #[cfg_attr(test, mockall::automock)]
o mockall cria por baixo dos panos:

````
pub struct MockClientRepository {
    // campos internos do mockall
}

impl MockClientRepository {
    pub fn new() -> Self { ... }

    // Para CADA método do trait, gera um expect_*:
    pub fn expect_create(&mut self) -> ... { ... }
    pub fn expect_find_all(&mut self) -> ... { ... }
    pub fn expect_find_by_id(&mut self) -> ... { ... }
    pub fn expect_update(&mut self) -> ... { ... }
    pub fn expect_delete(&mut self) -> ... { ... }
}

// Você não escreve nada disso — o mockall faz por você
````

| Comparação | `mock!{}` manual | `automock` |
|-----------|------------------|------------|
| Onde fica | Arquivo de teste | No próprio trait |
| Manutenção | ⚠️ Atualizar em dois lugares | ✅ Atualiza automaticamente |
| Visibilidade | Só no teste | Disponível em todo módulo de teste |
| Código extra | ❌ Mais verboso | ✅ Mais limpo |
| Recomendado | ❌ | ✅ |


- Teste service:

A parte de teste é igual sendo o mock manual ou automatico.

````
use super::client_service::ClientService;
use crate::errors::ApiError;
use crate::models::client::{
    Client, ClientAddress, ClientEmail, ClientName,
    CreateClientDto, PlanType, UpdateClientDto
};
use crate::traits::client_repository::ClientRepository;
use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;
use chrono::{Utc, NaiveDateTime};

// se forma mock manual aqui importamos use mockall::mock;  
// e criamos:
// mock! {
// Define a struct MockClientRepositoryMock
// mockall gera automaticamente todos os métodos expect_*
//    pub ClientRepositoryMock {}
//
// Implementa o trait para o mock
//    #[async_trait]
//    impl ClientRepository for ClientRepositoryMock {}
//}

// Helper para não precisar repetir codigo

fn create_mock_client() -> Client {
    Client {
        id: Uuid::new_v4(),                    // UUID aleatório
        name: "Test Client".to_string(),
        email: "test@example.com".to_string(),
        address: "123 Test St".to_string(),
        plan: PlanType::Anual,
        created_at: Utc::now().naive_utc(),    // timestamp atual
        updated_at: Utc::now().naive_utc(),
     }
}

#[tokio::test]
async fn test_create_client_sucess() {
    //mut arquivo mutavel pois vamos alterar ele 
    let mut mock_repo = MockClientRepository::new();

     // O client que o mock vai "retornar" quando create() for chamado
    let expected_client = create_mock_client();

    // O DTO que vamos enviar ao service 
    let new_client = CreateClientDto {
        name: ClientName("Test Client".to_string()),
        email: ClientEmail("test@example.com".to_string()),
        address: ClientAddress("123 Test St".to_string()),
        plan: PlanType::Anual,
    }

    // configuração do mock -> explicação no final deste service
    mock_repo.expect_create()
      .once()
      .withf(move |dto| {
        dto.name.0 == "Test Client" &&
        dto.email.0 == "test@example.com" &&
        dto.address.0 = "123 Test St" &&
        dto.plan == PlanType::Anual
      })
      .return_once(|_| Ok(expected_client.clone()));
    
    // mock_repo simula a ação que o repository faz e o retorno do banco esperado
    let service = ClientService::new(Arc::new(mock_repo));

    let result = service.create_client(new_client_dto).await;

    assert!(result.is_ok());

    let client = result.unwrap();

    assert_eq!(client.name, expected_client.name);
    assert_eq!(client.email, expected_client.email);
}

#[tokio::test]
async fn test_feat_all(){
    let mut mock_repo = MockClientRepository::new();

    // como feat espera um Vec<Client>
    // Vec com apenas 1 client controlamos o que o "banco" retorna
    let expected_clients = Vec![create_mock_client()];

    mock_repo.expect_find_all()
        .once()
        .return_once(|_| Ok(expected_clients.clone()));
    
    // find_all() não tem argumentos
    // mas return_once ainda recebe |_| por padrão do mockall

    let service = ClientService::new(Arc::new(mock_repo));
    let result = service.get_all_clients().await;

    assert!(result.is_ok());
    // Verifica se retornou exatamente 1 item — o que colocamos no mock
    assert_eq!(result.unwrap().len(), 1);
}
````

// configuração do mock de create:
````
// expect_create() -> gerado pelo mockall para o método create()
mock_repo.expect_create()
        // .once() -> afirma que o create vai se chamado exatamente uma vez
        // se for chamado 0 ou 2 vezes → teste falha automaticamente
        .once()

        // withf -> verifica os argumentos recebido
        // dto -> o CreateClientDto que chega no mock
        // || (closure) -> retorna true se os args estão corretos 
        // e retornar false se o  teste falha: "argumentos incorretos"
        .withf(move |dto| {
            dto.name.0 == "New Client" &&
            dto.email.0 == "new@example.com" &&
            dto.address.0 == "456 New Ave" &&
            dto.plan == PlanType::Anual
        })

        // return_once -> o que retorna quando chamado
        // |_| clousere -> ignora o argumento recebido
        // Ok(expected_client.clone()) -> simula banco retornando o client
        .return_once(|_| Ok(expected_client.clone()));

````
# Testes Unitarios Service (Regras de negocio):

Agora vamos para o nível Sênior: Mocks.
O nosso ClientService depende do ClientRepository. Para testar o Service sem precisar do Postgres, vamos criar um "Repositório de Mentira" (Mock) usando a crate mockall.
Isso permite testar cenários difíceis, como: "O que acontece se o banco de dados retornar um erro específico no meio da criação?".
Vou atualizar o src/traits/client_repository.rs para habilitar o Mock.

````
// Problema dos testes de service:
// O service precisa do repository
// O repository precisa do banco de dados
// Banco de dados em teste unitário = lento, complexo, frágil

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


Versão 1 — mock!{} manual
````
// client_repository.rs — trait simples
#[async_trait]
pub trait ClientRepository: Send + Sync {
    async fn create(&self, ...) -> Result<Client, ApiError>;
    // ...
}

// No arquivo de teste — você cria o mock manualmente
use mockall::mock;

mock! {
    pub ClientRepositoryMock {} // ← você cria a struct mock

    #[async_trait]
    impl ClientRepository for ClientRepositoryMock {
        async fn create(&self, ...) -> Result<Client, ApiError>;
        // ← repete todas as assinaturas manualmente
        // se mudar o trait → tem que mudar aqui também
    }
}
````

Versão 2 — #[cfg_attr(test, mockall::automock)]
````
// client_repository.rs — trait com automock
#[cfg_attr(test, mockall::automock)]
// ↑ "em modo teste, gera o mock automaticamente"
// cfg_attr = aplica o atributo APENAS na condição
// test     = a condição: só em `cargo test`
// mockall::automock = gera MockClientRepository automaticamente

#[async_trait]
pub trait ClientRepository: Send + Sync {
    async fn create(&self, ...) -> Result<Client, ApiError>;
    // ← define uma vez → mock gerado automaticamente
    // se mudar o trait → mock atualiza sozinho ✅
}

// No arquivo de teste — só importa e usa
use crate::traits::client_repository::MockClientRepository; // ← já existe!
// sem precisar escrever mock!{} manualmente
````

| Comparação | `mock!{}` manual | `automock` |
|-----------|------------------|------------|
| Onde fica | Arquivo de teste | No próprio trait |
| Manutenção | ⚠️ Atualizar em dois lugares | ✅ Atualiza automaticamente |
| Visibilidade | Só no teste | Disponível em todo módulo de teste |
| Código extra | ❌ Mais verboso | ✅ Mais limpo |
| Recomendado | ❌ | ✅ |



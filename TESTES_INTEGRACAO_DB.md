# Testes Integração DB:

- utils.rs

Função compartilhada por TODOS os testes de integração
Configura o banco de dados em estado limpo antes de cada teste

````
use sqlx::{PgPool, migrate::Migrator, query};
use std::env;

pub async fn setup_test_db() -> PgPool {

  // Lê a URL do banco de teste 
  // deveria ser DATABASE_TEST_URL para não misturar com produção!
  let database_url = env::var("DATABASE_TEST_URL")
        .expect("DATABASE_TEST_URL must be set for testing");

  // Conecta ao banco de teste
  let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to Postgres for testing.")

  // Migrator -> lê os arquivos .sql da pasta migrations
  // e os executa em ordem no banco
  // garante que as tabelas existem antes dos testes
  let migrator = Migrator::new(std::path::Path::new("./migrations"))
      .await
      .expect("Failed to create migrator");

   migrator.run(&pool)
        .await
        .expect("Failed to run migrations");

  // TRUNCATE -> apaga TODOS os dados da tabela clients
  // RESTART IDENTITY -> reseta os IDs sequenciais para 1
  // CASCADE -> apaga também dados de tabelas relacionadas
  // Garante que cada teste começa com banco VAZIO e LIMPO
  query("TRUNCATE TABLE clients RESTART IDENTITY CASCADE")
        .execute(&pool)
        .await
        .expect("Failed to clean database before tests");

   pool // retorna o pool pronto para uso
}
````
# MAIN

A main.rs é a porteira do seu servidor ela carrega configurações,
conecta e abre o banco e abre a porta para receber requisições."


Todo programa rust tem uma main
````
fn main(){
  // programa começa aqui
  // programa termina aqui
}
````

No caso de uma main assíncrona para suportar await e erros 
usamos macro e IO

````
#[actix_web::main] // macro — o Actix configura o runtime
async fn main() -> std::io::Result<()>{
   // std::io::Result<()> → pode retornar erro de I/O
   // ex: porta 8080 já ocupada → retorna Err
   // ex: tudo ok → retorna Ok(())
}

// A macro transforma isso num main síncrono por baixo
// que inicializa o runtime assíncrono do Tokio
````


Estrutura inicial do projeto:
````
// web -> ferramenta de roteamento de dados
// App -> estrutura principal da aplicação web (aplicação em si)
// HttpServer -> servidor http
use actix_web::{web, App, HttpServer};

// postgres::PgPoolOptions -> como configurar o pool
// Pool -> o pool pronto
// Postgres -> qual é o banco
use sqlx::{postgres::PgPoolOptions, Pool, Postgres}; // mais informações no SQLX.md

// carrega as variáveis de ambiente
use dotenv::dotenv;

// lê variaveis de ambiente do sistema
use std::env

mod models; // structs e enums dos dados
mod repositories; // queries SQL
mod services; // regras de negócios
mod handlers; // funções dos endpoints
mod routes; // mapeamento de URLs
mod erros; // tipos de erro customizados
mod traits; // contratos/interfaces reutilizáveis
mod utils; // funções utilitárias 


// Macro que transforma o main em runtime assíncrono do Actix
#[actix_web::main]
async fn main()-> std::io::Result<()>{
  // Carrega o .env
  dotenv().ok();

  // Lê a URL do banco — trava com mensagem clara se não existir
  let database_url = env::var("DATABASE_URL")
      .expect("DATABASE_URL must be set");

  // Cria o pool de conexão 
  let pool = PgPoolOptions::new()
     .max_connections(5) // Máximo de conexões simultâneas
     .connect(&database_url)
     .await
     .expect("Failed to create pool.")

  // Cria um servidor HTTP
  HttpServer::new(move || {
    // Compartilha o pool com todos os handlers via injeção
    App::new()
       .app_data(web::Data::new(pool.clone()))
       // ← aqui entrarão as rotas quando os módulos forem criados
  })
   .bind(("127.0.0.1", 8080))?  // ✅ IP corrigido
   .run()
   .await
}
````

// ═══════════════════════════════════════════════
// INFRAESTRUTURA DE TESTES — no final por design
// #[cfg(test)] → este módulo só existe em modo teste
//               é completamente removido em produção
// ═══════════════════════════════════════════════
#[cfg(test)]
pub mod tests_util; // vai ser uma conexão common com o banco de dados de test
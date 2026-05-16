# Testes Integração Handles (E2E):

- Testes de Integração (Handlers/API): Garantir que o JSON entra, percorre todas as camadas e volta corretamente.

- SETUP — monta a aplicação Actix para testes
Retorna a app configurada e pronta para receber requisições fake

O que é App?

App = a sua aplicação web completa
 É ela que sabe:
 -> quais rotas existem (/clients, /clients/{id})
 -> quais handlers respondem cada rota
 -> quais dados estão disponíveis (pool, service)

 SEM App — o Actix não sabe nada:
 POST /clients -> ??? não existe

 COM App configurada:
 POST /clients -> create_client_handler ✅
 GET  /clients -> get_all_clients_handler ✅

- O que é setup_app() e para que serve?

Em prod -> main.rs — servidor REAL
````
#[actix_web::main]
async fn main() {
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(client_service.clone()))
            .configure(client_routes::config)
    })
    .bind(("127.0.0.1", 8080))? // ← abre porta de rede
    .run()
    .await
}
````

Em teste -> setup_app() é o mesmo que a main mas sem abrir porta:

````
// setup_app() — servidor FAKE para testes
async fn setup_app() {
    test::init_service(          // ← ao invés de HttpServer::new()
        App::new()               // ← mesma App do main.rs
            .app_data(...)       // ← mesmo service
            .configure(...)      // ← mesmas rotas
    ).await
    // Não tem .bind() nem .run()
    // Não abre porta nenhuma
    // Vive só na memória durante o teste
}
````

- Por que precisa da App nos testes?

````
// Para simular uma requisição HTTP você precisa de algo que a receba
// Esse "algo" é a App

// SEM setup_app():
// Você tem a função create_client_handler
// Mas como simular que chegou um POST /clients com JSON?
// Não tem como — você precisaria de um servidor rodando

// COM setup_app():
let app = setup_app().await;  // ← app fake em memória

// Agora você pode simular requisições:
let req = test::TestRequest::post()
    .uri("/clients")
    .set_json(&dto)
    .to_request();

let resp = test::call_service(&app, req).await;
// app recebe o req e processa como se fosse real
// sem abrir porta, sem internet, sem servidor
````

````
  PRODUÇÃO                        TESTE
────────────────────            ────────────────────
Cliente HTTP real               test::TestRequest
       ↓                               ↓
   Internet                     test::call_service
       ↓                               ↓
  Porta 8080                      App em memória
       ↓                               ↓
HttpServer::new(App)            test::init_service(App)
       ↓                               ↓
    Handler                         Handler
       ↓                               ↓
    Service                         Service
       ↓                               ↓
  Repository                      Repository
       ↓                               ↓
Banco produção                  Banco de teste
````

- E2E

````
use actix_web::{test, web, App};
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::ApiError;
use crate::models::client::{...};
use crate::repositories::sqlx_client_repository::SqlxClientRepository;
use crate::services::client_service::ClientService;
use crate::tests_util::setup_test_db; // ← setup compartilhado
use crate::routes::client_routes;


// Retorna a app configurada e pronta para receber requisições fake
async fn setup_app() -> App<
    actix_web::app::Service<actix_web::app::App<actix_web::app::Service<actix_web::app::App<()>>>>,
> {
    // Banco limpo para chamada
    let pool = setup_test_db().await;

    // Conecta as dependências reais 
    // Repository e Service reais nada de mock aqui
    let client_repository = Arc::new(SqlxClientRepository::new(pool.clone()));
    let client_service = Arc::new(ClientService::new(client_repository));


    // test::init_service → inicializa a app Actix em modo teste
    // Sem abrir porta de rede — tudo em memória
    test::init_service(
        App::new()
            .app_data(web::Data::new(client_service.clone()))
            .configure(client_routes::config), // mesmas rotas do main.rs
    )
    .await
}

````
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


    // test::init_service -> inicializa a app Actix em modo teste
    // Sem abrir porta de rede — tudo em memória
    test::init_service(
        App::new()
            .app_data(web::Data::new(client_service.clone()))
            .configure(client_routes::config), // mesmas rotas do main.rs
    )
    .await
}

#[tokio::test]
async fn test_create_client_handler_sucess(){
    let app = setup_app().await();

    let new_client_dto = CreateClientDto {
        name: ClientName("Test Client".to_string()),
        email: ClientEmail("test@example.com".to_string()),
        address: ClientAddress("123 Test".to_string()),
        plan: PlanType::Mensal,
    }
  
    // Monta a requisição HTTP fake
    let req = test::TestRequest::post()
          .uri("/clients")              // POST/clients
          .set_json(&new_client_dto)    // corpo = JSON do DTO
          .to_request()                 // constrói a requisição

    // Envia a requisição para a app e aguarda a resposta
    let resp = test::call_service(&app, req).await;

    // Verifica o STATUS CODE da resposta
    assert_eq!(resp.status(), StatusCode::CREATED); // exatamente 201
    
    // Lê e desserializa o CORPO da resposta
    let client: Client = test.read_body_json(resp).await;

    // Verifica os DADOS retornados
    // .0 acessa o String dentro do Value Object
    assert_eq!(client.name, new_client_dto.name.0);
    assert_eq!(client.email, new_client_dto.email.0);

}

#[tokio::test]
async fn test_create_client_handler_invalid_input() {
    let app = setup_app().await();

    let new_client_dto = CreateClientDto {
        name: ClientName("a".to_string()),          // 1 char -> min=3 -> inválido
        email: ClientEmail("invalid-email".to_string()), // sem @ -> inválido
        address: ClientAddress("short".to_string()), // 5 chars -> min=5 -> na borda
        plan: PlanType::Mensal,
    };

    let req = test::TestRequest::post()
        .uri("/clients")
        .set_json(&new_client_dto)
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Espera 400 Bad Request — dado inválido
    assert!(resp.status().is_bad_request());
    // ⚠️ PROBLEMA: CreateClientDto usa Value Objects com #[validate]
    // A validação só roda se você chamar .validate() manualmente
    // ou usar Validated<T> extractor
    // Se o handler usa web::Json<CreateClientDto> sem validação
    // esse teste pode PASSAR mesmo com dados inválidos!
}

#[tokio::test]
async fn test_get_all_clients_handler_success() {
    let app = setup_app().await;

    // Cria um client primeiro para a lista não ficar vazia
    let new_client_dto = CreateClientDto {
        name: ClientName("Test Client".to_string()),
        email: ClientEmail("test@example.com".to_string()),
        address: ClientAddress("123 Test".to_string()),
        plan: PlanType::Mensal,
    };

    // Agora busca todos
    let req_create = test::TestRequest::post()
        .uri("/clients")
        .set_json(&new_client_dto)
        .to_request();

    test::call_service(&app, req_create).await; // cria — ignora a resposta

    // Agora busca todos
    let req = test::TestRequest::get()
        .uri("/clients")
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_ok()); // 200

    let clients: Vec<Client> = test::read_body_json(resp).await;

    assert!(!clients.is_empty());

    assert_eq!(clients.len(), 1); // só o que criamos — banco estava limpo
}

#[tokio::test]
async fn test_get_client_by_id_handler_success() {
    let app = setup_app().await;

    let new_client_dto = CreateClientDto {
        name: ClientName("Test Client".to_string()),
        email: ClientEmail("test@example.com".to_string()),
        address: ClientAddress("123 Test".to_string()),
        plan: PlanType::Mensal,
    };


    let req_create = test::TestRequest::post()
        .uri("/clients")
        .set_json(&new_client_dto)
        .to_request();

    let resp_create = test::call_service(&app, req_create).await;

    // Desserializa a resposta do create para pegar o ID gerado
    let created_client: Client = test::read_body_json(resp_create).await;

    // Vamos usar o id na rota aqui
    let req = test::TestRequest::get()
        .uri(&format!("/clients/{}", created_client.id)) // ← ID real
        .to_request();

    let resp = test::call_service(&app. req).await;

    assert!(resp.status().is_ok());

    let client: Client = test::read_body_json(resp).await;

    assert_eq!(client.id, created_client.id); // mesmo ID
} 

#[tokio::test]
async fn test_get_client_by_id_handler_not_found() {
    let app = setup_app().await();

    // UUID gerado aleatoriamente — certamente não existe no banco limpo
    let non_existent_id = Uuid::new_v4();

    let req = test::TestRequest::get()
        .uri(&format!("/clients/{}", non_existent_id))
        .to_request();
    let resp = test::call_service(&app, req).await;

    // Espera 404 Not Found
    assert!(resp.status().is_not_found());
}

#[tokio::test]
async fn test_update_client_handler_success() {
    let app = setup_app().await;

    let new_client_dto = CreateClientDto{
        name: ClientName("Test Up".to_string()),
        emaii: ClientEmail("client_email@ex.com".to_string()),
        address: ClientAddress("123 ad".to_string()),
        plan: PlanType::Mensal
    }

    let req_create = test::TestRequest::post()
        .uri("/clients")
        .set_json(&new_client_dto)
        .to_request();

    // Cria o client original
    let resp_create = test::call_service(&app, req_create).await;
    let created_client: Client = test::read_body_json(resp_create).await;

    let updated_dto = UpdateClientDto {
        name: Some("Updated Client 3".to_string()),
        email: None,                // não muda
        address: None,              // não muda
        plan: Some(PlanType::Anual),
    }

    let req = test:TestRequest::put()
        .uri("/clients/{}", created_client.id)
        .set_json(&updated_dto)
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_ok());

    let client: Client = test::read_body_json(resp).await;

    // Verifica que os campos mudaram
    assert_eq!(client.name, "Updated Client 3");
    assert_eq!(client.plan, PlanType::Anual);
    // email e address não foram enviados → não mudaram
}

#[tokio::test]
async fn test_update_client_handler_not_found() {
    let app = setup_app().await;
    let non_existent_id = Uuid::new_v4();

    let req = test::TestRequest::put()
        .uri(&format!("/clients/{}", non_existent_id))
        .set_json(&updated_dto)
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_not_found()); // 404
}

#[tokio::test]
async fn test_delete_client_handler_success() {
    let app = setup_app().await;

    // 1. Cria
    let created_client: Client = test::read_body_json(resp_create).await;

    // 2. Deleta
    let req = test::TestRequest::delete()
        .uri(&format!("/clients/{}", created_client.id))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_no_content()); // 204

    // 3. Confirma que foi deletado — tenta buscar e espera 404
    let req_get = test::TestRequest::get()
        .uri(&format!("/clients/{}", created_client.id))
        .to_request();
    let resp_get = test::call_service(&app, req_get).await;

    assert!(resp_get.status().is_not_found()); // 404
    // Esta verificação extra é o que torna este teste EXCELENTE
    // Não confia só no status 204 — confirma que realmente sumiu
}


#[tokio::test]
async fn test_delete_client_handler_not_found() {
    let app = setup_app().await;
    let non_existent_id = Uuid::new_v4();

    let req = test::TestRequest::delete()
        .uri(&format!("/clients/{}", non_existent_id))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_not_found()); // 404 
}

````


Usuário real:    Postman/Frontend → Internet → Porta 8080 → main.rs → App → Handler

Teste:           test::TestRequest → test::call_service → setup_app → App → Handler

O test::TestRequest substitui o Postman/Frontend, e o setup_app substitui o main.rs — a App, as rotas e os handlers são exatamente os mesmos. Por isso teste de integração é tão valioso — você testa o código real, só muda quem faz a chamada.
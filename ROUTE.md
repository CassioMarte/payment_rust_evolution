# Routes

````
// Importa as ferramentas de roteamento do Actix
use actix_web::web;
// Importa os handlers que serão associados às rotas
use crate::handlers::client_handler;


// Função de configuração de rotas
// Chamada automaticamente pelo Actix no main.rs via .configure()
pub fn config(cfg: &mut web::ServiceConfig) {

    cfg
        // ═══════════════════════════════════════
        // RECURSO BASE: /clients
        // Rotas que NÃO precisam de ID na URL
        // ═══════════════════════════════════════
        .service(
            web::resource("/clients")

                // POST /clients -> cria um novo cliente
                // corpo da requisição: JSON com CreateClientDto
                .route(web::post().to(client_handler::create_client_handler))

                // GET /clients -> lista todos os clientes
                // sem corpo — só retorna a lista
                .route(web::get().to(client_handler::get_all_clients_handler))
        )

        // ═══════════════════════════════════════
        // RECURSO COM ID: /clients/{id}
        // Rotas que precisam de um UUID na URL
        // ex: /clients/550e8400-e29b-41d4-a716-446655440000
        // ═══════════════════════════════════════
        .service(
            web::resource("/clients/{id}")
            // {id} = parâmetro dinâmico
            // Actix extrai automaticamente via web::Path<Uuid> no handler

                // GET /clients/{id} -> busca um cliente específico
                .route(web::get().to(client_handler::get_client_by_id_handler))

                // PUT /clients/{id} -> atualiza um cliente
                // corpo: JSON com UpdateClientDto (campos opcionais)
                .route(web::put().to(client_handler::update_client_handler))

                // DELETE /clients/{id} -> remove um cliente
                // sem corpo — só precisa do UUID na URL
                // retorna 204 No Content
                .route(web::delete().to(client_handler::delete_client_handler))
        );
}

````


- Main

Agora com as rotas, handlers, service, repository e model precisamos conectar tudo e neste projeto vamos fazer isso na main.rs

````
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool.");

    // Monta a cadeia de dependências:
    // Pool -> Repository -> Service
    let repository = Arc::new(SqlxClientRepository::new(pool));
    let client_service = Arc::new(ClientService::new(repository));

    HttpServer::new(move || {
        App::new()
            // Disponibiliza o service para todos os handlers via injeção
            .app_data(web::Data::new(client_service.clone()))

            // Registra todas as rotas de client de uma vez
            .configure(routes::client_routes::config)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
````
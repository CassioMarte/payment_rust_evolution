# Erros 

- erro genérico sem significado
````
) -> Result<Payment, Box<dyn std::error::Error>>
````

- erros com significado e HTTP code correto
````
) -> Result<Payment, ApiError>
````


## error.rs

````
use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::Serialize;
use std::fmt;
````

## enum
É como criar um "vocabulário de erros" para sua API — 
cada situação tem um nome e um significado claro.

````
// #[derive(Debug)] -> posso imprimir no log com {:?}
// #[derive(Serialize)] -> posso converter para json

#[derive(Debug, Serialize)]
pub enum ApiError{
  // Cada variante carrega uma String — a mensagem do erro
  NotFound(String),            // Recurso não encontrado 404
  InvalidInput(String),        // Dados inválidos do clinte 400
  InternalServerError(String), // Erro inesperado do servidor 500
  DatabaseError(String),       // erro específico do banco 500
  Unaunthorized(String),       // sem permissão → 401
  Conflict(String)             // conflito de dados (ex: duplicado) → 409
}
````

## impl fmt::Display

````
//Display -> define como o erro aparece como texto
// É o que o .to_string() usa
// É o que aparece nos logs do servidor

impl fmt::Display for ApiError {
  // f -> o escritor de text
  // fmt::Result -> Ok se escreveu, Err se falhou 
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
    // write!(f, ...) -> escreve no f
    // {} e substituido pela mensagem

     ApiError::NotFound(msg) => write!(f, "Not found: {}", msg)
                              //|-> aparece no log com: "Not found: Payment not found"

     ApiError::InvalidInput(msg) => write!(f, "Invalid Input: {}", msg),

    ApiError::InternalServerError(msg) =>
                write!(f, "Internal Server Error: {}", msg),

    ApiError::DatabaseError(msg) =>
                write!(f, "Database Error: {}", msg),

    ApiError::Unauthorized(msg) =>
                write!(f, "Unauthorized: {}", msg),

    ApiError::Conflict(msg) =>
                write!(f, "Conflict: {}", msg),
    }
  }
}
````

- Na prática:
````
let err = ApiError::NotFound("Payment not found".to_string());
println!("{}", err); // → "Not Found: Payment not found"
log::error!("{}", err); // → aparece no log do servidor
````

## impl ResponseError

````
//ResponseError -> trait do Actix que transforma ApiError em uma resposta Http
// É o tradutor de erro Rust

impl ResponseError for ApiError {
  fn error_response(&self)-> HttpResponse{
    match self {
      // HttpResponse::build(StatusCode) -> cria resposta com o status code
      // .json(msg) -> corpo da resposta em JSON

      ApiError::NotFound(msg)=>
            HttpResponse::build(StatusCode::NOT_FOUND)
              .json(msg), //404

      ApiError::InvalidInput(msg) =>
              HttpResponse::build(StatusCode::BAD_REQUEST) // 400
               .json(msg),

      ApiError::InternalServerError(msg) =>
               HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR) // 500
                  .json(msg),

      ApiError::DatabaseError(msg) =>
               HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR) // 500
                  .json(msg),
            // DatabaseError e InternalServerError → ambos 500
            // mas separados para o LOG identificar a origem

      ApiError::Unauthorized(msg) =>
               HttpResponse::build(StatusCode::UNAUTHORIZED) // 401
                  .json(msg),

      ApiError::Conflict(msg) =>
              HttpResponse::build(StatusCode::CONFLICT) // 409
                    .json(msg),
       }
    }
}
````

### Na prática — o Actix chama isso automaticamente:

````
pub async fn get_payment_handler(...) -> Result<HttpResponse, ApiError> {
    let payment = service::get_payment(uuid).await?;
    // se retornar Err(ApiError::NotFound("..."))
    // o Actix chama error_response() automaticamente
    // cliente recebe: HTTP 404 { "Not Found: Payment not found" }
    Ok(HttpResponse::Ok().json(payment))
}
````


## impl From<sqlx::Error>

````
// From -> trait de conversão automatica 
// Como transformar um sqlx::Error em ApiError
// É o que permite o `?` funcionar em funções que retornam ApiError

impl From<sqlx::Error> for ApiiError{
  fn from(err: sqlx::Error)-> ApiError{
    // qualquer erro do SQLx vira DatabaseError
    ApiError::DatabaseError(err.to_string())
  }
}
````

- Pq do From?

Sem From (manual) 
```
let payment = sqlx::query_as!(Payment, "SELECT...")
    .fetch_one(&pool)
    .await
    .map_err(|e| ApiError::DatabaseError(e.to_string()))?;
```

Com from (automático)
````
let payment = sqlx::query_as!(Payment, "Select ....")
    .fetch_one(&pool)
    .await? // ← o ? converte sqlx::Error → ApiError automaticamente!
````

## impl From<validator::ValidationErrors>

// mesma ideia do anterior converte erros do validator em ApiError
````
impl From<validator::ValidationErrors> for ApiError {
  fn from(err: validator::ValidationErrors) -> ApiError {
    // serde_json::to_string(&err) -> converte o erro para json string
    // .unwrap_or_else(|_| "Validation error".to_string()) -> se a conversão falhar 
    // use a  mensagem genérica "Validation error"

    ApiError::InvalidInput(
            serde_json::to_string(&err)
                .unwrap_or_else(|_| "Validation error".to_string())
        )
  }
}
````

- Na prática:
```
new_payment.validate()?;
```

 validate() retorna Err(ValidationErrors)
 ? converte ValidationErrors → ApiError::InvalidInput automaticamente
  cliente recebe: HTTP 400 com os detalhes dos erros


Error return:
   Resposta JSON não padronizada
   Hoje retorna só a string da mensagem:
   HttpResponse::build(StatusCode::NOT_FOUND).json(msg)
   
- cliente recebe: "Payment not found"



## Erro com resposta mas completa 

- {
    "error": "NOT_FOUND",
    "code": "resource_not_found",
    "message": "Payment not found"
  }

````
use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::Serialize;
use std::fmt;

//Nova struct que padroniza o resposta de erro

#[derive(Debug, Serialize)]
struct ErrorResponse {
  error: String,   // categoria do erro  → "NOT_FOUND"
  code: String,    // código específico  → "payment_not_found"
  message: String  // mensagem legível   → "Payment not found"
}


//Helper que constrói o ErrorResponse
//evita repetição nos match abaixo

impl ErrorResponse {
  fn new(error: &str, code: &str, message: &str) -> Self {
        ErrorResponse {
            error: error.to_string(),
            code: code.to_string(),
            message: message.to_string(),
        }
    }
}

// Continua igual 
#[derive(Debug, Serialize)]
pub enum ApiError {
    NotFound(String),
    InvalidInput(String),
    InternalServerError(String),
    DatabaseError(String),
    Unauthorized(String), 
    Conflict(String),
}

// Continua igual — é só para os logs do servidor
impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::NotFound(msg) =>
                write!(f, "Not Found: {}", msg),
            ApiError::InvalidInput(msg) =>
                write!(f, "Invalid Input: {}", msg),
            ApiError::InternalServerError(msg) =>
                write!(f, "Internal Server Error: {}", msg),
            ApiError::DatabaseError(msg) =>
                write!(f, "Database Error: {}", msg),
            ApiError::Unauthorized(msg) =>
                write!(f, "Unauthorized: {}", msg),
            ApiError::Conflict(msg) =>
                write!(f, "Conflict: {}", msg),
        }
    }
}

impl ResponseError for ApiError{
  fn error_response(&self) -> HttpResponse {
    match self {
      // 404 — recurso não encontrado
            ApiError::NotFound(msg) =>
                HttpResponse::build(StatusCode::NOT_FOUND)
                    .json(ErrorResponse::new(
                        "NOT_FOUND",        // categoria
                        "resource_not_found", // código
                        msg,                // mensagem dinâmica
                    )),
            // cliente recebe:
            // {
            //   "error": "NOT_FOUND", 
            //   "code": "resource_not_found", 
            //   "message": "Payment not found"
            // }

        // 400 — dado inválido
            ApiError::InvalidInput(msg) =>
                HttpResponse::build(StatusCode::BAD_REQUEST)
                    .json(ErrorResponse::new(
                        "BAD_REQUEST",
                        "invalid_input",
                        msg,
                    )),

         // 500 — erro genérico do servidor
            ApiError::InternalServerError(msg) => {
                // log com detalhes internos — só no servidor
                log::error!("Internal server error: {}", msg);
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .json(ErrorResponse::new(
                        "INTERNAL_SERVER_ERROR",
                        "internal_error",
                        "An unexpected error occurred", // ← genérico para o cliente
                    ))
                // cliente NUNCA vê o msg interno
            }

           // 500 — erro do banco
            // ⚠️ SEGURANÇA: msg nunca vai para o cliente
            ApiError::DatabaseError(msg) => {
                // detalhes do banco ficam só no log
                log::error!("Database error: {}", msg);
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .json(ErrorResponse::new(
                        "INTERNAL_SERVER_ERROR",
                        "database_error",
                        "An unexpected error occurred", // ← genérico para o cliente
                    ))
                // cliente não sabe que foi erro do banco
                // não vaza IP, porta ou query SQL
            }


            ...
    }
  }
}

// Continua igual
impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> ApiError {
        ApiError::DatabaseError(err.to_string())
        // to_string() aqui é ok — vai para o log, não para o cliente
    }
}


impl From<validator::ValidationErrors> for ApiError {
  fn from(err:validator::ValidationErrors) -> ApiError {
    //coleta campo por campo
    let message:Vec<String>  err
        .field_errors()
        .iter()
        .flat_map(|(field, errors)| {
           errors.iter().filter_map(move |e| {
                    e.message.as_ref().map(|m| {
                        format!("{}: {}", field, m) // ex: "amount: O valor deve ser maior que zero"
                    })
                })
        })
        .collect();

        // Junta todas as mensagens
        let message = if messages.is_empty() {
            "Validation error".to_string()
        } else {
            messages.join(", ")
            // ex: "amount: O valor deve ser maior que zero, currency: A moeda não pode ser vazia"
        };

        ApiError::InvalidInput(message)
  }
}
````


// ❌ ANTES
"Payment not found"  // só uma string solta


// ✅ DEPOIS
{
    "error": "NOT_FOUND",
    "code": "resource_not_found",
    "message": "Payment not found"
}

// ❌ ANTES — DatabaseError expõe info interna
{
    "error": "Database Error: connection to server at 192.168.1.1 failed"
}

// ✅ DEPOIS — DatabaseError genérico para o cliente
{
    "error": "INTERNAL_SERVER_ERROR",
    "code": "database_error",
    "message": "An unexpected error occurred"
}
// log do servidor: "Database error: connection to server at 192.168.1.1 failed"


// ❌ ANTES — ValidationErrors perde mensagens
"Validation error"

// ✅ DEPOIS — ValidationErrors com detalhes
{
    "error": "BAD_REQUEST",
    "code": "invalid_input",
    "message": "amount: O valor deve ser maior que zero, currency: A moeda não pode ser vazia"
}


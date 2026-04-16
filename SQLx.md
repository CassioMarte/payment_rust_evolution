# SQLx

SQLx é uma biblioteca que faz seu Rust conversar com o banco de dados
- ela envia SQL e recebe os resultados já convertidos para suas structs.


## postgres::PgPoolOtions

- postgres -> submódulo do sqlx específico para PostgresSQL
- PgPoolOptions -> configurador do pool de conexões 

- Pool -> conjunto de conexões abertas e prontas para uso
É como um estacionamento de conexões:

 ┌─────────────────────────────┐
 │     Pool (5 conexões)       │
 │  [conn1] [conn2] [conn3]    │
 │  [conn4] [conn5]            │
 └─────────────────────────────┘

 Quando o handler precisa do banco ele:
 1- pega uma conxão livre
 2- usa
 3 devolve para pool

````
 PgPoolOptions::new()
  .max_connections(5)     // máximo 5 conexões simultâneas
  .connect(&database_url) // conecta ao banco
  .await                  // aguarda a conexão
````

## Pool

Pool<DB> -> o pool em si depois de configurado

Pool<Postgres> -> o pool conectado ao PostgreSQL

````
let pool: Pool<Postgres> = PgPoolOptions::new()
        .connect(&url)
        .await?
````

PgPoolOptions = o projeto da garagem
Pool          = a garagem construída e funcionando

### OBS:

Postgres -> é o tipo que identifica qual banco está sendo usado

O SQLx suporta varios banco como Postgres, MySQL, SQLite, ...

Pool<> -> funciona como uma etiqueta

````
Pool<Postgres> // este pool fala com PostgreSQL
Pool<MySql>    // este pool fala com MySQL
Pool<Sqlite>   // este pool fala com SQLite
````

Por isso esses dois são equivalentes:
pool: Pool<Postgres>  // explícito
pool: PgPool          // atalho — mais comum no dia a dia

````
use sqlx::{
  postgres::PgPoolOptions, // configuração do pool
  Pool,                    // o pool pronto
  Postgres                 // banco
}

            //tipo          // configuração
let pool : Pool<Postgres> = PgPoolOptions::new()
             .max_connections(5) 
             .connect(&database_url)
             .await
             .expect("Failed to create pool.")

// compartilhar com handlers 
App::new()
     .app_data(web::Data::new(pool.clone()))
                            // copia o pool para cada thread
````
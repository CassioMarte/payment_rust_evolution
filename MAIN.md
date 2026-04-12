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
````
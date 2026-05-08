# Service 
- Injeção de dependêcia e regras de negócio

````
// Arc -> ponteiro de referência, permita compartilhar repository
// entre múltiplas threads com segurança
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::ApiError;
use crate::models::client::{Client, CreateClientDto, UpdateClientDto};
// Importa o CONTRATO — service não conhece a implementação
use crate::traits::client_repository::ClientRepository;


// A struct do service — guarda o repository
pub struct ClientService{
    // Arc<dyn ClientRepository>:
    // Arc     = compartilhável entre threads
    // dyn     = qualquer tipo que implemente ClientRepository
    // ou seja = "não sei qual banco é — só sei que é um repositório"
   repository: Arc<dyn ClientRepository>,
}

impl ClientService {

  // Construtor — recebe qualquer implementação de ClientRepository
  // Pode ser SqlxClientRepository, MockRepository, etc
  pub fn new(repository: Arc<dyn ClientRepository>) -> Self {
       ClientService { repository }
  }


  pub async fn create_client(
        &self,
        new_client: CreateClientDto
    ) -> Result<Client, ApiError> {

        // Validação já aconteceu antes de chegar aqui:
        // -> ClientName, ClientEmail, ClientAddress validam no tipo
        // -> #[validate] no DTO garante os campos
        // Service só delega para o repository
        self.repository.create(new_client).await
        // sem ? aqui — explicado abaixo!
    }

  pub async fn get_all_clients(&self) -> Result<Vec<Client>, ApiError> {
        self.repository.find_all().await
        // sem ? aqui — explicado abaixo!
    }

   // ✅ OBS —> .await? + .ok_or_else()
   //self.repository.update(id, updated_client)
   // .await?                                    // desempacota Result -> Option<Client>
   // .ok_or_else(|| ApiError::NotFound(...))    // converte Option -> Result<Client>
   // mais detalhes abaixo

  pub async fn get_client_by_id(&self, id: Uuid) -> Result<Client, ApiError> {

        self.repository.find_by_id(id)
            .await?   // COM ? aqui — propaga erro do banco
            //           retorna Result<Option<Client>, ApiError>
            //           o ? desempacota para Option<Client>

            .ok_or_else(|| ApiError::NotFound(
                format!("Cliente com ID {} não encontrado", id)
            ))
            // ok_or_else -> converte Option em Result:
            // Ok(Some(client)) -> Ok(client)
            // Ok(None)         -> Err(ApiError::NotFound)
            // Err(e)           -> Err(e) propagado pelo ?
    }

  pub async fn update_client(
        &self,
        id: Uuid,
        updated_client: UpdateClientDto
    ) -> Result<Client, ApiError> {

        self.repository.update(id, updated_client)
            .await?  // COM ? — propaga erro do banco
            .ok_or_else(|| ApiError::NotFound(
                format!("Cliente com ID {} não encontrado para atualização", id)
            ))
        // Mesmo padrão do get_by_id:
        // Ok(Some(client)) -> Ok(client)
        // Ok(None)         -> Err(ApiError::NotFound)
        // Err(e)           -> Err(e) propagado pelo ?
    }

  pub async fn delete_client(&self, id: Uuid) -> Result<(), ApiError> {

        // repository.delete retorna Result<bool, ApiError>
        // true  = encontrou e deletou
        // false = não encontrou
        let deleted = self.repository.delete(id).await?;
        // ? aqui propaga o ApiError se o banco falhar

        if deleted {
            Ok(())   // deletou com sucesso — retorna vazio
        } else {
            // false = não encontrou o cliente
            Err(ApiError::NotFound(
                format!("Cliente com ID {} não encontrado para exclusão", id)
            ))
        }
    }
}
````

- Por que sem ? no create e find_all?

Com `?` propaga e sai da função se Err

 ````
 self.repository.create(new_client).await?;

 retorna Ok(client) ou sai com Err
 ````

 Sem `?` retorna o Result inteiro (no caso Result<Client, ApiError>)

  ````
 self.repository.create(new_client).await;

 retorna Ok(client) ou Err — mas quem decide o que fazer é quem chamou
 ````



OBS:
````
pub async fn create_client(...) -> Result<Client, ApiError> {
    let client = self.repository.create(new_client).await?; // 1. Extrai o Client
    Ok(client) // 2. Envelopa de novo em Ok()
}
Isso funciona? Sim. Mas é redundante (mais código para fazer a mesma coisa). ]
````


O TIPO de retorno da função:
pub async fn create_client(...) -> Result<Client, ApiError>

O TIPO que o repository retorna:
repository.create(...) -> Result<Client, ApiError>

- São IDÊNTICOS!

 A função pode simplesmente retornar o Result do repository diretamente sem precisar desempacotar com ? e reempacotar com Ok()

````
pub async fn create_client(...) -> Result<Client, ApiError> {
    self.repository.create(new_client).await // ← devolve o Result diretamente
}

// É equivalente a isso — mas mais verboso:
pub async fn create_client(...) -> Result<Client, ApiError> {
    let result = self.repository.create(new_client).await;
    match result {
        Ok(client) => Ok(client),  // reempacota — redundante!
        Err(e) => Err(e),          // reempacota — redundante!
    }
}
````


-  Precisa do `?` quando você quer FAZER ALGO com o valor antes de retornar

ex:
````
pub async fn get_client_by_id(&self, id: Uuid) -> Result<Client, ApiError> {

    self.repository.find_by_id(id).await?
    //                               ↑
    //              PRECISA do ? aqui porque:
    //              repository retorna Result<Option<Client>, ApiError>
    //              função retorna Result<Client, ApiError>
    //              são TIPOS DIFERENTES — não pode retornar direto!
    //
    //              o ? desempacota o Result → sobra Option<Client>
    //              aí o .ok_or_else() converte Option → Result
            .ok_or_else(|| ApiError::NotFound(...))
}
````

Resumo:

````
create — tipos IGUAIS — sem ?
─────────────────────────────
repository → Result<Client, ApiError>
função     → Result<Client, ApiError>
            ↑ mesmos — retorna direto ✅


get_by_id — tipos DIFERENTES — com ?
──────────────────────────────────────
repository → Result<Option<Client>, ApiError>
                    ↑ tem Option!
função     → Result<Client, ApiError>
                    ↑ sem Option!

? desempacota → Option<Client>
ok_or_else   → Result<Client, ApiError> ✅
````


Resumo: o ? é necessário quando você precisa fazer algo com o valor antes de retornar. Quando o tipo de retorno do repository é idêntico ao da função, você pode retornar o Result diretamente sem desempacotar e reempacotar — é mais limpo e mais idiomático em Rust.


- await?.ok_or_else ->

````
self.repository.update(id, updated_client)
.await?
.ok_or_else(|| ApiError::NotFound(...))
````

self.repository.update(id, update_client) // tipo até aqui: `Future<Result<Option<Client>, ApiError>>`

.await // tipo até aqui: `Result<Option<Client>, ApiError>`

? 
// ? desempacota o Result:
// se Err(e) -> sai da função com Err(e)
// se Ok(v)  -> continua com v
// tipo até aqui: Option<Client>

.ok_or_else(|| ApiError::NotFound(...))
// converte Option em Result:
// Some(client) → Ok(client)
// None         → Err(ApiError::NotFound(...))
// tipo até aqui: Result<Client, ApiError> ✅ bate com o retorno da função

Resumo:

````
.await?
  ↓
Result<Option<Client>, ApiError>
  ↓ ? abre o Result
Option<Client>
  ↓ .ok_or_else() converte
Result<Client, ApiError>  ← o que a função precisa retornar ✅
````

São três passos encadeados — .await executa o Future, ? abre o Result (e sai se for erro), .ok_or_else() converte o Option em Result. Cada um faz exatamente uma coisa e os três juntos transformam Future<Result<Option<Client>>> no Result<Client> que a função precisa retornar.

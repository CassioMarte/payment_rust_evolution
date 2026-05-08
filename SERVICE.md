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

 
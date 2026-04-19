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
}
````
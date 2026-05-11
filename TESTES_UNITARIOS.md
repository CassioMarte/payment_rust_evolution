# Testes Unitarios:

Nossa Estratégia de Testes:
- Testes Unitários (Models): Validar se nossas regras de Newtype e DTOs estão barrando o que devem barrar. (Zero banco de dados).

- Testes de Unidade com Mocks (Services): Testar a lógica de negócio isolando o banco de dados através do Trait.

- Testes de Integração (Repository): Garantir que nossas queries SQL estão corretas batendo no Postgres real.

- Testes de Integração (Handlers/API): Garantir que o JSON entra, percorre todas as camadas e volta corretamente.


- Onde colocar os arquivos de testes?

 * no mesmo arquivo do model no final; 

 * um arquivo diferente na mesma pasta; 

 * ou em uma pasta específica de testes;

````
#[cfg(test)]
mod tests {
    // `use super::*` seria mais comum aqui
    // mas o autor preferiu caminhos completos — ambos funcionam
    use crate::models::client::{ClientName, ClientEmail, CreateClientDto, PlanType};
    use validator::Validate;

     // ... testes aqui
}
````


## test_client_name_validation

model:
````
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Validate)]
pub struct ClientName(
  #[validate(
     length(min = 3, max = 100, message = "Client name must be between 3 and 100 characters.")
    )]
  pub String,
);
````

teste:
````
#[test] // -> macro que registra esta função como teste sem ela o cargo test não funciona
fn test_client_name_validation(){

  // Teste c/ valor válido
  let name = ClientName("John Silva".to_string());

  // assert!(condição) -> se for false teste falha
  // name.validate()   -> chama o impl validate for ClientName
  // is_ok             -> verifica se está ok
  // Leitura: "afirmo que validar 'João Silva' é Ok"
  assert!(name.validate().is_ok());


  // "Jo" tem 2 caracteres -> viola min=3
  let short_name = ClientName("Jo".to_string());

   // .is_err() -> verifica se retornou Err(ValidationErrors)
   // Leitura: "afirmo que validar 'Jo' é um erro"
   // ⚠️ OBS: reutilizar a variável `name` — em Rust isso é shadowing e seria valido no teste let name = clientName("Jo".to_string())
   // a segunda `let name` "cobre" a primeira — ambas existem mas
   // só a mais recente é acessível
  assert!(short_name.validate().is_err());

  // Nome muito longo com 103 de length
  let very_long_name = ClientName("a".repeat(101));

  // Leitura: "afirmo que validar 101 caracteres é um erro"
  // ⚠️ OBS da mesma forma que o anterior poderia ser `let name`
  assert!(very_long_name.is_err());
}
````

- test_client_email_validation

model: 
````
pub struct ClientEmail(
  #[validate(email(message = "Email must be a velid email address"))]
  pub string
)
````

test:
````
fn test_client_email_validate(){

  let email = ClientEmail("john@gmail.com".to_string());

  // #[validate(email)] verifica formato RFC
  // tem @ -> tem domínio -> tem extensão -> válido 
  assert!(email.validate().is_ok());

  // usei shadowing citado em let name
  let email = ClientEmail("john_com".to_string());

  // sem @ -> inválido
  // o validator checa o formato completo
  assert!(email.validate().is_err());

  let email = ClientName("".to_string);

  // string vazia não tem formato de email -> inválido
  assert!(email.validate().is_err());
}
````

- Segue o mesmo padrão em address, 

- test_create_client_dto_default_plan

model:
````
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Validate)]
pub struct CreateClientDto{
   #[validate]
   pub name: ClientName,
   #[validate]
   pub email: ClientEmail,
   #[validate]
   pub address: ClientAddress,
   #[validate]
   pub plan: PlanType,
}
````

test ok:
````
#[test]
pub test_create_client_dto_default_plan() {
  let dto = CreateClientDto{
    name: ClientName("Cassio".to_string()),
    email: ClientEmail("test@test.com".to_string()),
    address: ClientAddress("Address, 1007".to_string),
    plan: PlanType::Mensal

    // se eu tiver configurado valor default
    plan: PlanType::default(),
    // Chama o impl Default for PlanType
    // O teste AFIRMA que o default é Mensal
  }
   
   // se eu tiver configurado valor default
   // Verifica se o default é realmente Mensal
    assert_eq!(dto.plan, PlanType::Mensal);
   // assert_eq!(a, b) -> verifica se a == b
   // se não forem iguais -> teste falha com mensagem:
   // "left: Mensal, right: Mensal"

   // Verifica se o DTO inteiro é válido
   assert!(dto.validate().is_ok());
   // Valida em cascata primeiiro existe ClientName dentro do dto e se sim é valido depois existe ClientEmail dentro do dto  se sim e valido, ...

}
````

test error:
````
#[test]
fn test_create_client_dto_cascade_validation() {
    // Este é o teste MAIS IMPORTANTE dos quatro
    // Testa se a validação do DTO "enxerga" erros dentro dos Value Objects

    let dto = CreateClientDto {
        name: ClientName("Jo".to_string()),
        // INVÁLIDO: 2 chars, min=3
        // O Rust permite criar ClientName("Jo") sem erro!
        // A validação só acontece ao chamar .validate()
        // Isso é importante — o tipo não valida no construtor

        email: ClientEmail("valido@teste.com".to_string()),
        // válido

        address: crate::models::client::ClientAddress("Endereço válido".to_string()),
        // válido

        plan: PlanType::Anual,
        // válido
    };

    // A pergunta do teste:
    // "Se name é inválido, o dto.validate() vai pegar isso?"
    assert!(dto.validate().is_err());
    // Isso funciona por causa do #[validate] no campo:
    //
    // pub struct CreateClientDto {
    //     #[validate]           "vai lá dentro do ClientName e valida"
    //     pub name: ClientName, se ClientName for inválido -> dto inválido
    // }
    //
    // É a validação em CASCATA:
    // dto.validate()
    //   -> entra no ClientName
    //     -> chama ClientName.validate()
    //       -> "Jo" tem 2 chars, min=3 -> ERRO
    //   -> dto retorna Err com os erros de ClientName
}
````
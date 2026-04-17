# Model e Validação 

Primeiro vamos compreender algumas logicas que são importantes para construirmos os model:

- struct -> molde de dados

```
pub struct ClientName(pub String); // tuple struct com 1 campo
```

- impl -> adiciona comportamentos a este molde

1- impl NomeDaStruct (ex: ClientName)
        └── "Estou definindo métodos que só existem em ClienteName
              Você mesmo está inventando esses métodos.
```
// COMPORTAMENTO: define o que a struct SABE FAZER
impl ClientName {
    pub fn new(value: String) -> Self {
        ClientName(value)
    }
    
    pub fn len(&self) -> usize {
        self.0.len() // self.0 = primeiro campo da tuple struct
    }
}
```

2- impl AlgumaTrait for NomeDaStruct (ex: impl From<String> for ClientName)
            └── "Estou cumprindo um CONTRATO que já existe no Rust/biblioteca"
                 O From, Display, etc. já existem você só esta dizendo 
                 como sua struct se comporta dentro deste contrato

````
impl From<String> for ClientName{
//   ^^^^^^^^^^^^      ^^^^^^^^^^^^^
//   TRAIT              TIPO que recebe a trait
  fn from(value: String){
    ClientName(value)
  }
}

````

Quando você usa a struct (ex: ClientName), 
você tem acesso aos campos (dados) + todos os métodos definidos nos blocos "impl".


- for -> significa implemente um trait (contrato, caracteristica) para um tipo 

```
impl [O QUE VOCÊ QUER ENSINAR] for [QUEM VAI APRENDER]

impl fmt::Display for PlanType
//   ^^^^^^^^^^^^      ^^^^^^^^
//   trait Display      quem implementa

```

### Legenda do Model usado no projeto 

```
#[derive(
    Debug,          // permite imprimir com {:?} no log/terminal
                    //   sem Debug: println!("{:?}", client) → ERRO
                    //   com Debug: println!("{:?}", client) → Client { id: ..., name: ... } 
    
    Clone,          // permite copiar o valor com .clone()
                    // sem clone: let b = a → move (a não existe mais)
                    // com clone: let b = a.clone() a e b existem e tem o valor

    PartialEq,      // permite comparar com == e !=
                    // sem PartialEq: if client_a == client_b → ERRO
                    // com PartialEq: if client_a == client_b → funciona  

    Eq,             // versão completa do PartialEq
                    //   PartialEq: "pode ser igual" (floats por ex não são sempre iguais)
                    //   Eq: "sempre tem resposta definida para igualdade"
                    //   use Eq quando todos os campos também são Eq

    Serialize,      // Rust -> JSON (para Enviar dados)
                    // Client { name: "João" } vira -> { "name": "João" }


    Deserialize,    // JSON -> Rust (para receber dados)
                    // { "name": "João" } vira -> Client { name: "João" }

            
    Validate,       // habilita .validate() na struct
                    // permite usar #[validate(...)] nos campos
                    // sem Validate: .validate() da erro de compilação

    sqlx::Type      // ensina o SQLx como ler/escrever este tipo no banco
                    // necessário para enums que existem no PostgreSQL
                    // ex: PlanType -> "mensal", "anual" no banco


    sqlx::FromRow,  // converte uma linha do banco → struct Rust
                    // sem: extração manual campo por campo
                    // com: SQLx converte automaticamente        
    
)]
```


````
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{NaiveDateTime, Utc};
use validator::Validate;
use std::fmt;


// Encapsula a validação dentro do Tipo
// ClientName garante que o nome sempre seja valido

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Validate)]
pub struct ClientName(
  //validate no campo da tuple struct
  #[validate(length(
    min = 3,
    max = 100,
    message = "Client name must be between 3 and 100 characters"
  ))]
  pub String,  // o único campo — o nome em si
)

// From<String> → permite converter String → ClientName automaticamente
// "João".to_string().into() → ClientName("João")
impl From<String> for ClientName{
  fn from(value: String) -> Self{
    ClientName(value)
  }
}

// Display define como aparece quando impresso com {}
impl fmt::Display for ClientName {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0) // self.0 acessa o String interno
  }
}

/ ClientEmail → garante que o email SEMPRE tem formato válido
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Validate)]
pub struct ClientEmail(
    #[validate(email(message = "Email must be a valid email address"))]
    pub String,
);

impl From<String> for ClientEmail {
    fn from(email: String) -> Self {
        ClientEmail(email)
    }
}

impl fmt::Display for ClientEmail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Validate)]
pub struct ClientAddress( // ← corrigido
    #[validate(length(
        min = 5,
        max = 200,
        message = "Client address must be between 5 and 200 characters."
    ))]
    pub String,
);

impl From<String> for ClientAddress {
    fn from(address: String) -> Self {
        ClientAddress(address)
    }
}

impl fmt::Display for ClientAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}


// Enum

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type )]
// sqlx::Type -> ensina o SQLx a ler/escrever PlanType no banco
// type_name = nome do tipo no PostgreSQL
// rename_all = converte PascalCase → lowercase automaticamente
// Diaria → "diaria" no banco
#[sqlx(type_name = "plan_type", rename_all = "lowercase")]
pub enum PlanType {
    Diaria,
    Mensal,
    Trimestral,
    Semestral,
    Anual,
}

// Display → como aparece como texto
impl fmt::Display for PlanType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlanType::Diaria      => write!(f, "diaria"),
            PlanType::Mensal      => write!(f, "mensal"),
            PlanType::Trimestral  => write!(f, "trimestral"),
            PlanType::Semestral   => write!(f, "semestral"),
            PlanType::Anual       => write!(f, "anual"),
        }
    }
}


// client_model

// sqlx::FromRow -> SQLx converte linha do banco -> Client automaticamente
// NÃO tem Validate — Client é o dado que VEM do banco
// dados do banco já foram validados quando foram inseridos
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
pub struct Client {
    pub id: Uuid,                  // gerado pelo banco
    pub name: String,              // String simples — dado do banco
    pub email: String,             // String simples — dado do banco
    pub address: String,           // String simples — dado do banco
    pub plan: PlanType,            // enum — mapeado pelo sqlx::Type
    pub created_at: NaiveDateTime, // gerado pelo banco
    pub updated_at: NaiveDateTime, // gerado pelo banco
}

// model de dados que vem do cliente para cria 

// sqlx::FromRow -> busca do banco depois de criar com RETURNING
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Validate, sqlx::FromRow)]
pub struct CreateClientDto {
    #[validate] // ← delega a validação para ClientName (já tem regras dentro)
    pub name: ClientName,

    #[validate] // ← delega para ClientEmail
    pub email: ClientEmail,

    #[validate] // ← delega para ClientAddress
    pub address: ClientAddress,

    pub plan: PlanType, // ← enum fechado — sem validação extra necessária
}


// model de dados que vem do cliente para atualização
// Todos os campos são Option — cliente envia só o que quer mudar

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Validate)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Validate)]
pub struct UpdateClientDto {
    // Option<String> → cliente pode ou não enviar o nome
    #[validate(length(
        min = 3,
        max = 100,
        message = "Nome deve ter entre 3 e 100 caracteres"
    ))]
    pub name: Option<String>, // Some("João") = enviou | None = não enviou

    #[validate(email(message = "Email inválido"))]
    pub email: Option<String>,

    #[validate(length(
        min = 5,
        max = 200,
        message = "Endereço deve ter entre 5 e 200 caracteres"
    ))]
    pub address: Option<String>,

    pub plan: Option<PlanType>, // None = não quer mudar o plano
}
````


## Opção de model com valores default


- Ex: na regra de negócio caso na criação do client não seja enviado plano ele deve ser criado como Experimental

Opção 1 - com fn mais simples mas o defalt pode dar problemas em testes
````
#[sqlx(type_name = "plan_type", rename_all = "lowercase")]
pub enum PlanType {
    Experimental,
    Diaria,
    Mensal,
    Trimestral,
    Semestral,
    Anual,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Validate, sqlx::FromRow)]
pub struct CreateClientDto {
    #[validate] // ← delega a validação para ClientName (já tem regras dentro)
    pub name: ClientName,

    #[validate] // ← delega para ClientEmail
    pub email: ClientEmail,

    #[validate] // ← delega para ClientAddress
    pub address: ClientAddress,

    #[serde(default = "default_plan")]
    pub plan: PlanType, // ← enum fechado — sem validação extra necessária
}

fn defalt_plan() -> PlanType {
   PlanType::Experimental
}
````

Opção 2- usando impl e o default é criado no enum

````
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "plan_type", rename_all = "lowercase")]
pub enum PlanType {
    Experimental,
    Diaria,
    Mensal,
    Trimestral,
    Semestral,
    Anual,
}

// 1. Implementamos o Default para o Enum (O jeito Rust de ser)
impl Default for PlanType {
    fn default() -> Self {
        PlanType::Experimental
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateClientDto {
    #[validate]
    pub name: ClientName,
    #[validate]
    pub email: ClientEmail,
    #[validate]
    pub address: ClientAddress,

    // 2. A MÁGICA: O Serde agora usa o Default::default() do próprio Enum!
    #[serde(default)] 
    pub plan: PlanType,
}

````


Resumo:
┌─────────────────────────────────────────────────────┐
│  struct ClientName(pub String)                       │
│       └── define o DADO que a struct carrega         │
├─────────────────────────────────────────────────────┤
│  impl ClientName { ... }                             │
│       └── métodos PRÓPRIOS da struct                 │
├─────────────────────────────────────────────────────┤
│  impl From<String> for ClientName { ... }            │
│       └── implementa uma TRAIT (contrato externo)    │
│           "ensina" a struct a se converter de String │
├─────────────────────────────────────────────────────┤
│  impl Display for ClientName { ... }                 │
│       └── "ensina" a struct a se exibir com {}       │
└─────────────────────────────────────────────────────┘
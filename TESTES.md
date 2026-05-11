# Base de testes

Nossa Estratégia de Testes:
- Testes Unitários (Models): Validar se nossas regras de Newtype e DTOs estão barrando o que devem barrar. (Zero banco de dados).

- Testes de Unidade com Mocks (Services): Testar a lógica de negócio isolando o banco de dados através do Trait.

- Testes de Integração (Repository): Garantir que nossas queries SQL estão corretas batendo no Postgres real.

- Testes de Integração (Handlers/API): Garantir que o JSON entra, percorre todas as camadas e volta corretamente.


````
// assert!(condição)
// → se TRUE  → teste passa silenciosamente
// → se FALSE → teste falha com:
//   "thread 'tests::test_name' panicked at 'assertion failed: condição'"

assert!(name.validate().is_ok());   // afirma que é Ok
assert!(name.validate().is_err());  // afirma que é Err


// assert_eq!(a, b)
// → se IGUAIS    → passa silenciosamente
// → se DIFERENTES → falha com:
//   "left: Mensal
//    right: Experimental"
// Muito mais útil que assert! quando valores importam

assert_eq!(dto.plan, PlanType::Experimental);


// assert_ne!(a, b) — não usado aqui mas existe
// → se DIFERENTES → passa
// → se IGUAIS     → falha
````



## Roda TODOS os testes
cargo test

## Roda só os testes deste módulo
cargo test tests::

## Roda um teste específico
cargo test test_client_name_validation

## Mostra output mesmo em testes que passam
cargo test -- --nocapture

## Roda testes em paralelo (padrão) ou sequencial
cargo test -- --test-threads=1
# Testes Integração Handles:

Nossa Estratégia de Testes:
- Testes Unitários (Models): Validar se nossas regras de Newtype e DTOs estão barrando o que devem barrar. (Zero banco de dados).

- Testes de Unidade com Mocks (Services): Testar a lógica de negócio isolando o banco de dados através do Trait.

- Testes de Integração (Repository): Garantir que nossas queries SQL estão corretas batendo no Postgres real.

- Testes de Integração (Handlers/API): Garantir que o JSON entra, percorre todas as camadas e volta corretamente.
# RELATO

## Envcheck - desenvolvimento orientado por especificação com Spec Kit

**Contexto e objetivo.** O projeto Envcheck foi desenvolvido como um CLI em Rust para validar arquivos `.env` de forma determinística, local e somente leitura. A aplicação compara um `.env` com um `.env.example` ou com um `.env.schema` TOML tipado. O schema suporta `string`, `integer`, `float` e `boolean`, além de `required`, `allow_empty`, `min`, `max`, `min_length`, `max_length`, `allowed` e `pattern`. A descoberta automática procura primeiro `.env.schema` e depois `.env.example` no diretório do arquivo alvo. Os diagnósticos não exibem os valores reais do `.env`.

## Fluxo utilizado no Spec Kit

1. **Instalação / inicialização** - o Spec Kit foi inicializado no repositório e integrado ao agente de código, criando a estrutura `.specify/` e as skills em `.agents/skills/`.
2. **`$speckit-constitution`** - definiu os princípios do projeto: correção antes de conveniência, estabilidade do CLI, separação de responsabilidades, testes, erros acionáveis, segurança de segredos, dependências mínimas, portabilidade, operação local/in-memory e quality gates Rust.
3. **`$speckit-specify`** - transformou a ideia em requisitos funcionais e critérios mensuráveis para comparação por `.env.example`, validação por `.env.schema`, discovery, diagnósticos e códigos de saída.
4. **`$speckit-clarify`** - removeu ambiguidades sobre identificadores, comentários inline, valores vazios, tipos numéricos, regex, `allowed`, duplicatas, descoberta e redaction.
5. **`$speckit-plan`** - definiu arquitetura e tecnologia: Rust stable, `clap`, `serde`/`toml`, `regex` e `dotenvx-primitives::scan`, com parser, schema, validação e output desacoplados.
6. **`$speckit-checklist`** - revisou a qualidade dos requisitos depois do plan. Ao final, `requirements.md` ficou 16/16 e `validation-contract.md` 48/48 revisados.
7. **`$speckit-tasks`** - quebrou o trabalho em tarefas dependentes e incrementais, com fases RED/GREEN para cada comportamento.
8. **`$speckit-analyze`** - fez uma análise cruzada de spec, plan, tasks, constituição e código antes da implementação, localizando inconsistências.
9. **`$speckit-implement`** - implementou as tarefas em ciclos curtos: primeiro testes falhando, depois comportamento mínimo para deixá-los verdes; ao final, todos os gates foram executados novamente.
10. **`$speckit-converge`** - converteu os gaps encontrados em T057/T058; depois, `$speckit-implement` corrigiu o gate de release e a comparação exata entre `f64` e constraints TOML integer acima de `2^53`.

## Estrutura resultante

```text
src/
  cli.rs               argumentos do CLI
  env/                 leitura e parsing dotenv
  schema/              modelo + parsing TOML
  validation/          regras, diagnósticos e orquestração
  output/              renderização stdout/stderr
tests/cli.rs            testes do binário compilado
specs/001-env-validation/
  spec.md, plan.md, tasks.md, quickstart.md
  contracts/, checklists/, research.md, data-model.md
examples/                9 cenários executáveis
.github/workflows/
  ci.yml                 Linux + macOS + Windows
  release.yml            build/release após CI verde na main
```

## Exemplos de uso

```bash
# Comparação explícita com .env.example
envcheck .env --example .env.example

# Validação tipada explícita
envcheck .env --schema .env.schema

# Discovery automático ao lado do alvo
envcheck config/.env
```

No discovery, `.env.schema` tem precedência sobre `.env.example`. Os principais códigos de saída são: `0` válido ou apenas warnings; `1` erros de validação; `2` uso incorreto do CLI; `3` falha de arquivo/discovery/dotenv; `4` schema inválido. A pasta `examples/` inclui casos simples, complexos, apenas example, apenas schema, ambos, override explícito, schema inválido e ausência de definição.

## Experiência com o processo

Na prática, o fluxo funcionou melhor quando os artefatos foram tratados como parte do trabalho, e não só como documentação. O **Clarify** ajudou a fechar dúvidas ainda na especificação, antes de elas virarem decisões de implementação. Depois do **Plan**, o **Checklist** serviu para conferir se os requisitos e o desenho continuavam coerentes antes de quebrar o trabalho em tarefas. O **Analyze**, usado antes do **Implement**, funcionou como uma revisão cruzada entre spec, plan, tasks e constituição e mostrou pontos que ainda estavam soltos. Depois da implementação, o **Converge** transformou os gaps restantes em tarefas objetivas, que voltaram para o mesmo ciclo de código e testes. Esse vai e volta deixou as etapas mais consistentes entre si e evitou que decisões importantes ficassem só no código ou fossem resolvidas tarde demais.

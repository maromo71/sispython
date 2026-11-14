# AGENTS.md — Instruções Mestras do LogicKiosk

Este projeto segue rigorosamente a especificação técnica definida em `SPEC.md` (Spec-Driven Development).

## Fronteira Imutável de Contrato IPC
- Back-end Rust: `src-tauri/src/ipc/commands.rs`
- Front-end TypeScript: `src/types/ipc.ts`

Qualquer alteração em estruturas serializadas entre Tauri e React deve manter simetria estrita entre esses dois arquivos.

## Etapas de Arquitetura
1. **Etapa 1:** Fundação Tauri v2, Schema SQLite (`src-tauri/migrations/0001_initial_schema.sql`) e Semente Pedagógica do Módulo 1 (5 fases).
2. **Etapa 2:** Sandbox segura (`src-tauri/src/sandbox/`) com parser AST (`rustpython-parser`), executor isolado com timeout de 2000ms e os 3 agentes de regras de negócio (`judge.rs`, `socratic.rs`, `offline_rules.rs`, `game_master.rs`).
3. **Etapa 3:** Interface Dual Theme (`dark-tech` / `light-tech`) em `src/styles/themes.css`, Quiosque de 3 painéis (`SplitKioskView`), `TopHudBar`, `MissionPanel`, `CodeEditorPane` (CodeMirror 6), `ExecutionConsole` e `VariableWatch`.
4. **Etapa 4:** Integração ponta a ponta com IPC Tauri, `SkillTreeGraph` (Hard Gatekeeping + Boss Level), configuração BYOK segura via OS Keyring e `WalletExporter` com assinatura SHA-256.

## Regras Invioláveis
- O `SocraticTutorAgent` nunca deve retornar blocos de código de solução pronta (sanitização determinística obrigatória em Rust).
- Fases `BOSS` bloqueiam pedidos proativos de pistas socráticas.
- Credenciais BYOK nunca são gravadas no SQLite ou `localStorage`; usar exclusivamente `keyring`.
- Suporte completo a atalhos de teclado: `Ctrl+Enter` (executar), `Alt+H` (pista socrática), `Alt+T` (alternar tema), `Ctrl+Shift+Enter` (avançar fase).

# LogicKiosk

Plataforma gamificada e **100% Open Source** para ensino de **lógica de programação e algoritmos em Python**, construída com **Tauri v2**, **Rust**, **React 19**, **TypeScript**, **Tailwind CSS v4**, **CodeMirror 6** e **SQLite** (local-first).

> **Licença Aberta e Livre (MIT):** Este projeto é totalmente open source sob a [Licença MIT](./LICENSE). Você tem liberdade total para usar, estudar, copiar, modificar, adicionar novos recursos, redistribuir, criar produtos derivados e **comercializar / lucrar** sem restrições ou royalties.

---

## Sumário

- [Visão Geral e Recursos](#visão-geral-e-recursos)
- [Downloads e Distribuições](#downloads-e-distribuições)
- [Como Rodar na Web](#como-rodar-na-web)
- [Desenvolvimento Local (Desktop / Tauri)](#desenvolvimento-local-desktop--tauri)
- [Builds Multiplataforma (Windows, macOS, Android e iOS)](#builds-multiplataforma-windows-macos-android-e-ios)
- [Arquitetura e Regras Pedagógicas](#arquitetura-e-regras-pedagógicas)
- [Atalhos de Teclado](#atalhos-de-teclado)
- [Variáveis de Ambiente e IA (BYOK)](#variáveis-de-ambiente-e-ia-byok)
- [Testes e Qualidade (QA)](#testes-e-qualidade-qa)
- [Estrutura do Projeto](#estrutura-do-projeto)
- [Contribuição e Uso Comercial](#contribuição-e-uso-comercial)

---

## Visão Geral e Recursos

- **Quiosque de 3 Painéis (`SplitKioskView`):** enunciado da missão (`MissionPanel`), editor Python com syntax highlight e realce de linha (`CodeEditorPane` via CodeMirror 6) e console de execução + inspeção de variáveis (`ExecutionConsole`, `VariableWatch`).
- **Sandbox Python em Duas Etapas (`src-tauri/src/sandbox/`):**
  - Inspeção estática de AST com `rustpython-parser` e `rustpython-ast`, bloqueando chamadas/imports inseguros e validando estruturas obrigatórias da fase.
  - Execução isolada com timeout padrão de `2000ms`, limite de memória (`64 MB`) e limite de saída (`10000` caracteres).
- **Motor Pedagógico Multiagente (`src-tauri/src/agents/`):**
  - `JudgeAgent`: executa casos de teste públicos e casos de borda ocultos.
  - `SocraticTutorAgent`: gera perguntas socráticas guiadas sem entregar respostas prontas (com sanitizador determinístico em Rust que remove blocos de código de solução; pedidos proativos de pista ficam bloqueados em fases `BOSS`).
  - `OfflineRulesEngine`: feedback pedagógico offline determinístico quando não há IA externa configurada.
  - `GameMasterAgent`: XP, sequência diária (*streak* com detecção de divergência de relógio), insígnias e *Hard Gatekeeping*.
- **Árvore de Habilidades (`SkillTreeGraph`):** progressão bloqueada por maestria até o desafio final (`BOSS`) do módulo.
- **Carteira Acadêmica Assinada (`WalletExporter`):** exportação e verificação de certificados JSON com assinatura **SHA-256**.
- **Tema Dual (`dark-tech` / `light-tech`):** alternância instantânea persistida por perfil.
- **Privacidade & BYOK Segura:** chaves de API opcionais (Google Gemini Free Tier / BYOK) são gravadas exclusivamente no **Keyring / Keychain nativo do Sistema Operacional** (Windows Credential Manager, macOS/iOS Keychain ou Linux Secret Service) — nunca no SQLite ou `localStorage`.

---

## Downloads e Distribuições

Todos os instaladores e pacotes oficiais gerados pelo pipeline de CI/CD ficam disponíveis diretamente no GitHub:

- **Releases Oficiais (Instaladores prontos por versão `v*`):**
  https://github.com/maromo71/sispython/releases
- **Última Release Publicada:**
  https://github.com/maromo71/sispython/releases/latest
- **Builds Contínuos (GitHub Actions — Artefatos de cada commit na `main`):**
  https://github.com/maromo71/sispython/actions/workflows/build-all-platforms.yml

| Plataforma | Runner CI | Artefato / Formato | Como obter |
| --- | --- | --- | --- |
| **Windows (`x86_64`)** | `windows-latest` | Instaladores `.msi` e `.exe` (NSIS) | [Releases](https://github.com/maromo71/sispython/releases) / artefato `logickiosk-windows` no [Actions](https://github.com/maromo71/sispython/actions/workflows/build-all-platforms.yml) |
| **macOS (Universal)** | `macos-latest` | `.dmg` Universal (Apple Silicon + Intel) e `LogicKiosk-macos-universal.app.tar.gz` | [Releases](https://github.com/maromo71/sispython/releases) / artefato `logickiosk-macos` no [Actions](https://github.com/maromo71/sispython/actions/workflows/build-all-platforms.yml) |
| **Android / Android Studio** | `ubuntu-latest` | `.apk`, `.aab` e projeto completo `LogicKiosk-android-studio-project.tar.gz` | [Releases](https://github.com/maromo71/sispython/releases) / artefato `logickiosk-android` no [Actions](https://github.com/maromo71/sispython/actions/workflows/build-all-platforms.yml) |
| **iOS / Xcode** | `macos-latest` | `LogicKiosk-ios-xcode-project.tar.gz` + `.ipa` (com assinatura Apple) ou `LogicKiosk-ios-unsigned-archive.tar.gz` (sem assinatura) | [Releases](https://github.com/maromo71/sispython/releases) / artefato `logickiosk-ios` no [Actions](https://github.com/maromo71/sispython/actions/workflows/build-all-platforms.yml) |
| **Web (Front-end SPA)** | Qualquer servidor estático / Vite | Bundle estático em `dist/` | Gerado localmente com `npm run build` (detalhes abaixo) |

---

## Como Rodar na Web

O front-end do **LogicKiosk** é uma aplicação **React 19 + Vite** (`vite.config.ts`, porta padrão `1420`).

### 1. Servidor de desenvolvimento Web

```bash
git clone https://github.com/maromo71/sispython.git
cd sispython
npm install
npm run dev
```

Acesse no navegador:

- **URL local:** `http://localhost:1420`

### 2. Build de produção para hospedagem Web (`dist/`)

Para gerar os arquivos estáticos prontos para **GitHub Pages**, **Vercel**, **Netlify**, **Cloudflare Pages**, **Nginx** ou qualquer servidor HTTP:

```bash
npm install
npm run build
```

Para testar o build de produção localmente antes do deploy:

```bash
npm run preview
```

Os arquivos finais da aplicação Web ficam na pasta `dist/` (configurada em `frontendDist: "../dist"` no [src-tauri/tauri.conf.json](./src-tauri/tauri.conf.json)).

### Importante sobre o Modo Web Puro vs. Runtime Nativo (Tauri)

> **Nota técnica:** Na arquitetura atual do repositório, a inspeção de AST (`rustpython-parser`), o executor sandbox Python, o banco SQLite local (`logickiosk_state.sqlite`), o Keyring do sistema e a assinatura SHA-256 residem no núcleo Rust ([src-tauri/](./src-tauri)).
> - Quando aberto **diretamente em um navegador Web comum** (`isTauriRuntime() === false` em [src/services/ipcClient.ts](./src/services/ipcClient.ts)), a interface React carrega o layout, temas e shell de navegação normalmente sem falhar, mas não há backend IPC conectado para compilar/executar códigos Python ou persistir o currículo no SQLite.
> - Para rodar **todas as funcionalidades ponta a ponta** (sandbox Python + tutor socrático + banco SQLite), execute via `npm run tauri dev` ou instale um dos binários nativos da seção [Downloads e Distribuições](#downloads-e-distribuições).
> - Se quiser publicar uma versão 100% autônoma no navegador (por exemplo, integrando Pyodide/WASM ou uma API HTTP no lugar de `invoke()`), basta adaptar a camada [src/services/ipcClient.ts](./src/services/ipcClient.ts) — contribuições e forks com backend Web/WASM são bem-vindos!

---

## Desenvolvimento Local (Desktop / Tauri)

### Pré-requisitos

- **Node.js** `22+` e **npm**
- **Rust** (`rustup` / `cargo` estável)
- **Dependências do Tauri v2** para seu sistema operacional:
  - **Windows:** Microsoft C++ Build Tools + WebView2 (já presente no Windows 10/11)
  - **macOS:** Xcode Command Line Tools (`xcode-select --install`)
  - **Linux (Ubuntu/Debian):** `libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev libdbus-1-dev pkg-config`
- **Python 3** disponível no sistema para execução isolada da sandbox (ou configuração de ambiente equivalente)

### Comandos

```bash
# 1. Instalar dependências JavaScript
npm install

# 2. Subir o aplicativo completo (Front-end Vite + Back-end Rust/SQLite)
npm run tauri dev

# 3. Gerar build instalável para o seu sistema operacional
npm run tauri build
```

---

## Builds Multiplataforma (Windows, macOS, Android e iOS)

O projeto já possui pipeline automatizado em [.github/workflows/build-all-platforms.yml](./.github/workflows/build-all-platforms.yml) e documentação detalhada em [BUILD_CI.md](./BUILD_CI.md).

### Disparo via GitHub Actions

- **Push / Pull Request em `main` ou `master`:** executa `qa-checks` (`npm test`, `npm run build`, `cargo test`) e compila os artefatos de Windows, macOS, Android Studio e iOS.
- **Nova Release com tag (`v*`):**
  ```bash
  git tag v0.1.0
  git push origin v0.1.0
  ```
  Anexa automaticamente todos os pacotes gerados à GitHub Release.
- **Disparo manual:** aba **Actions → Build Multiplatform Releases (Windows, macOS, Android Studio, iOS) → Run workflow**.

### Secrets opcionais para assinatura iOS

Sem secrets configurados, o CI gera automaticamente o projeto Xcode (`LogicKiosk-ios-xcode-project.tar.gz`) e o `.xcarchive` não assinado (`LogicKiosk-ios-unsigned-archive.tar.gz`). Para gerar `.ipa` assinado no CI, cadastre em **Settings → Secrets and variables → Actions**:

- `APPLE_DEVELOPMENT_TEAM`
- `IOS_CERTIFICATE_BASE64`
- `IOS_CERTIFICATE_PASSWORD`
- `IOS_PROVISIONING_PROFILE_BASE64`

---

## Arquitetura e Regras Pedagógicas

O projeto segue especificação orientada em [SPEC.md](./SPEC.md) e regras arquiteturais em [AGENTS.md](./AGENTS.md):

1. **Contrato IPC Simétrico:** estruturas serializadas entre Rust e TypeScript mantêm espelhamento estrito entre [src-tauri/src/ipc/commands.rs](./src-tauri/src/ipc/commands.rs) e [src/types/ipc.ts](./src/types/ipc.ts).
2. **Privacidade do Tutor Socrático:** o `SocraticTutorAgent` passa por sanitização determinística em Rust para nunca retornar blocos de código com a solução pronta.
3. **Proteção de Avaliação (`BOSS`):** fases do tipo `BOSS` bloqueiam solicitações proativas de pistas socráticas.
4. **Segurança de Credenciais:** chaves de API BYOK utilizam exclusivamente `keyring` do sistema operacional.

---

## Atalhos de Teclado

| Atalho | Ação |
| --- | --- |
| `Ctrl + Enter` | Executar / Submeter código na sandbox |
| `Alt + H` | Solicitar pista socrática (bloqueado em fases `BOSS`) |
| `Alt + T` | Alternar tema (`dark-tech` / `light-tech`) |
| `Ctrl + Shift + Enter` | Avançar para a próxima fase desbloqueada |

---

## Variáveis de Ambiente e IA (BYOK)

Um modelo completo de configuração está disponível em [.env.example](./.env.example), incluindo:

- Banco SQLite local (`LOGICKIOSK_DB_FILENAME=logickiosk_state.sqlite`)
- Provedor Google AI Studio Free Tier (`GEMINI_API_BASE_URL`, `GEMINI_PRIMARY_FREE_MODEL`, `GEMINI_FALLBACK_FREE_MODEL`, `GEMINI_OPEN_FREE_MODEL`)
- Fallback local opcional via **Ollama** (`LOCAL_LLM_ENABLED`, `LOCAL_LLM_ENDPOINT`, `LOCAL_LLM_MODEL`)
- Limites da sandbox (`SANDBOX_MAX_EXECUTION_TIME_MS=2000`, `SANDBOX_MAX_MEMORY_MB=64`, `SANDBOX_MAX_OUTPUT_CHARS=10000`)

---

## Testes e Qualidade (QA)

```bash
# Testes de interface e stores (Vitest + Testing Library + jsdom)
npm test

# Verificação de tipos TypeScript e bundle Vite
npm run build

# Testes unitários e de integração do núcleo Rust (AST, Sandbox, Agentes e SQLite)
cargo test --manifest-path src-tauri/Cargo.toml
```

---

## Estrutura do Projeto

```text
sispython/
├── .github/workflows/
│   └── build-all-platforms.yml   # CI/CD: QA + Windows, macOS, Android e iOS
├── src/                          # Front-end React 19 + TypeScript + Tailwind v4
│   ├── components/
│   │   ├── academic/             # WalletExporter (certificados SHA-256)
│   │   ├── kiosk/                # MissionPanel, CodeEditorPane, ExecutionConsole, VariableWatch
│   │   ├── layout/               # TopHudBar, SplitKioskView
│   │   ├── map/                  # SkillTreeGraph (Hard Gatekeeping)
│   │   └── settings/             # ApiKeyModal (BYOK)
│   ├── services/ipcClient.ts     # Ponte IPC Tauri + detecção de runtime
│   ├── store/                    # Zustand stores (tema, quiosque, progressão)
│   ├── styles/themes.css         # Temas dark-tech / light-tech
│   └── types/ipc.ts              # Contratos TypeScript espelhados do Rust
├── src-tauri/                    # Núcleo nativo Tauri v2 + Rust
│   ├── migrations/               # 0001_initial_schema.sql + semente pedagógica
│   ├── src/agents/               # judge, socratic, offline_rules, game_master
│   ├── src/db/                   # Repositório SQLite (sqlx)
│   ├── src/ipc/                  # Comandos Tauri IPC
│   ├── src/sandbox/              # Parser AST + runner isolado
│   ├── tests/                    # Testes de integração
│   ├── Cargo.toml
│   └── tauri.conf.json
├── .env.example
├── AGENTS.md
├── BUILD_CI.md
├── LICENSE                       # Licença MIT (uso livre e comercial permitido)
├── README.md
└── SPEC.md                       # Especificação técnica completa
```

---

## Contribuição e Uso Comercial

Este software é publicado sob a **[Licença MIT](./LICENSE)**:

- **Pode usar** para fins pessoais, educacionais, institucionais ou empresariais.
- **Pode modificar e estender** (adicionar módulos, linguagens, suporte a WebAssembly/Pyodide, novos tutores de IA, integração LMS/SCORM/LTI etc.).
- **Pode redistribuir, sublicenciar e lucrar** (vender licenças, oferecer como SaaS, empacotar em soluções educacionais pagas ou consultorias), mantendo apenas o aviso de copyright e a licença MIT nos arquivos derivados.

Pull requests, issues e forks são muito bem-vindos em:
https://github.com/maromo71/sispython

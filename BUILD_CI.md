# Guia de Builds Multiplataforma via GitHub Actions

O workflow [build-all-platforms.yml](file:///c:/sispython/.github/workflows/build-all-platforms.yml) gera e faz o upload automático dos artefatos para **Windows**, **macOS**, **Android Studio** e **iOS** após validar todos os testes (`npm test` + `cargo test`).

## Artefatos Gerados por Plataforma

| Plataforma | Runner | Artefatos Enviados (`upload-artifact`) |
| --- | --- | --- |
| **Windows** | `windows-latest` | Instaladores `.msi` e `.exe` (NSIS) |
| **macOS** | `macos-latest` | `.dmg` Universal (Apple Silicon + Intel) e `LogicKiosk-macos-universal.app.tar.gz` |
| **Android Studio** | `ubuntu-latest` | `.apk`, `.aab` e projeto completo do Android Studio (`LogicKiosk-android-studio-project.tar.gz`) |
| **iOS** | `macos-latest` | Projeto Xcode (`LogicKiosk-ios-xcode-project.tar.gz`) e `.ipa` (com assinatura) ou `.app` sem assinatura (`LogicKiosk-ios-unsigned-app.tar.gz`) |

## Como Disparar os Builds no GitHub

1. Inicialize o repositório Git (se ainda não estiver versionado) e envie para o GitHub:
   ```bash
   git init
   git add .
   git commit -m "feat: LogicKiosk multiplatform + GitHub Actions CI"
   git remote add origin <URL_DO_SEU_REPOSITORIO>
   git push -u origin main
   ```
2. Para publicar uma **Release** no GitHub com todos os instaladores anexados automaticamente:
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```
3. Também é possível disparar manualmente pela aba **Actions → Build Multiplatform Releases → Run workflow**.

## Assinatura iOS Opcional (Secrets do GitHub)

Sem secrets configurados, o workflow compila o projeto Xcode e gera o bundle `.app` sem assinatura para validação e abertura direta no Xcode. Para gerar um `.ipa` assinado automaticamente, configure em **Settings → Secrets and variables → Actions**:
- `APPLE_DEVELOPMENT_TEAM`
- `IOS_CERTIFICATE_BASE64`
- `IOS_CERTIFICATE_PASSWORD`
- `IOS_PROVISIONING_PROFILE_BASE64`

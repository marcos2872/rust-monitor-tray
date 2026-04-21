---
name: release
description: "Guia para criar e publicar releases por tag neste projeto. Usa o fluxo documentado em RELEASE.md e o pipeline do GitHub Actions acionado por tags v*."
argument-hint: "Opcionalmente informe a versão desejada, por exemplo: v0.2.1"
---

# Como criar uma release

O pipeline de CI compila e publica automaticamente via GitHub Actions quando uma tag `v*` é criada.

> Release atual publicada: **`v0.2.0`**

## Passos

**1. Rode testes e lint obrigatoriamente antes da tag**

Esse passo é **obrigatório** antes de criar qualquer release.

```bash
make test
make lint
```

Se o ambiente não tiver `qmllint`, ao menos garanta explicitamente:

```bash
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

**2. Escolha o número da versão** seguindo [Semantic Versioning](https://semver.org):

| Mudança | Exemplo |
|---|---|
| Correção de bug | `v0.2.1` |
| Nova funcionalidade | `v0.3.0` |
| Quebra de compatibilidade | `v1.0.0` |

**3. Crie e envie a tag**

```bash
git tag v0.2.0
git push origin v0.2.0
```

O GitHub Actions irá automaticamente:
- Executar `cargo test`
- Compilar o binário (`cargo build --release`)
- Empacotar o plasmoid (`monitor-tray-plasmoid.tar.gz`)
- Criar a release em [github.com/marcos2872/rust-monitor-tray/releases](https://github.com/marcos2872/rust-monitor-tray/releases)

**4. Acompanhe o build** em [Actions](https://github.com/marcos2872/rust-monitor-tray/actions) (~2 min)

**5. Verifique a release** em [Releases](https://github.com/marcos2872/rust-monitor-tray/releases) e confira os assets:
- `monitor-tray` — binário Linux x86_64
- `monitor-tray-plasmoid.tar.gz` — plasmoid + scripts de instalação

**6. Atualize a documentação da release (obrigatório)**

Após publicar a nova tag, sincronize a versão atual em:

- `RELEASE.md`
- `.agents/skills/release/SKILL.md`
- `wiki/Home.md`

Esse passo é obrigatório para manter a skill e a documentação coerentes com a release mais recente.

---

## Remover uma tag (se necessário)

```bash
git tag -d v0.2.0                  # remove local
git push origin --delete v0.2.0   # remove remota
```

# Como criar uma release

O pipeline de CI compila e publica automaticamente via GitHub Actions quando uma tag `v*` é criada.

## Passos

**1. Certifique-se de que o branch `main` está estável**

```bash
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

**2. Escolha o número da versão** seguindo [Semantic Versioning](https://semver.org):

| Mudança | Exemplo |
|---|---|
| Correção de bug | `v1.0.1` |
| Nova funcionalidade | `v1.1.0` |
| Quebra de compatibilidade | `v2.0.0` |

**3. Crie e envie a tag**

```bash
git tag v1.0.0
git push origin v1.0.0
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

---

## Remover uma tag (se necessário)

```bash
git tag -d v1.0.0                  # remove local
git push origin --delete v1.0.0   # remove remota
```

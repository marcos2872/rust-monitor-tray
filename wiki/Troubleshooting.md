# Troubleshooting

---

## O widget mostra "Backend DBus indisponível"

O backend Rust não está em execução ou não está registrado no DBus.

**Verificar o serviço:**
```bash
systemctl --user status monitor-tray.service
```

**Ver os logs:**
```bash
journalctl --user -u monitor-tray.service -n 50
```

**Testar manualmente:**
```bash
gdbus call --session \
  --dest com.monitortray.Backend \
  --object-path /com/monitortray/Backend \
  --method com.monitortray.Backend.Ping
```
Deve retornar `('ok',)`.

**Reiniciar o serviço:**
```bash
systemctl --user restart monitor-tray.service
```

---

## O widget não aparece na lista de widgets do KDE

O plasmoid pode não ter sido instalado corretamente.

```bash
# Verificar se está instalado
kpackagetool6 --type Plasma/Applet --list | grep monitortray

# Reinstalar
kpackagetool6 --type Plasma/Applet --remove com.monitortray.plasmoid
kpackagetool6 --type Plasma/Applet --install plasma/
```

Se ainda não aparecer, reinicie o `plasmashell`:
```bash
kquitapp6 plasmashell && plasmashell &
```

---

## O ícone do widget aparece como "?"

O ícone precisa estar no tema `hicolor` do usuário.

```bash
# Verificar se o ícone está no lugar certo
ls ~/.local/share/icons/hicolor/256x256/apps/com.monitortray.plasmoid.png

# Copiar e atualizar o cache
cp plasma/contents/icons/com.monitortray.plasmoid.png \
   ~/.local/share/icons/hicolor/256x256/apps/
kbuildsycoca6 --noincremental
```

---

## Sensores não aparecem

Os sensores dependem do que o kernel expõe em `/sys/class/hwmon`.

```bash
# Verificar chips hwmon disponíveis
ls /sys/class/hwmon/
for d in /sys/class/hwmon/hwmon*/; do echo "$(cat $d/name): $(ls $d/temp*_input 2>/dev/null | wc -l) sensores"; done

# Carregar módulos de kernel para sensores (se necessário)
sudo modprobe coretemp   # Intel
sudo modprobe k10temp    # AMD
```

---

## GPU não aparece ou aparece como "GPU (cardN)"

**AMD:** Verificar se o driver `amdgpu` está ativo:
```bash
cat /sys/class/drm/card1/device/uevent | grep DRIVER
ls /sys/class/drm/card1/device/gpu_busy_percent
```

**NVIDIA:** Verificar se `nvidia-smi` está no PATH:
```bash
which nvidia-smi
nvidia-smi --query-gpu=name --format=csv,noheader
```

**Intel:** O suporte é limitado. Clock só disponível em kernel ≥ 5.16:
```bash
ls /sys/class/drm/card0/gt/gt0/rps_cur_freq_mhz 2>/dev/null || echo "não disponível"
```

---

## O plasmoid pisca ao atualizar

Todas as tabs usam property bindings diretos. Se ainda houver piscar, verifique se alguma tab usa `Repeater` no nível raiz — isso recria os delegates a cada atualização do objeto `metrics`.

---

## Erro de compilação: "cargo não encontrado"

```bash
# Instalar Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Verificar
cargo --version
```

---

## Porta DBus já em uso / serviço duplicado

```bash
# Verificar se há outra instância do backend rodando
gdbus call --session --dest com.monitortray.Backend \
  --object-path /com/monitortray/Backend \
  --method com.monitortray.Backend.Ping

# Matar processos órfãos
pkill -f "monitor-tray --dbus"
systemctl --user restart monitor-tray.service
```

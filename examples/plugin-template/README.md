# Paths — Template de Plugin Nativo (.so)

Este diretório contém a estrutura modelo para criação e compilação de plugins externos em formato nativo Linux (`.so`) para o Paths.

## Como Compilar

```bash
cd examples/plugin-template
cargo build --release
```

O arquivo gerado estará em `target/release/libgnome_paths_example_plugin.so`.

## Como Instalar

### Opção 1: Pela Interface Gráfica
1. Abra o Paths
2. Vá em **Menu Principal → Preferências → Plugins**
3. Clique em **"Instalar novo plugin (.so)..."** e selecione o arquivo `.so` compilado.

### Opção 2: Manualmente
Copie o arquivo `.so` para a pasta de plugins do usuário:
```bash
mkdir -p ~/.local/share/gnome-paths/plugins
cp target/release/libgnome_paths_example_plugin.so ~/.local/share/gnome-paths/plugins/
```

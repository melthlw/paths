<p align="center">
  <img src="data/icons/hicolor/scalable/apps/io.github.lewis.GnomePaths.svg" alt="Logotipo do GNOME Paths" height="128">
</p>

<h1 align="center">GNOME Paths</h1>
<p align="center"><em>Editor de gráficos vetoriais e ilustrações para o ambiente GNOME.</em></p>

<p align="center">
  🇺🇸 <a href="README.md">Read in English</a>
</p>

<p align="center">
  Criar e editar gráficos vetoriais com mais rápidez, responsividade e perfeitamente integrado ao ambiente de trabalho.
</p>

<p align="center">
  O <b>GNOME Paths</b> é uma ferramenta de design vetorial leve, construída com <b>GTK4</b>, <b>Libadwaita</b>, <b>Rust</b> e o motor gráfico <b>Skia</b>.
</p>

<div align="center">
  <div style="display: flex; flex-wrap: wrap; justify-content: center; gap: 1em;">
    <a href="https://flathub.org/">
      <img width="190" alt="Baixar no Flathub" src="https://flathub.org/api/badge?locale=pt_BR" />
    </a>
  </div>
</div>

<div align="center" style="display:flex; justify-content:center; align-items:center; gap:12px; margin-top: 14px;">
    <a href="https://ko-fi.com/lauel" style="display:flex; align-items:center;">
        <img src="https://ko-fi.com/img/githubbutton_sm.svg"
             alt="Apoie no Ko-fi"
             style="height:30px; width:auto; display:block;">
    </a>
</div>

---

<p align="center" style="display: flex; justify-content: center; gap: 0.8em; flex-wrap: wrap;">
  <img alt="GTK4" src="https://img.shields.io/badge/GTK-4.18+-blue.svg" />
  <img alt="Libadwaita" src="https://img.shields.io/badge/Libadwaita-1.6+-purple.svg" />
  <img alt="Rust" src="https://img.shields.io/badge/Rust-2021-orange.svg" />
  <img alt="Engine" src="https://img.shields.io/badge/Engine-Skia%20GPU-green.svg" />
  <img alt="License" src="https://img.shields.io/badge/License-GPL--3.0--or--later-blue.svg" />
</p>

---

## Capturas de Tela

<div align="center" style="display: flex; flex-wrap: wrap; justify-content: center; gap: 16px;">
  <img src="screenshots/main-window.png" alt="Janela Principal" style="max-height:360px; max-width: 48%; object-fit: contain; border-radius: 8px;">
  <img src="screenshots/preferences.png" alt="Janela de Preferências" style="max-height:360px; max-width: 48%; object-fit: contain; border-radius: 8px;">
</div>

---

## Recursos

- **Edição de Caminhos Bézier**: Editor de nós com suporte a alças cúspides, suaves e simétricas.
- **Formas Paramétricas**: Retângulos, círculos, estrelas, polígonos e espirais.
- **Operações Booleanas**: União, Diferença, Interseção, Exclusão, Divisão e Cortar/Fatiar.
- **Gradientes e Malha**: Gradientes lineares, radiais e grades de malha 2D (Mesh Gradient) editáveis diretamente na tela.
- **Pranchetas Multipáginas**: Gerenciamento de múltiplas páginas em um único documento com opções de exportação individual.
- **Formatos de Exportação**: SVG, PNG, PDF, JPG e WebP.
- **Ajuste Magnético e Guias**: Encaixe magnético em grades, caixas delimitadoras de objetos e centros de pranchetas.
- **Interface Adaptativa**: Preferências com temas personalizados e tema claro e escuro do sistema com barras de ferramentas customizáveis.

---

## Sobre o Projeto e Visão

O **GNOME Paths** nasceu da paixão por computação gráfica vetorial, profundamente inspirado na versatilidade e no poder do **Inkscape**, com o objetivo de oferecer uma experiência moderna, rápida, fluida e com integração nativa ao desktop GNOME.

### Desenvolvimento e Transparência
Este projeto foi extensivamente desenvolvido e iterado com o auxílio de **inteligência artificial em pair-programming**, sendo cuidadosamente planejado, arquitetado e mantido com grande atenção aos detalhes e respeito ao ecossistema do GNOME.

Prezamos pela transparência e pela colaboração aberta. Caso você prefira o fluxo de desenvolvimento tradicional e não utilize ferramentas de IA, sinta-se totalmente bem-vindo para contribuir com código tradicional, desenvolvimento de plugins, novas ideias, testes e relatórios de bugs.

### Horizontes e Ideias Futuras
Ainda não sei exatamente onde este caminho vai chegar ou que rumo o projeto tomará, mas há diversas ideias em mente que podem ser exploradas:
- **Espaços de Trabalho Dinâmicos**: Um sistema de interfaces adaptativas inspirado no Blender, onde a interface altera a disposição das ferramentas de acordo com o tipo de fluxo de trabalho (ex.: ilustração vetorial, edição de bitmap, diagramação editorial).
- **Animação Vetorial**: Linha do tempo, interpolação por quadros-chave (*keyframes*) e curvas de interpolação de movimento.
- **Diagramação Editorial e Documentos**: Ferramentas avançadas de layout multipágina para brochuras, livros e design editorial.
- **Sistema de Nós (Node Graphs)**: Edição procedural e não-destrutiva de caminhos, malhas e transformações através de grafos de nós.

Grande parte disso ainda são ideias e conceitos experimentais que poderão ou não se concretizar conforme o projeto evoluir.

---

## Contribuição e Apoio

O apoio da comunidade é fundamental para o crescimento do GNOME Paths. Se você deseja participar, toda forma de ajuda é muito bem-vinda:

- **Relatórios de Bugs e Sugestões**: Ajude a encontrar falhas, relatar comportamentos inesperados ou sugerir melhorias no GitLab.
- **Código e Desenvolvimento**: Envie pull requests com otimizações, correções de bugs ou novas ferramentas.
- **Traduções**: Ajude a traduzir o GNOME Paths para outros idiomas (consulte [TRANSLATING.md](TRANSLATING.md)).
- **Plugins**: Desenvolva plugins para adicionar novas funcionalidades ao GNOME Paths.
- **Apoio Financeiro**: Se o projeto for útil para você e quiser apoiar o desenvolvimento contínuo, considere contribuir no [Ko-fi](https://ko-fi.com/lauel).
---

## Como Compilar

### GNOME Builder

1. Instale o **GNOME Builder** pelo Flathub.
2. Clone a URL do repositório: `https://gitlab.gnome.org/lewisHeart/gnome-paths.git`.
3. Selecione a configuração de runtime Flatpak.
4. Clique em **Executar** para compilar e iniciar o aplicativo.

### Linha de Comando (Flatpak CLI)

```bash
# Instale o SDK e extensões do GNOME 47
flatpak install flathub \
  org.gnome.Platform//47 \
  org.gnome.Sdk//47 \
  org.freedesktop.Sdk.Extension.rust-stable//24.08 \
  org.freedesktop.Sdk.Extension.llvm19//24.08

# Compilar e instalar
flatpak-builder --user --install --force-clean build-dir io.github.lewis.GnomePaths.json

# Executar
flatpak run io.github.lewis.GnomePaths
```

### Meson e Ninja

```bash
meson setup build
ninja -C build
./build/gnome-paths
```

### Cargo

```bash
cargo run --release
```

Para executar os testes unitários:
```bash
cargo test
```

---

## Traduções

Instruções detalhadas sobre como contribuir e gerenciar traduções podem ser encontradas em **[TRANSLATING.md](TRANSLATING.md)**.

---

## Repositório e Suporte

- **GitLab**: [gitlab.gnome.org/lewisHeart/gnome-paths](https://gitlab.gnome.org/lewisHeart/gnome-paths)
- **Ko-fi**: [ko-fi.com/lauel](https://ko-fi.com/lauel)

---

## Licença

O GNOME Paths é licenciado sob a [GNU General Public License v3.0 ou posterior (GPL-3.0-or-later)](LICENSE).

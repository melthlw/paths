<p align="center">
  <img src="data/icons/hicolor/scalable/apps/io.github.lewis.GnomePaths.svg" alt="Logotipo do GNOME Paths" height="128">
</p>

<h1 align="center">GNOME Paths</h1>

<p align="center">
  <b>Estúdio moderno de ilustrações e computação gráfica vetorial para o GNOME.</b>
</p>

<p align="center">
  <a href="README.pt-BR.md">Versão em Português</a> • 
  <a href="README.md">Versão em Inglês</a>
</p>

<p align="center">
  <img alt="GTK4" src="https://img.shields.io/badge/GTK-4.18+-3584e4.svg?style=flat-square&logo=gnome" />
  <img alt="Libadwaita" src="https://img.shields.io/badge/Libadwaita-1.6+-9141ac.svg?style=flat-square" />
  <img alt="Rust" src="https://img.shields.io/badge/Rust-2021%20%2F%202024-e66100.svg?style=flat-square&logo=rust" />
  <img alt="Engine" src="https://img.shields.io/badge/Engine-Skia%202D%20GPU-26a269.svg?style=flat-square" />
  <img alt="Licença" src="https://img.shields.io/badge/Licença-GPL--3.0--or--later-1c71d8.svg?style=flat-square" />
</p>

<div align="center" style="margin-top: 14px; margin-bottom: 24px;">
  <a href="https://ko-fi.com/lauel">
    <img src="https://ko-fi.com/img/githubbutton_sm.svg" alt="Apoie no Ko-fi" style="height:32px; width:auto; display:inline-block;">
  </a>
</div>

---

## Capturas de Tela

<div align="center">
  <p><b>Área de Trabalho Principal e Ilustração Vetorial</b></p>
  <img src="screenshots/main-window.png" alt="Área de Trabalho do GNOME Paths" style="max-width: 100%; border-radius: 10px; box-shadow: 0 8px 24px rgba(0,0,0,0.4);" />
</div>

<br/>

<div align="center">
  <p><b>Painel de Preferências e Configurações</b></p>
  <img src="screenshots/preferences.png" alt="Preferências do GNOME Paths" style="max-width: 85%; border-radius: 10px; box-shadow: 0 8px 24px rgba(0,0,0,0.4);" />
</div>

---

## Recursos e Funcionalidades

<table>
  <tr>
    <td width="50%">
      <h3>Edição Avançada de Nós e Curvas Bezier</h3>
      <p>Controle cirúrgico de caminhos com nós cúspides (canto vivo), suaves e simétricos. Manipulação interativa de alças tangentes, inserção e remoção de nós, conversão de segmentos (reta/curva) e suavização contínua com de Casteljau.</p>
    </td>
    <td width="50%">
      <h3>Customização de Interface e Espaço de Trabalho</h3>
      <p>Barra de ferramentas HUD flutuante ou acoplada com reorganização de ferramentas, painéis laterais retráteis (Inspetor de Propriedades, Camadas e Bibliotecas) e integração com tema claro e escuro do GNOME.</p>
    </td>
  </tr>
  <tr>
    <td width="50%">
      <h3>Pranchetas Multipáginas (Artboards)</h3>
      <p>Ambiente multipágina com suporte a múltiplas pranchetas independentes em um único documento. Controle de dimensões por prancheta, reordenação de páginas, navegação e exportação individual ou em lote.</p>
    </td>
    <td width="50%">
      <h3>Operações Booleanas em Vetores</h3>
      <p>Combinação geométrica de caminhos em tempo real com algoritmos de <b>União</b>, <b>Diferença</b>, <b>Interseção</b>, <b>Exclusão</b>, <b>Divisão</b> e <b>Fatiamento / Corte</b>.</p>
    </td>
  </tr>
  <tr>
    <td width="50%">
      <h3>Gradientes em Malha (Mesh Gradient)</h3>
      <p>Pintura vetorial avançada com gradientes lineares, radiais com múltiplos pontos de parada e grades de malha 2D editáveis diretamente sobre os objetos no canvas.</p>
    </td>
    <td width="50%">
      <h3>Clones Vinculados (Linked Clones)</h3>
      <p>Instanciação de objetos vinculados com sincronização automática do elemento mestre e opção de desvinculação seletiva para criação de padrões complexos.</p>
    </td>
  </tr>
  <tr>
    <td width="50%">
      <h3>Formas Geométricas Paramétricas</h3>
      <p>Primitivas editáveis dinamicamente: retângulos com raios de canto independentes, círculos, elipses, polígonos regulares com ajuste de lados, estrelas e espirais.</p>
    </td>
    <td width="50%">
      <h3>Ajuste Magnético e Guias Inteligentes</h3>
      <p>Encaixe magnético com alinhamento dinâmico a centros de pranchetas, caixas delimitadoras de objetos, nós vizinhos, réguas com origem configurável e linhas-guia.</p>
    </td>
  </tr>
</table>

---

## Formatos Suportados de Exportação e Importação

| Formato | Exportação | Importação | Características Principais |
| :--- | :---: | :---: | :--- |
| **SVG** | Sim | Sim | Preservação total de dados de projeto, curvas W3C padrão, gradientes e metadados |
| **PNG** | Sim | Não | Rasterização em alta resolução com canal alfa (transparência) |
| **PDF** | Sim | Não | Páginas vetoriais prontas para impressão e publicação |
| **JPG** | Sim | Não | Imagens rasterizadas compactadas para web e pré-visualização |
| **WebP** | Sim | Não | Formato moderno de alta fidelidade para gráficos na web |

---

## Como Compilar e Executar

### Pré-requisitos

Pacotes de desenvolvimento necessários:
- **Rust** (versão estável)
- **GTK4** (`>= 4.18`)
- **Libadwaita** (`>= 1.6`)
- **Clang / LLVM** (necessário para a compilação do `skia-safe`)

### 1. Cargo (Desenvolvimento Local)

```bash
git clone https://gitlab.com/lewisHeart/gnome-paths.git
cd gnome-paths

cargo run --release
```

Para rodar todos os 68 testes unitários automatizados:
```bash
cargo test
```

### 2. GNOME Builder (Flatpak)

1. Abra o **GNOME Builder**.
2. Clone o repositório `https://gitlab.com/lewisHeart/gnome-paths.git`.
3. Selecione o runtime Flatpak **GNOME 47**.
4. Clique em **Executar**.

### 3. Flatpak Builder via Linha de Comando

```bash
# Instalar SDK e extensões do GNOME 47
flatpak install flathub \
  org.gnome.Platform//47 \
  org.gnome.Sdk//47 \
  org.freedesktop.Sdk.Extension.rust-stable//24.08 \
  org.freedesktop.Sdk.Extension.llvm19//24.08

# Compilar e instalar o pacote Flatpak
flatpak-builder --user --install --force-clean build-dir io.github.lewis.GnomePaths.json

# Executar o aplicativo
flatpak run io.github.lewis.GnomePaths
```

### 4. Meson e Ninja

```bash
meson setup build
ninja -C build
./build/gnome-paths
```

---

## Sobre o Projeto e Visão

O **GNOME Paths** nasceu da paixão por gráficos vetoriais, profundamente inspirado na versatilidade e no poder do **Inkscape**, com o objetivo de oferecer uma experiência moderna, rápida, fluida e com integração nativa ao ambiente GNOME.

### Desenvolvimento e Transparência
Este projeto é ativamente desenvolvido com o auxílio de **inteligência artificial em pair-programming**, sendo cuidadosamente estruturado, planejado e mantido com olhar e arquitetura humana.

Prezamos pela transparência e colaboração aberta. Caso você prefira o fluxo de desenvolvimento tradicional sem ferramentas de IA, você é calorosamente convidado a contribuir via pull requests, revisões de código, desenvolvimento de plugins nativos, sugestões de design ou relatórios de bugs.

### Possíveis Ideias Futuras
- **Espaços de Trabalho Dinâmicos**: Interfaces adaptativas que se reorganizam conforme o foco do trabalho (Ilustração, Editorial, Edição de Bitmap, Tipografia).
- **Animação Vetorial**: Linha do tempo, interpolação por quadros-chave (keyframes) e curvas de movimento.
- **Sistema de Nós Procedurais**: Operações e modificadores não-destrutivos baseados em grafos de nós.
- **Diagramação Editorial Avançada**: Páginas-mestre, colunas de texto encadeadas e layout multipágina para livros e brochuras.

---

## Contribuição e Comunidade

- **Relatórios de Bugs e Sugestões**: Abra uma issue no [Rastreador de Issues do GitLab](https://gitlab.com/lewisHeart/gnome-paths/-/issues).
- **Traduções**: Ajude a traduzir o GNOME Paths para outros idiomas! Veja **[TRANSLATING.md](TRANSLATING.md)**.
- **Plugins**: Veja o modelo de plugin em [`examples/plugin-template`](examples/plugin-template/).
- **Apoie o Projeto**: Se o GNOME Paths for útil para você, considere apoiar o desenvolvimento no [Ko-fi](https://ko-fi.com/lauel).

---

## Licença

O GNOME Paths é um software livre de código aberto sob a licença **[GNU General Public License v3.0 ou posterior (GPL-3.0-or-later)](LICENSE)**.

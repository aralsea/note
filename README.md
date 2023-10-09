# Note
## Overview
vscode拡張機能のlatex-workshopが快適に動くような環境を自動設定するツールです。

## Requirements
- MacOS
- rust
  https://www.rust-lang.org/tools/install
- vscode + latex-workshop
  ``` bash
  brew install --cask visual-studio-code
  code --install-extension James-Yu.latex-workshop
  ```
- Texlive
  ``` bash
  brew install --cask mactex-no-gui
  ```
- biblatex + biber
  ``` bash
  tlmgr install biblatex
  tlmgr install biber
  ```

## Installation
``` bash
cargo install --git https://github.com/aralsea/note
```
- `~/.note` ディレクトリが作成され、そこに設定ファイルやテンプレートファイルが保存される

## Usage
- `note new PROJECT_NAME`: カレントディレクトリに`PROJECT_NAME`ディレクトリが作成され、latex-workshopが動く環境が構成される
  - `note new --language english PROJECT_NAME`: 英語用の設定を使用
  - 使用されるテンプレートは `~/.note/templates/src`を編集することで変更できる 
- `note config`: 現在の設定を確認
- `note config --author YOUR_NAME`: 著者名のデフォルト値を設定する
- `note config --lang english`: 言語のデフォルトを英語に設定する
- その他の詳細は `note --help`を参照
  

# Sugoroku

盤面が見えない状態でサイコロの出目を自由に決めながら進める双六ゲームです。

## インストール方法

`install.sh`を実行してください。
実行には`cargo`コマンドが必要なので、ない場合は[このページ](https://www.rust-lang.org/ja/tools/install)からインストールしてください。

## 補完スクリプトの作成

`make_completion_script.sh`を実行してください。

## 実行方法

ゲームを実行する場合

```sh
sugoroku game <player list file> <world file>
```

ワールドファイルをLaTeX形式で出力する場合（同じディレクトリに拡張子が`tex`に変更されたファイルが作成されます。既存の場合は上書きされます。）

```sh
sugoroku world-to-tex <world file>
```

ヘルプを見る場合

```sh
sugoroku
```

## 入力ファイル

プレイヤーリストとワールドが記述されたファイルがそれぞれ必要となります。
どちらもTOML形式で記述されます。
書き方と例は[sugoroku\_examples](sugoroku_examples)に置いてあります。

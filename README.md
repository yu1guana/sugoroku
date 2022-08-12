# Sugoroku

盤面が見えない状態でサイコロの出目を自由に決めながら進める双六ゲームです。

## インストール方法

`install.sh`を実行してください。
実行には`cargo`コマンドが必要なので、ない場合は[このページ](https://www.rust-lang.org/ja/tools/install)からインストールしてください。

## 補完スクリプトの作成

`make_completion_script.sh`を実行してください。

## 入力ファイル

プレイヤーリストとワールドが記述されたファイルがそれぞれ必要となります。
どちらもTOML形式で記述されます。
[sugoroku\_examples](sugoroku_examples)に例を置いてあります。

## 現在設定できるエリア効果

### GoToStart

|||
|-|-|
|効果|ふりだしに戻る。|
|形式|GoToStart:|

### SkipSelf

|||
|-|-|
|効果|休みを追加する。|
|形式|SkipSelf: times = \<u8>|

### PushSelf

|||
|-|-|
|効果|プレイヤーを進める。|
|形式|PushSelf: num = \<usize>|

### PullSelf

|||
|-|-|
|効果|プレイヤーを戻す。|
|形式|PullSelf: num = \<usize>|

### PushOthersAll

|||
|-|-|
|効果|自分以外を進める。|
|形式|PushOthersAll: num = \<usize>|

### PullOthersAll

|||
|-|-|
|効果|自分以外を戻す。|
|形式|PullOthersAll: num = \<usize>|

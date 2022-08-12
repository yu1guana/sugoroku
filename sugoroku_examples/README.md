# Sugoroku の入力ファイル

## プレイヤーリスト

### 書き方

次のようにプレイヤーの名前を一人ずつ追加していく。

```toml
[[player]]
name = "Alice"

[[player]]
name = "Bob"
```


### 例

- [player\_list.toml](player_list.toml)

## ワールド

### 書き方

まず最初にワールド全体の設定を次のように書きます。

```toml
[general]
title = "タイトル"
opening_msg = "タイトル画面に出力される文章"
start_description = "スタート位置の文章"
goal_description = "ゴール位置の文章"
dice_max = 4  # これはサイコロの最大値
```

次に各マスの文章と効果を次の要領で書いていきます。
書いた順に1マス目から順に割り当てられます。
マスに割り当てる効果の書き方は[こちら](#現在設定できるエリア効果)を見てください。

```toml
[[area]]
description = "表示される文章"
[[area.effect]]
element = "PushSelf: num = 2"
[[area.effect]] # ひとつのマスに複数の効果を当てることもできる。
element = "SkipSelf: times = 1"

[[area]] # 効果のないマスも作れます。
description = "表示される文章"
```

### 例

- [world\_01.toml](world_01.toml)

### 現在設定できるエリア効果

#### GoToStart

|||
|-|-|
|効果|ふりだしに戻る。|
|形式|GoToStart:|

#### SkipSelf

|||
|-|-|
|効果|休みを追加する。|
|形式|SkipSelf: times = \<u8>|

#### PushSelf

|||
|-|-|
|効果|プレイヤーを進める。|
|形式|PushSelf: num = \<usize>|

#### PullSelf

|||
|-|-|
|効果|プレイヤーを戻す。|
|形式|PullSelf: num = \<usize>|

#### PushOthersAll

|||
|-|-|
|効果|自分以外を進める。|
|形式|PushOthersAll: num = \<usize>|

#### PullOthersAll

|||
|-|-|
|効果|自分以外を戻す。|
|形式|PullOthersAll: num = \<usize>|

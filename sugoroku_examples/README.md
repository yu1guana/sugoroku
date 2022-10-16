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

まず最初に盤面全体の設定を次のように書きます。

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
マスに割り当てる効果の書き方は[こちら](#現在設定できる効果)を見てください。

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

### 現在設定できる効果

| 名前       | 効果                           | 入力形式                 |
| -          | -                              | -                        |
| GoToStart  | 振り出しに戻る。               | GoToStart:               |
| SkipSelf   | 休みを追加する。               | SkipSelf: times = \<u8>  |
| PushSelf   | プレイヤーを進める。           | PushSelf : num = \<u8>   |
| PullSelf   | プレイヤーを戻す。             | PullSelf: num = \<u8>    |
| PushOthers | 自分以外のプレイヤーを進める。 | PushOthers : num = \<u8> |
| PullOthers | 自分以外のプレイヤーを戻す。   | PullOthers: num = \<u8>  |

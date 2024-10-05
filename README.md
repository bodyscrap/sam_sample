# 画像認識アプリサンプル(Tauri + Vanilla + candle)

## 1. 概要
Tauri と candleを使ったアプリケーションの練習。  

## 2. ビルド時の注意
本サンプルはhuggingface candleのSegmentAnythingの[example](https://github.com/huggingface/candle/tree/main/candle-examples/examples/segment-anything)を参考に作成しています。  
そのため、サンプルと同じ組込みのモデルを使用しているため、次のモデルをダウンロードして`src-tauri/models`に配置してからビルドしてください。  

[mobile_sam-tiny-vitt.safetensors](https://huggingface.co/lmz/candle-sam/blob/main/mobile_sam-tiny-vitt.safetensors)

もともとのサンプルでは`hf-hub` crateを使用し、モデルが無い場合はダウンロードをする仕組みになっていますが、ネットワークが無い環境への配布を想定してビルド時にバンドルする構成に書き換えています。  
モデルデータ自体は私のオリジナルで無いことと、サイズが大きいことから、リポジトリには含めていません。  

あとは普通に`cargo tauri dev`や`cargo tauri build`を実行すればビルドできます。  

## 3. アプリの使い方
(作成中)

- 「開く」ボタンで画像を開く
- 画像上をクリックして注目点を指定(1点)
- 「セグメンテーション」ボタンでSegment Anythingを実行
- セグメンテーション結果のマスクが画像にオーバーレイして表示される
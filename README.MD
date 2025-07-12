# Rusted-CA

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)](https://github.com/kazuma0606/rusted-ca/actions)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-stable-blue?logo=rust)](https://www.rust-lang.org/)

## distributed-storageブランチについて

このブランチは**MySQL+Redisによる分散ストレージPoC**構成です。
- docker-composeでMySQL/Redis/phpMyAdmin/RedisInsightを一発起動
- Rustアプリは**POST /api/user**でMySQLとRedis両方にデータを保存
- TiDB/MockTiKVは使いません

---

## クイックスタートガイド（MySQL+Redis）

1. **リポジトリをクローン**
```sh
git clone https://github.com/kazuma0606/rusted-ca.git
cd rusted-ca/rusted-ca
```
2. **docker-composeでDB/Redis起動**
```sh
docker-compose up -d
```
- MySQL: `localhost:3306`（ユーザー: root, パスワード: root, DB: rusted_ca）
- Redis: `localhost:6379`
- phpMyAdmin: [http://localhost:8080](http://localhost:8080)（root/root）
- RedisInsight: [http://localhost:8001](http://localhost:8001)

3. **Rustツールチェーンのインストール**
    - [Rust公式サイト](https://www.rust-lang.org/tools/install) の手順で `rustup` を導入
    - 推奨: 最新の stable チャンネル
4. **依存クレートの取得**
```sh
cargo fetch
```
5. **マイグレーション実行（MySQL）**
```sh
sqlx migrate run
```
6. **サーバー起動**
```sh
cargo run
```
    - デフォルトで `localhost:3000` でAPIが起動します。

---

## PoC API（分散ストレージ同期保存）

- **エンドポイント:** `POST /api/user`
- **機能:** 送信したユーザー情報をMySQLとRedis両方に保存

### 検証用curlコマンド
```sh
curl -X POST http://localhost:3000/api/user \
  -H "Content-Type: application/json" \
  -d '{
    "email": "john.doe@example.com",
    "name": "John Doe",
    "password": "supersecret123",
    "phone": "+1234567890",
    "birth_date": "1990-01-01"
  }'
```
- 成功するとJSONでユーザー情報が返ります
- MySQL/Redis両方にデータが保存されます

---

## docker-compose構成
- `mysql:8.0`（DB: rusted_ca, ユーザー: root/root）
- `redis/redis-stack:latest`
- `phpmyadmin/phpmyadmin:5.2`（MySQL管理GUI）
- `RedisInsight`（Redis管理GUI）

---

## 注意点
- **TiDB/MockTiKVはこのブランチでは使いません**
- テーブル定義やマイグレーションはMySQL互換で記述してください
- DB/Redisが完全に起動してからマイグレーション・アプリ起動を行ってください
- 開発用のためパスワード等は簡易設定です。本番運用時は適宜変更してください

---

## その他の機能・設計思想
- クリーンアーキテクチャ + CQRS + DI + gRPC + Discord通知 + セキュリティヘッダー対応
- ドメイン層/アプリケーション層/インフラ層/プレゼンテーション層の分離
- CQRS（コマンド・クエリ分離）
- DIコンテナ
- ログ・メトリクス収集
- **Discord通知（エラー時/拡張可）**
- **主要なセキュリティヘッダー自動付与（CSP, HSTS, X-Frame-Options, X-Content-Type-Options）**

---

## ライセンス

MIT License で誰でも自由に利用・改変・商用利用できます。

## Author

Yoshimura Hisanori

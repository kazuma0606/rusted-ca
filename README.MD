# Rusted-CA

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)](https://github.com/kazuma0606/rusted-ca/actions)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-stable-blue?logo=rust)](https://www.rust-lang.org/)

## distributed-storageブランチについて

このブランチは**MySQL+Redisによる分散ストレージPoC**構成です。
- docker-composeでMySQL/Redis/phpMyAdmin/RedisInsightを一発起動
- Rustアプリは**CRUD全API**でMySQLとRedis両方にデータを同期
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

## API仕様（CRUD + 分散ストレージ同期）

### 1. ユーザー作成（Create）
- **POST /api/user**
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
- **レスポンス例**
```json
{
  "id": "...",
  "email": "john.doe@example.com",
  "name": "John Doe",
  "phone": "+1234567890",
  "birth_date": "1990-01-01",
  "created_at": "...",
  "updated_at": "..."
}
```

### 2. ユーザー取得（Read）
- **GET /api/user/{id}**
```sh
curl -X GET http://localhost:3000/api/user/{USER_ID}
```
- **レスポンス例**
```json
{
  "id": "...",
  "email": "john.doe@example.com",
  "name": "John Doe",
  "phone": "+1234567890",
  "birth_date": "1990-01-01",
  "created_at": "...",
  "updated_at": "..."
}
```

### 3. ユーザー更新（Update）
- **PUT /api/user/{id}**
```sh
curl -X PUT http://localhost:3000/api/user/{USER_ID} \
  -H "Content-Type: application/json" \
  -d '{
    "name": "John Smith",
    "phone": "+1987654321",
    "birth_date": "1995-05-15"
  }'
```
- **部分更新も可能**
```sh
curl -X PUT http://localhost:3000/api/user/{USER_ID} \
  -H "Content-Type: application/json" \
  -d '{ "name": "John Updated" }'
```
- **レスポンス例**
```json
{
  "id": "...",
  "email": "john.doe@example.com",
  "name": "John Updated",
  "phone": "+1987654321",
  "birth_date": "1995-05-15",
  "created_at": "...",
  "updated_at": "..."
}
```

### 4. ユーザー削除（Delete）
- **DELETE /api/user/{id}**
```sh
curl -X DELETE http://localhost:3000/api/user/{USER_ID}
```
- **レスポンス例**
```json
{
  "email": "john.doe@example.com",
  "name": "John Updated",
  "status": "deleted"
}
```

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

---

## マイグレーションの更新・運用手順

### 1. 新しいマイグレーションファイルの作成
```sh
sqlx migrate add <migration_name>
```
例: ユーザーテーブルにrole/statusカラムを追加したい場合
```sh
sqlx migrate add add_role_and_status_to_user
```

### 2. マイグレーションSQLの編集
`migrations/`ディレクトリに作成されたSQLファイルに、必要なALTER TABLEやCREATE TABLE文を記述します。

例:
```sql
ALTER TABLE users
  ADD COLUMN role VARCHAR(32) NOT NULL DEFAULT 'USER',
  ADD COLUMN status VARCHAR(32) NOT NULL DEFAULT 'ACTIVE';
```

### 3. マイグレーションの適用
```sh
sqlx migrate run
```
- DBが起動していることを確認してから実行してください。
- `DATABASE_URL`環境変数が必要です。
  - 例: `export DATABASE_URL=mysql://root:root@localhost:3306/rusted_ca`（Linux/mac）
  - 例: `$env:DATABASE_URL="mysql://root:root@localhost:3306/rusted_ca"`（PowerShell）

### 4. マイグレーションの状態確認
```sh
sqlx migrate info
```

### 5. マイグレーションのロールバック（直前の変更を取り消す）
```sh
sqlx migrate revert
```

### 6. Rust構造体・enumの型も必ず合わせて修正
- DBスキーマとRustの型がズレているとSQLxでエラーになります。
- 追加カラムは一時的にOption<>型で受けると安全です。

---

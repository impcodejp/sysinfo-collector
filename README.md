# sysinfo-collector

Windows / Windows Server 環境のシステム情報を一括収集し、テキストファイルに出力する CLI ツール。

## 収集情報

- ネットワーク情報（IP・MAC・DNS・ゲートウェイ等）
- OS 情報（バージョン・ビルド番号・起動日時等）
- ハードウェア情報（CPU・メモリ）
- ドライブ情報（容量・使用率）
- タスクスケジューラ情報（ユーザー定義タスクのみ）

## 動作要件

| 項目 | 内容 |
|------|------|
| OS | Windows 10/11、Windows Server 2016/2019/2022 |
| 権限 | **管理者権限（Administrator）必須** |
| アーキテクチャ | x86_64（64bit） |

## ビルド

```powershell
# リリースビルド
cargo build --release --target x86_64-pc-windows-msvc
```

成果物: `target\x86_64-pc-windows-msvc\release\sysinfo-collector.exe`

## 使い方

```powershell
# カレントディレクトリに出力
.\sysinfo-collector.exe

# 出力先を指定
.\sysinfo-collector.exe --output C:\Reports
.\sysinfo-collector.exe -o C:\Reports
```

出力ファイル名: `sysinfo_<ホスト名>_<YYYYMMDD_HHMMSS>.txt`

## テスト

```powershell
cargo test                        # 全テスト
cargo test --lib                  # ユニットテストのみ
cargo test --test collector_test  # 統合テスト（要 Windows 環境）
```

## プロジェクト構成

```
src/
├── main.rs             # エントリポイント
├── collector/          # 情報収集ロジック
│   ├── network.rs
│   ├── os_info.rs
│   ├── hardware.rs
│   ├── drive.rs
│   └── scheduler.rs
├── model/              # データ構造定義
├── formatter/          # テキスト出力フォーマット
└── writer/             # ファイル書き込み
```

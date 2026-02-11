# gg - Git & GitHub CLI Guard

[English](README.md)

`git` と `gh` コマンドにポリシーを適用する安全プロキシ。AIコーディングエージェントによる危険な操作を防止します。

```
$ gg push --force origin main
[gg] BLOCKED: `git push --force origin main` is denied by policy

$ gg status
On branch main
nothing to commit, working tree clean

$ gg pr list
Showing 3 of 3 pull requests in owner/repo
```

## なぜ必要か

AIコーディングエージェント（Claude Code, Cursor, Copilot など）は、あなたの代わりに `git` や `gh` コマンドを実行します。多くの場合は問題ありませんが、一部のコマンドは破壊的で元に戻すのが困難です:

- `git push --force` はリモートの履歴を上書きする
- `git reset --hard` はコミットされていない変更を破棄する
- `gh repo delete` はリポジトリ全体を削除する
- `gh pr merge` は人間のレビューなしにマージする

**gg** はエージェントと git/gh の間に立ち、コマンド実行前に allow/confirm/deny ルールを適用します。

## インストール

### ソースから

```bash
cargo install --git https://github.com/noumi0k/gg
```

### crates.io から

```bash
cargo install github-guard
```

### バイナリ

[GitHub Releases](https://github.com/noumi0k/gg/releases) からダウンロード。

## クイックスタート

1. 設定ファイルを `~/.config/gg/config.toml` に作成:

```toml
[options]
deny_by_default = true

[git.rules]
allow = ["status*", "log*", "diff*", "branch*"]
confirm = ["add*", "commit*", "push", "pull*"]
deny = ["push --force*", "push -f*", "reset --hard*"]

[gh.rules]
allow = ["pr list*", "pr view*", "issue list*"]
confirm = ["pr create*", "issue create*"]
deny = ["pr merge*", "repo delete*"]
```

2. AI エージェントが `git`/`gh` の代わりに `gg` を使うよう設定:

**Claude Code** (`~/.claude/settings.json`):
```json
{
  "permissions": {
    "allow": ["Bash(gg *)"],
    "deny": ["Bash(git *)", "Bash(gh *)"]
  }
}
```

**シェルエイリアス** (手動利用時):
```bash
alias git='gg --git'
alias gh='gg --gh'
```

## 仕組み

```
エージェントが実行: gg push origin main
                        │
                        ▼
                ┌──────────────┐
                │  自動判別     │  git か gh か？
                │  git vs gh   │  (ルール → サブコマンド一覧)
                └──────┬───────┘
                       │
                       ▼
                ┌──────────────┐
                │  ルール評価   │  deny → confirm → allow の順
                │              │  (最初にマッチしたルールが適用)
                └──────┬───────┘
                       │
               ┌───────┼────────┐
               ▼       ▼        ▼
            [許可]   [確認]    [拒否]
               │       │        │
               │    ユーザーに   ブロック
               │    確認        (exit 77)
               ▼       │
            git/gh  ┌─┴─┐
            実行    y   n
                    │   │
                 実行  ブロック
```

### ルール評価順序

1. **deny** ルールを最初にチェック - マッチしたらブロック
2. **confirm** ルールを次にチェック - マッチしたらユーザーに確認
3. **allow** ルールを最後にチェック - マッチしたら実行
4. どのルールにもマッチしない場合: `deny_by_default = true` ならブロック、`false` なら許可

### パターンマッチ

- 完全一致: `"status"` は `gg status` にマッチ
- 前方一致: `"push"` は `gg push origin main` にマッチ
- グロブ: `"push --force*"` は `gg push --force origin main` にマッチ

## 設定

### 設定ファイルの検索順序

1. `./gg.toml` (プロジェクトローカル)
2. `$GG_CONFIG` (環境変数)
3. `~/.config/gg/config.toml` (XDG)
4. プラットフォーム設定ディレクトリ (macOS: `~/Library/Application Support/gg/config.toml`)
5. `~/.gg.toml` (ホーム)

### オプション

| キー | デフォルト | 説明 |
|------|-----------|------|
| `deny_by_default` | `true` | ルールにマッチしないコマンドをブロック |
| `log` | `true` | 監査ログを書き込む |
| `log_file` | `~/.local/share/gg/audit.log` | カスタムログファイルパス |
| `priority` | `"git"` | コマンドが git と gh の両方のルールにマッチした場合の優先ツール |

### 環境変数

| 変数 | 説明 |
|------|------|
| `GG_CONFIG` | 設定ファイルのパス |
| `GG_GIT_PATH` | git バイナリのパス (デフォルト: `git`) |
| `GG_GH_PATH` | gh バイナリのパス (デフォルト: `gh`) |
| `GG_VERBOSE` | 設定読み込みメッセージを表示 |
| `GG_NO_LOCAL` | ローカル `./gg.toml` と `$GG_CONFIG` を無視 (グローバル設定のみ使用) |

## CLI リファレンス

```
gg [--git|--gh] <command...>

オプション:
  --git          git として強制実行
  --gh           gh として強制実行
  --dump-config  読み込まれた設定を表示
  -h, --help     ヘルプを表示
  -V, --version  バージョンを表示

終了コード:
  0     成功
  77    ポリシーによりブロック
  78    git/gh の判別不能 (--git または --gh を使用)
  other git/gh からのパススルー
```

## セキュリティ

脅威モデルと制限事項は [SECURITY.md](SECURITY.md) を参照してください。

**重要**: gg はポリシーレイヤーであり、サンドボックスではありません。コマンドが gg を経由する場合のみ保護が有効です。保護を最大化するための推奨設定はセキュリティドキュメントを参照してください。

## ライセンス

[MIT](LICENSE)

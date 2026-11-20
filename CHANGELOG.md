# 更新日志（CHANGELOG）

## 2026-11-20（基于官方 RustDesk 1.5.3）
- **分支重建**：本地与云端 `master` 同步至官方最新版 1.5.3，清理旧分支 `1.4.0-qnap`，从最新 `master` 拉出独立定制分支 `qnap-main`。
- **品牌定制**：新增图标生成脚本 `scripts/qnap_brand.py`，基于根目录 `favicon.ico` 生成替换全套 Windows `.ico`、macOS `.icns` 及 PNG 图标；将窗口标题与安装包元数据名称调整为 `QNAP RustDesk`。
- **中转服务器零硬编码**：新建独立云端打包工作流 `.github/workflows/qnap-build.yml`，全部通过 GitHub Secrets 注入 `RENDEZVOUS_SERVER`、`RS_PUB_KEY`、`API_SERVER`。
- **6 个月强制更新双保险**：新增独立后端模块 `src/qnap_update.rs` 与前端弹窗 `flutter/lib/desktop/widgets/qnap_force_update_dialog.dart`，实现“在线版本清单 `min_version` + 编译满 180 天兜底到期”双重校验，触发后弹窗不可关闭且 Rust 底层同时阻断主控与被控连接。
- **项目固定文档**：补齐 `PRODUCT.md`、`OPERATIONS.md`、`CHANGELOG.md`、`TODO.md`、`.env.example` 及版本清单模板 `docs/qnap-update-manifest.example.json`。

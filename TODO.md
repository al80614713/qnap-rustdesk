# 待办事项（TODO）

## 进行中
- 推送 `qnap-main` 至 GitHub，并由产品经理在仓库 Settings 填入中转服务器与版本清单 Secrets。

## 待办
- 首次在 GitHub Actions 触发 `QNAP RustDesk Build (Windows & macOS)` 工作流，完成 Windows `.exe` 与 macOS 双架构 `.dmg` 的云端编译验证。
- 创建首个 `manifest` Release 上传 `update-manifest.json`，并在实机上验证：品牌图标、默认中转连接、以及将 `min_version` 调高后的强制弹窗与底层断连效果。

## 已知问题
- macOS 无签名安装包在较新系统版本上首次启动会触发 Gatekeeper 拦截，需随包附带 `OPERATIONS.md` 中的客户打开指引。

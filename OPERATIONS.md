# QNAP RustDesk 运维与发布手册

## 1. GitHub Secrets 配置（必须先配，切勿写入代码）

在 GitHub 仓库网页进入：`Settings → Secrets and variables → Actions → New repository secret`，依次添加：

| Secret 名称 | 必填 | 说明 | 示例（仅格式参考） |
| --- | --- | --- | --- |
| `RENDEZVOUS_SERVER` | 是 | 自建 ID / 中转服务器地址（可带端口） | `rd.example.com` 或 `1.2.3.4:21116` |
| `RS_PUB_KEY` | 是 | 自建服务器公钥（`id_ed25519.pub` 内容） | `xxxxxxxxxxxxxxxxxxxx=` |
| `API_SERVER` | 否 | 自建 API 服务地址（未部署 Pro 版可留空） | `https://api.example.com` |
| `QNAP_UPDATE_MANIFEST_URL` | 推荐 | 在线版本清单 JSON 的直链 URL | `https://github.com/al80614713/qnap-rustdesk/releases/download/manifest/update-manifest.json` |

> **切国内地址方法**：日后只需把 `update-manifest.json` 上传到国内对象存储/服务器，修改 GitHub Secret `QNAP_UPDATE_MANIFEST_URL` 并重新打包即可，无需修改源码。

---

## 2. 如何触发云端打包与发布

1. 打开仓库 GitHub 页面 → 点击顶部 **Actions**。
2. 左侧选择 **QNAP RustDesk Build (Windows & macOS)**。
3. 点击右侧 **Run workflow**：
   - **Use workflow from**：选择 `qnap-main` 分支。
   - **release_tag**：
     - 测试打包：留空，产物会保存在本次运行的 **Artifacts** 中供下载。
     - 正式发版：填写标签名（如 `v1.5.3-qnap.1`），构建完成后自动上传到 GitHub Releases。
   - **expire_days**：默认 `180`（6 个月兜底到期；测试强制过期弹窗时可临时填 `1` 或改系统时间）。
4. 等待构建完成，得到 3 个安装包：
   - `qnap-rustdesk-1.5.3-x86_64.exe`（Windows 单文件）
   - `qnap-rustdesk-1.5.3-x86_64.dmg`（macOS Intel 芯片）
   - `qnap-rustdesk-1.5.3-aarch64.dmg`（macOS Apple M 系列芯片）

---

## 3. 版本清单 JSON 维护（强制更新开关）

1. 准备一份 `update-manifest.json`（模板见 [qnap-update-manifest.example.json](file:///Users/lian/开发/qnap-rustdesk/docs/qnap-update-manifest.example.json)）：
   ```json
   {
     "min_version": "1.5.3",
     "latest_version": "1.5.3",
     "download_url": "https://github.com/al80614713/qnap-rustdesk/releases/latest",
     "message": "当前 QNAP RustDesk 版本已过期，请点击下方按钮下载安装最新版本。"
   }
   ```
2. 托管到固定地址（例如在 GitHub Releases 创建一个名为 `manifest` 的 Release，上传 `update-manifest.json`）。
3. **需要淘汰老版本时**：将清单里的 `min_version` 调高为新版本号并重新上传覆盖，所有低于该版本的客户端下次启动就会弹出不可关闭的更新框并阻断远控。

---

## 4. 客户侧打开指引（macOS 无签名 .dmg）

由于 macOS 安装包为免费无签名版本，首次打开时系统可能提示“无法验证开发者”或“已损坏”。请客户按以下任一方式打开：

- **方式一（推荐，图形界面）**：
  1. 双击 `.dmg` 并将图标拖入 **应用程序（Applications）** 文件夹。
  2. 打开“访达 → 应用程序”，**右键点击**应用图标 → 选择 **打开** → 在弹窗中再次点击 **打开**。
  3. 若系统仍拦住，打开 **系统设置 → 隐私与安全性**，滑到底部找到拦截提示，点击 **仍要打开**。
- **方式二（终端一行命令，适用于 macOS 提示“已损坏”）**：
  打开“终端”执行：
  ```bash
  xattr -cr /Applications/RustDesk.app
  ```
  然后再双击打开，并在“系统设置 → 隐私与安全性”中授予**辅助功能**与**屏幕录制**权限。

---

## 5. 后续同步官方最新版 & 回退

- **同步官方最新版**：
  ```bash
  git checkout master
  git fetch upstream
  git merge --ff-only upstream/master
  git push origin master
  git checkout qnap-main
  git rebase master
  ```
- **紧急回退**：
  - 安装包回退：在 GitHub Releases 保留历史版本，并将 `update-manifest.json` 的 `min_version` 调回上一版本号。
  - 代码回退：`master` 永远纯净，可随时在 `qnap-main` 上 `git revert <commit>` 回退单次提交。

import 'dart:convert';
import 'package:flutter/material.dart';
import 'package:flutter_hbb/models/platform_model.dart';
import 'package:url_launcher/url_launcher.dart';

/// QNAP RustDesk 强制更新弹窗检查器。
///
/// 通过现有 FFI 接口 `mainGetCommon(key: 'qnap-force-update-status')` 异步查询
/// Rust 底层双保险校验结果（在线版本清单 `min_version` + 编译满 180 天兜底）。
/// 一旦判定为需要强制升级，弹出不可关闭的模态框引导下载新版本。
class QnapForceUpdateChecker {
  static bool _dialogShown = false;

  static Future<void> checkAndShow(BuildContext context) async {
    if (_dialogShown) return;
    try {
      final raw = await bind.mainGetCommon(key: 'qnap-force-update-status');
      if (raw.isEmpty) return;
      final data = jsonDecode(raw);
      if (data is! Map<String, dynamic>) return;
      if (data['blocked'] != true) return;

      final message = (data['message'] as String?)?.trim() ??
          '当前版本已停止使用，请下载安装最新版本的 QNAP RustDesk。';
      final downloadUrl = (data['download_url'] as String?)?.trim() ??
          'https://github.com/al80614713/qnap-rustdesk/releases/latest';
      final currentVersion = (data['current_version'] as String?) ?? '';
      final latestVersion = (data['latest_version'] as String?) ?? '';

      if (!context.mounted || _dialogShown) return;
      _dialogShown = true;

      await showDialog<void>(
        context: context,
        barrierDismissible: false,
        builder: (dialogContext) {
          return WillPopScope(
            onWillPop: () async => false,
            child: AlertDialog(
              title: const Row(
                children: [
                  Icon(Icons.system_update, color: Colors.orange),
                  SizedBox(width: 8),
                  Text('需要更新 QNAP RustDesk'),
                ],
              ),
              content: ConstrainedBox(
                constraints: const BoxConstraints(maxWidth: 420),
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(message),
                    if (currentVersion.isNotEmpty || latestVersion.isNotEmpty) ...[
                      const SizedBox(height: 12),
                      Text(
                        '当前版本：$currentVersion${latestVersion.isNotEmpty ? '   最新版本：$latestVersion' : ''}',
                        style: Theme.of(dialogContext).textTheme.bodySmall,
                      ),
                    ],
                  ],
                ),
              ),
              actions: [
                ElevatedButton.icon(
                  icon: const Icon(Icons.download),
                  label: const Text('前往下载最新版本'),
                  onPressed: () async {
                    final uri = Uri.tryParse(downloadUrl);
                    if (uri != null) {
                      await launchUrl(uri, mode: LaunchMode.externalApplication);
                    }
                  },
                ),
              ],
            ),
          );
        },
      );
    } catch (e) {
      debugPrint('QNAP force update check failed: $e');
    }
  }
}

#!/usr/bin/env python3
"""QNAP RustDesk 品牌图标生成脚本。

以根目录 favicon.ico（实为 192x192 透明底 PNG）为源图，生成并替换：
- Windows 多尺寸 ICO（res/icon.ico, res/tray-icon.ico, flutter/windows/runner/resources/app_icon.ico）
- 各尺寸 PNG（res/32x32.png, res/64x64.png, res/128x128.png, res/128x128@2x.png, res/icon.png, res/mac-icon.png）
- macOS 应用图标（flutter/macos/Runner/AppIcon.icns，需在 macOS 下使用 sips + iconutil）
"""

from pathlib import Path
import shutil
import struct
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parent.parent
SRC_PNG = ROOT / "favicon.ico"


def resize_png_sips(src: Path, dst: Path, size: int) -> None:
    """使用 macOS 内置 sips 缩放 PNG 图标。"""
    dst.parent.mkdir(parents=True, exist_ok=True)
    subprocess.run(
        ["sips", "-s", "format", "png", "-z", str(size), str(size), str(src), "--out", str(dst)],
        check=True,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.PIPE,
    )


def build_ico(png_entries: list[tuple[int, bytes]], dst: Path) -> None:
    """将多个尺寸的 PNG 字节打包为标准 Windows .ico 文件（PNG-in-ICO）。"""
    count = len(png_entries)
    header = struct.pack("<HHH", 0, 1, count)
    offset = 6 + count * 16
    dir_entries = []
    data_chunks = []
    for size, data in png_entries:
        dim = 0 if size >= 256 else size
        entry = struct.pack("<BBBBHHII", dim, dim, 0, 0, 1, 32, len(data), offset)
        dir_entries.append(entry)
        data_chunks.append(data)
        offset += len(data)
    dst.parent.mkdir(parents=True, exist_ok=True)
    dst.write_bytes(header + b"".join(dir_entries) + b"".join(data_chunks))


def generate_icns(src: Path, dst: Path, tmp_dir: Path) -> None:
    """使用 sips + iconutil 生成标准 macOS .icns 图标。"""
    iconset = tmp_dir / "AppIcon.iconset"
    iconset.mkdir(parents=True, exist_ok=True)
    specs = [
        ("icon_16x16.png", 16),
        ("icon_16x16@2x.png", 32),
        ("icon_32x32.png", 32),
        ("icon_32x32@2x.png", 64),
        ("icon_128x128.png", 128),
        ("icon_128x128@2x.png", 256),
        ("icon_256x256.png", 256),
        ("icon_256x256@2x.png", 512),
        ("icon_512x512.png", 512),
        ("icon_512x512@2x.png", 1024),
    ]
    for name, size in specs:
        resize_png_sips(src, iconset / name, size)
    dst.parent.mkdir(parents=True, exist_ok=True)
    subprocess.run(
        ["iconutil", "-c", "icns", str(iconset), "-o", str(dst)],
        check=True,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.PIPE,
    )


def main() -> int:
    if not SRC_PNG.exists():
        print(f"错误：找不到源图标 {SRC_PNG}", file=sys.stderr)
        return 1
    if not shutil.which("sips") or not shutil.which("iconutil"):
        print("错误：此脚本依赖 macOS 内置工具 sips 和 iconutil。", file=sys.stderr)
        return 1

    with tempfile.TemporaryDirectory(prefix="qnap_icons_") as td:
        tmp_dir = Path(td)
        # 1. 生成各类 PNG 资源
        png_targets = {
            ROOT / "res" / "32x32.png": 32,
            ROOT / "res" / "64x64.png": 64,
            ROOT / "res" / "128x128.png": 128,
            ROOT / "res" / "128x128@2x.png": 256,
            ROOT / "res" / "icon.png": 256,
            ROOT / "res" / "mac-icon.png": 256,
        }
        for target, size in png_targets.items():
            resize_png_sips(SRC_PNG, target, size)

        # 2. 生成多尺寸 Windows .ico
        ico_sizes = [16, 32, 48, 64, 128, 256]
        ico_entries: list[tuple[int, bytes]] = []
        for size in ico_sizes:
            p = tmp_dir / f"ico_{size}.png"
            resize_png_sips(SRC_PNG, p, size)
            ico_entries.append((size, p.read_bytes()))

        for ico_dst in [
            ROOT / "res" / "icon.ico",
            ROOT / "res" / "tray-icon.ico",
            ROOT / "flutter" / "windows" / "runner" / "resources" / "app_icon.ico",
        ]:
            build_ico(ico_entries, ico_dst)

        # 3. 生成 macOS .icns
        generate_icns(SRC_PNG, ROOT / "flutter" / "macos" / "Runner" / "AppIcon.icns", tmp_dir)

    print("QNAP RustDesk 全套品牌图标已生成完成。")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

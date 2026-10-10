# Vitrunda

**A lightweight liquid-glass hidden desktop for Windows · 面向 Windows 的轻量级液态玻璃隐藏桌面**

Vitrunda is an experimental Windows desktop layer built around one idea: **the wallpaper should own the desktop until you deliberately summon your workspace**.

Vitrunda 是一个面向 Windows 的轻量级隐藏桌面工具。它的核心目标很简单：**平时让桌面只展示壁纸，需要时再从屏幕角落召唤出完整工作空间。**

## Download · 下载

The current public build is **v0.1.0-preview.1** and is published as a GitHub Pre-release.

当前公开构建版本为 **v0.1.0-preview.1**，以 GitHub Pre-release 形式发布。

- [Vitrunda.exe](https://github.com/ZJY-HSBL/Vitrunda/releases/download/v0.1.0-preview.1/Vitrunda.exe)
- [Vitrunda-windows-x64.zip](https://github.com/ZJY-HSBL/Vitrunda/releases/download/v0.1.0-preview.1/Vitrunda-windows-x64.zip)
- [SHA256SUMS.txt](https://github.com/ZJY-HSBL/Vitrunda/releases/download/v0.1.0-preview.1/SHA256SUMS.txt)
- [All Releases](https://github.com/ZJY-HSBL/Vitrunda/releases)

> Preview builds are intended for interaction and rendering validation. The Direct3D 11 backend is now initialized in the codebase, but the DirectComposition/HLSL liquid renderer is still under development.
>
> 预览版本主要用于交互与渲染链路验证。目前代码中已经接入 Direct3D 11 初始化，但 DirectComposition/HLSL 液态玻璃渲染器仍在开发中。

## Concept · 核心交互

1. The normal desktop remains clean and unobstructed. / 默认桌面保持纯净，不常驻任何面板。
2. Click the top-right hot corner. / 点击屏幕右上角热区。
3. A glass layer expands from the corner and fills the desktop with a fluid animation. / 液态玻璃层从右上角流动展开并铺满桌面。
4. Click the same corner again, or press `Esc`, to retract it. / 再次点击右上角或按 `Esc`，界面沿原路径收回。

## v0.1 Prototype

The first milestone intentionally focuses on the interaction itself rather than file-management features.

当前 `v0.1` 原型刻意只验证最核心的体验，不提前堆叠文件管理功能：

- Invisible top-right hot corner / 无视觉干扰的右上角热区
- Native borderless overlay / Windows 原生无边框覆盖层
- Fluid corner expansion / 从右上角向左下角流动展开
- Damped easing and subtle overshoot / 阻尼式缓动与轻微回弹
- Native blur + translucent glass tint / 原生模糊与半透明玻璃底色
- Reverse retract animation / 反向吸回动画
- `Esc` to close / `Esc` 收回
- JSON configuration / JSON 配置
- Per-user autostart support / 当前用户级开机自启动

> The current prototype uses native DWM blur and geometry morphing. True refraction, dynamic highlights, and shader-based liquid edges belong to the next rendering milestone.
>
> 当前原型采用 DWM 模糊与几何形变完成第一阶段视觉验证。真正的背景折射、动态高光和 Shader 液态边缘将在下一渲染阶段加入。

## Why native Windows · 为什么使用原生 Windows 技术

Vitrunda is designed as a resident desktop utility, so idle cost matters more than framework convenience.

Vitrunda 需要长期常驻，因此优先考虑待机资源占用，而不是开发框架的便利性。

- Rust main process / Rust 主程序
- Win32 windowing / Win32 窗口与热区
- DWM composition / DWM 合成
- No Electron / 不使用 Electron
- No Chromium / 不内置 Chromium
- No Python runtime / 不依赖 Python Runtime
- No local web server / 不启动本地 Web 服务
- Event-driven idle state / 待机采用事件驱动，不后台轮询

## Configuration · 配置

On first launch, Vitrunda writes:

首次启动后会生成：

```text
%LOCALAPPDATA%\Vitrunda\config.json
```

Default configuration:

```json
{
  "hot_corner_size": 14,
  "animation_ms": 420,
  "glass_alpha": 214,
  "autostart": false,
  "close_on_escape": true
}
```

Set `autostart` to `true` and launch Vitrunda once to register it under the current Windows user.

将 `autostart` 修改为 `true` 并启动一次 Vitrunda，即可写入当前 Windows 用户的开机启动项。

## Build · 构建

Requirements:

- Windows 10/11
- Rust stable

```powershell
cargo build --release
```

The release binary will be located at:

```text
target\release\vitrunda.exe
```

## Roadmap · 路线

`v0.1` — native hot corner + glass overlay + fluid geometry animation  
`v0.2` — DirectComposition rendering path + spring animation engine  
`v0.3` — HLSL refraction + dynamic edge highlight + liquid mask  
`v0.4` — desktop items, shortcuts and free layout  
`v0.5` — settings UI, multi-monitor and appearance profiles

The project deliberately grows from a working minimal product. Features that increase background cost without improving the core interaction will not be added by default.

项目坚持从最小可运行版本逐层扩展。任何会显著增加常驻资源占用、却不能改善核心交互的功能，都不会默认加入。

## License

MIT

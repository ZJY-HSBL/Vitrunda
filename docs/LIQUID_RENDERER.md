# Liquid Glass renderer — implementation contract

Vitrunda's native Win32 overlay is the working baseline. The next rendering backend must not replace it until a Windows GPU build and manual visual check succeed.

## Rendering architecture

1. **Window host** — Win32 owns focus, hot-corner hit testing, monitor bounds, and lifecycle.
2. **Composition surface** — Direct3D 11 renders into a DirectComposition-compatible swap chain. The swap chain uses premultiplied alpha so the shape can have transparent edges.
3. **Liquid mask** — a pixel shader evaluates a signed-distance field (SDF) for a rounded, deformable expanding silhouette.
4. **Glass material** — tint, edge normal estimation, and specular highlights are derived from the SDF. Background refraction is a separate optional pass, **not** something DWM blur or DirectComposition automatically provides.
5. **Animation controller** — the same normalized progress drives both the fallback native geometry and GPU mask. Direction changes must preserve progress and preferably velocity.

## Shader contract

The proposed constant buffer is:

```hlsl
cbuffer LiquidParameters : register(b0)
{
    float2 resolution;
    float2 origin;
    float progress;
    float timeSeconds;
    float cornerRadius;
    float edgeWidth;
    float refractionStrength;
    float glassOpacity;
};
```

The liquid shader outputs **premultiplied RGBA**. Its mask must be exactly empty at progress 0 and cover the intended workspace at progress 1. The hotspot remains a separate input window above the visual overlay.

## GPU implementation milestones

- Create D3D11 device and composition swap chain on Windows.
- Attach swap chain to DirectComposition visual and commit it.
- Render a full-screen triangle with a simple alpha mask.
- Replace alpha mask with an animated liquid SDF.
- Add dynamic edge highlights and optional blur.
- Add backdrop capture/refraction only after choosing a Windows-compatible, permission-conscious capture mechanism. Avoid constant screen capture while idle.

## Acceptance tests

- Window still opens and closes when GPU initialization fails (native fallback).
- Reverse transitions do not jump in size or opacity.
- No animation timer or GPU redraw while hidden.
- Correct scaling on high-DPI monitors.
- No blocking focus theft or permanent topmost full-screen window after closing.
- Windows x64 CI builds; visual quality and resource use are validated on actual Windows hardware.

The above is a design contract, not a claim that the DirectComposition/HLSL backend has already been implemented.

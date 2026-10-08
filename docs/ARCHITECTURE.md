# Architecture

Vitrunda follows an event-driven resident-tool architecture.

```text
Windows message loop
        |
        +-- invisible hot corner
        |       |
        |       +-- click -> toggle animation
        |
        +-- glass overlay
                |
                +-- hidden while idle
                +-- 8 ms timer only during transition
                +-- DWM blur + translucent tint
                +-- geometry morph from top-right corner
```

## Runtime rules

1. No polling loop is allowed in the idle path.
2. Animation timers exist only while an opening/closing transition is active.
3. The overlay is hidden after retracting.
4. The hot-corner window remains tiny, transparent and non-activating.
5. Rendering features are layered progressively; the baseline must remain usable without advanced GPU effects.

## Rendering roadmap

The prototype uses DWM blur and region morphing because this is the smallest native end-to-end path.

The intended high-quality path is:

```text
Win32
  -> DirectComposition
      -> Direct2D surface
          -> HLSL liquid mask
          -> backdrop/refraction sampling
          -> dynamic edge highlight
          -> spring-driven shape morph
```

This keeps the high-cost rendering path active only while the layer is visible or transitioning.

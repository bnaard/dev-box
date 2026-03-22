---
name: pixijs-gamedev
description: PixiJS 2D rendering and game development including sprites, animations, interactions, and WebGL/Canvas rendering. Use when building PixiJS applications, creating 2D games, or implementing interactive graphics.
allowed-tools: Bash(npm:*) Bash(npx:*) Read Write
---

# PixiJS Game Development

## When to Use

When the user is building 2D games or interactive graphics with PixiJS, asks about
sprite management, animation loops, event handling, or says "create a PixiJS app"
or "add sprites" or "animate this".

## Instructions

### 1. Application Setup

- Create an `Application` instance and append its canvas to the DOM
- Always `await app.init({ width, height, background })` before using the app
- Use `Application.resize()` or `resizeTo: window` for responsive canvas
- Prefer WebGL renderer (default); PixiJS falls back to Canvas automatically
- Set `antialias: true` for smoother edges at a slight performance cost
- Use `resolution: window.devicePixelRatio` for sharp rendering on HiDPI displays

### 2. Sprites and Textures

- Load textures via `Assets.load('path/to/image.png')` — returns a promise
- Create sprites with `Sprite.from(texture)` or `new Sprite(texture)`
- Set `anchor.set(0.5)` to center the sprite's origin for rotation and scaling
- Use `Spritesheet` for atlas-based animations — define frames in a JSON descriptor
- Avoid creating textures from the same source repeatedly; `Assets` caches automatically
- Use `Texture.EMPTY` as a placeholder while assets load

### 3. Containers and Display Hierarchy

- `Container` is the base display object — use it to group and transform children
- Children inherit parent transforms (position, scale, rotation, alpha)
- Use `sortableChildren: true` and set `zIndex` for draw-order control
- Remove off-screen objects from the stage to avoid unnecessary rendering
- Use `ParticleContainer` for thousands of identical sprites (limited features, high speed)

### 4. Animation

- Use `app.ticker.add((ticker) => { ... })` for the main game loop
- `ticker.deltaTime` gives frame-time-normalized delta (1.0 at target FPS)
- For complex tweening, integrate GSAP: `gsap.to(sprite, { x: 200, duration: 1 })`
- Use `AnimatedSprite` for frame-based animations from spritesheets
- Control playback: `animatedSprite.play()`, `.stop()`, `.gotoAndPlay(frame)`
- Set `animationSpeed` to control frame rate independently of the ticker

### 5. Interaction and Events

- Set `eventMode = 'static'` on any display object to receive pointer events
- Listen with `.on('pointerdown', handler)` — works for mouse and touch
- Use `hitArea` to define custom interaction regions (Circle, Rectangle, Polygon)
- For drag: track `pointerdown` → `globalpointermove` → `pointerup` on `app.stage`
- Set `cursor = 'pointer'` for interactive elements
- Disable interaction on non-interactive children to improve event performance

### 6. Graphics and Drawing

- `new Graphics()` for vector shapes — `rect()`, `circle()`, `moveTo()`/`lineTo()`
- Chain drawing calls: `graphics.rect(0, 0, 100, 50).fill(0xff0000).stroke(0x000000)`
- Use `GraphicsContext` to share drawing instructions across multiple Graphics objects
- For static shapes, consider rendering to a texture with `app.renderer.generateTexture()`

### 7. Filters and Effects

- Apply filters via `displayObject.filters = [new BlurFilter(4)]`
- Common filters: `BlurFilter`, `ColorMatrixFilter`, `DisplacementFilter`
- Chain multiple filters in the array — they apply in order
- Filters are GPU-intensive; limit their use on mobile or low-end devices
- Use `AlphaFilter` for fading groups without affecting individual child alpha

### 8. Asset Loading

- Use `Assets.add({ alias, src })` then `Assets.load(alias)` for named assets
- Bundle assets: `Assets.addBundle('game', { bg: 'bg.png', hero: 'hero.png' })`
- Load bundles with `await Assets.loadBundle('game')` — shows progress via callback
- Supported formats: PNG, JPG, WebP, SVG, JSON (spritesheets), MP3/OGG (via plugins)
- Preload critical assets before showing the game; lazy-load secondary assets

### 9. Performance Tips

- Use `ParticleContainer` for large numbers of simple sprites (10k+)
- Minimize filter usage; each filter causes a render-texture pass
- Pool and reuse objects instead of creating/destroying frequently
- Use texture atlases to reduce draw calls (batch rendering)
- Call `destroy()` on removed display objects to free GPU memory
- Profile with `app.ticker.FPS` and browser DevTools Performance tab
- Avoid changing `blendMode` frequently — it breaks batching

## References

- `references/api-cheatsheet.md` — Common classes, methods, and setup patterns
- [PixiJS Documentation](https://pixijs.download/release/docs/)
- [PixiJS Examples](https://pixijs.io/examples/)

## Examples

**User:** "Set up a basic PixiJS game with a moving character"
**Agent:** Creates an Application, loads a character spritesheet via Assets, creates an
AnimatedSprite, adds it to the stage, and uses `app.ticker.add()` to update position
based on keyboard input. Sets `eventMode` on the stage for input handling.

**User:** "Add particle effects when the player collects a coin"
**Agent:** Creates a ParticleContainer, spawns small sprites on collection events with
randomized velocity and alpha, updates particles in the ticker loop, and removes them
when alpha reaches zero. Pools particle sprites for reuse.

**User:** "Make the game responsive to window resize"
**Agent:** Sets `resizeTo: window` on the Application, recalculates game object positions
relative to `app.screen.width/height`, and uses a resize observer to reposition UI
elements proportionally.

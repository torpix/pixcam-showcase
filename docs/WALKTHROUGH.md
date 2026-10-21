# 72-second browser walkthrough

Original capture: `media/pixcam-aurora-72s.mp4`, 1440 × 1280 pixels, 10 fps, H.264, 72 seconds, English on-screen captions, no audio. Capture consists of 720 browser viewport frames with scripted real UI interactions, encoded by FFmpeg. No desktop, terminal, browser address bar or existing application window is recorded.

| Time | View |
|---|---|
| 0–7 s | Start screen and original synthetic Aurora plate |
| 7–14 s | Top projection and nominal geometry |
| 14–20 s | Top-face Ø6 filter |
| 20–26 s | Slot information and source |
| 26–32 s | Pocket information and annotation limits |
| 32–39 s | Five orientation/tool groups |
| 39–57 s | ISO-NC playback with distinct movement colors and limits |
| 57–64 s | Export controls and offline availability |
| 64–72 s | Parser/simulation limitations and community-review closing |

Recreate from the public repository with Playwright and FFmpeg installed:

```sh
node scripts/record.mjs /absolute/path/to/playwright/index.mjs
```

Set `CHROMIUM` to the browser executable if it is not `/usr/bin/chromium`. The recorder refuses to overwrite an existing movie. Preserve or move the prior recording before rerunning. Screenshots are independently produced by `scripts/browser-test.mjs`.

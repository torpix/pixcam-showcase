# 72-second browser walkthrough

`media/pixcam-coordinate-inspector-72s.mp4` is an original 1440 × 1280, 10 fps, H.264 recording lasting exactly 72 seconds. It contains English captions and no audio. The recorder captures 720 frames from the synthetic browser viewport only — never the desktop, terminal, browser address bar or another application window.

| Time | View |
|---|---|
| 0–6 s | pixCAM v0.2 Coordinate Inspector opening |
| 6–19 s | Front and right setup transitions; explicit frame selector |
| 19–26.5 s | Compound orientation, CAD frame and 4×4 matrix |
| 26.5–32.5 s | Selected hole position and axis |
| 32.5–38.5 s | Synthetic feature map |
| 38.5–52.5 s | Clickable NC move explanations and arc playback |
| 52.5–59 s | Second arc and complete geometric details |
| 59–65 s | Offline deterministic exports |
| 65–72 s | Limits and closing identity |

Recreate with Playwright, Chromium and FFmpeg installed:

```sh
node scripts/record.mjs /absolute/path/to/playwright/index.mjs
```

Set `CHROMIUM` if the browser executable is not `/usr/bin/chromium`. The script overwrites only its named v0.2 movie. Screenshots are generated independently by `scripts/browser-test.mjs`.

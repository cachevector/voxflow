# Marketing graphics

Still images and a Remotion promo, built from the product type, palette,
and overlay pill so the words stay exact.

| File | Size | Use |
| --- | --- | --- |
| `feature-hero.png` | 1920 x 1080 | Site, X, LinkedIn, Product Hunt cover |
| `feature-ondevice.png` | 1920 x 1080 | Privacy / on-device story |
| `feature-square.png` | 1080 x 1080 | Instagram, X post, avatar-adjacent |
| `voxflow-promo.mp4` | 1920 x 1080, 22s | Site, X, Product Hunt, launch posts |

Rebuild stills:

```bash
python3 docs/marketing/render.py
```

Rebuild the video (needs Google Chrome):

```bash
cd docs/marketing/video
npm install
npm run build
```

Preview the composition in Remotion Studio:

```bash
cd docs/marketing/video
npm start
```

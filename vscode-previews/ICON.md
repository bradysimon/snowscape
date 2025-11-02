# Extension Icon

To add an icon to your extension:

1. Create a 128x128 PNG file named `icon.png` in this directory
2. Add this to `package.json`:
   ```json
   "icon": "icon.png"
   ```

## Suggested Design

- A snowflake icon (for "Snowscape")
- With a play button overlay (for "Preview")
- Blue/white color scheme to match VS Code

You can use tools like:
- Figma
- Canva
- GIMP
- Online icon generators

Or use this simple placeholder SVG:

```svg
<svg width="128" height="128" xmlns="http://www.w3.org/2000/svg">
  <rect width="128" height="128" fill="#007ACC"/>
  <text x="64" y="70" font-family="Arial" font-size="60" fill="white" text-anchor="middle">❄</text>
  <text x="90" y="95" font-family="Arial" font-size="40" fill="white">▶</text>
</svg>
```

Convert this SVG to PNG at 128x128 and name it `icon.png`.

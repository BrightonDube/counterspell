# Counterspell palette & type v2: "Steel Night" (11 Nov 2026, 22:55 SAST)
*Brighton's look-and-feel decisions applied to Option A from the earlier Option A palette (kept for history). Contrast ratios are WCAG 2.x relative-luminance ratios that I computed on bg / surface / raised. Fonts are verified as OFL-1.1 on their official repos, 11 Nov 2026.*

## Decisions (from Brighton, 11 Nov)
1. **Base:** a steel-blue night, keeping Option A's backgrounds.
2. **Poison green:** very subtle, mostly grey-green ("moss"). The bright green is used **only for pass and success states**.
3. **Second accent:** candle amber. **Red is only for failures and errors.**
4. **Fonts:** clean modern faces, no serif headings, and Cormorant Garamond is dropped.
   - **Geist** for the UI and headings.
   - **JetBrains Mono** for code.
   - **Pirata One** only for the logo wordmark, *optional*.
5. **Motion:** cinematic on the landing page, calm in the IDE.
6. **Dark mode only at launch.** The tokens are semantic, so a light theme can be added later by swapping values only.
7. **Monaco:** a custom "Crypt" theme (below).

## Tokens (dark, launch)
| Semantic token | Hex | Contrast bg / surface / raised | Use |
|---|---|---|---|
| `--bg` | `#080d0e` | — | page (crypt) |
| `--surface` | `#0f1719` | — | panels, editor gutter |
| `--raised` | `#162226` | — | cards, menus, tabs |
| `--border` | `#23353a` | decorative | hairlines |
| `--border-strong` | `#33494f` | 2.05 / 1.91 / 1.71 (decorative) | inputs, dividers |
| `--text` (bone) | `#e6e1d3` | 14.96 / 13.90 / 12.45 ✅ AAA | body and headings |
| `--text-muted` (fog) | `#9aa9a6` | 8.01 / 7.43 / 6.66 ✅ | secondary text |
| `--text-subtle` (ash) | `#7d8d8b` | 5.64 / 5.23 / 4.69 ✅ AA | captions, placeholders |
| `--accent-moss` | `#8a9e8f` | 6.86 / 6.37 / 5.71 ✅ | **the subtle grey-green**: links, active nav, icons, selected tab underline |
| `--accent-moss-strong` | `#9db3a2` | 8.75 / 8.13 / 7.28 ✅ | moss hover |
| `--accent-moss-muted` | `#5d7265` | 3.78 / 3.51 / 3.15 (UI ≥3 ✅, not body text) | moss borders, icon strokes, chart lines |
| `--accent-moss-deep` | `#26332c` | fill | moss tinted backgrounds. Bone text on it is 10.1 ✅ |
| `--accent-candle` | `#d9a25b` | 8.63 / 8.01 / 7.18 ✅ | **second accent**: primary CTA, highlights, streak flame, focus ring. Dark ink `#080d0e` on candle is 8.63 ✅ |
| `--accent-candle-glow` | `#e8b978` | 10.84 / 10.06 / 9.02 ✅ | CTA hover |
| `--info` (steel) | `#8fb3c0` | 8.73 / 8.11 / 7.26 ✅ | info, neutral badges |
| `--success` (poison) | `#7cc458` | 9.21 / 8.55 / 7.66 ✅ | **pass/success only**: test pass, Accepted, solved check |
| `--success-strong` | `#5fae3e` | 7.08 / 6.58 / 5.90 ✅ | pass icons, progress fill |
| `--success-bg` | `#1f3a1c` | fill | pass row tint. Success text on it is 5.89 ✅ |
| `--danger` (blood) | `#e0675a` | 5.82 / 5.41 / 4.85 ✅ AA | **failures and errors only**: test fail, compile error, form errors |
| `--danger-fill` | `#c4453a` | 3.97 / 3.68 / 3.30 (icons and large text only) | fail icons, destructive button background (dark ink, large text) |
| `--danger-bg` | `#3a1714` | fill | fail row tint. Danger text on it is 4.77 ✅ |
| `--focus` | `#d9a25b` (candle) | ≥7 ✅ | 2px focus ring + 2px offset |

**What changed from Option A:**
- Poison green (`#5fae3e`/`#7cc458`) is no longer a general accent. It now means success only.
- General accent duty moves to **moss** (grey-green) and **candle** (amber).
- Red is restricted to errors and failures.
- The ghost-input card uses moss and steel (it's a *finding*, not an error). Its "got" value is in danger red and its "expected" value is in success green.

## Type
| Role | Font | Licence (verified) | Notes |
|---|---|---|---|
| UI + headings | **Geist** (Sans) | OFL-1.1 ([vercel/geist-font](https://github.com/vercel/geist-font/blob/main/LICENSE.txt); npm `geist` 1.9.2) | weights 400/500/600/700; headings 600, tracking -0.01em |
| Code, test I/O, ghost values | **JetBrains Mono** | OFL-1.1 ([JetBrains/JetBrainsMono](https://github.com/JetBrains/JetBrainsMono/blob/master/OFL.txt); `@fontsource/jetbrains-mono` 5.3.0) | ligatures off in the IDE by default (toggle) |
| Logo wordmark only (**optional**) | **Pirata One** | OFL-1.1 ([google/fonts ofl/pirataone](https://github.com/google/fonts/tree/main/ofl/pirataone)), Reserved Font Name "Pirata" | Export the wordmark as an **SVG with outlined paths** so the font isn't shipped at all. If it is shipped, don't rename modified copies "Pirata" (an OFL RFN rule) |

**Scale (rem):** 0.75 / 0.875 / 1 / 1.125 / 1.25 / 1.5 / 2 / 2.5 / 3.5 (landing hero only). Line height is 1.5 for body and 1.15 for headings.

## Motion
- **Landing (cinematic):** GSAP timelines for the moon reveal, graveyard smoke rising, a bat flight path and parallax on scroll, at 0.8–2.4 s with ease `power2.out`/`sine.inOut`.
- **IDE and dashboard (calm):** 120–200 ms fades and slides only. No looping ambient animation near the editor. The ghost-reveal card gets one 400 ms fade and blur-in. *Added 12 Nov [D 12 Nov]:* the only other IDE motion is the **shrink animation** on the ghost card (each recorded shrink step replaces the previous input with a short 150–250 ms crossfade and the removed parts fading out in ash, capped at about 3 s in total; it plays once and doesn't loop) plus the familiar's mood changes. The Bug Autopsy card uses the same moss/steel frame and red "got" / green "expected" rules as the ghost card.
- **`prefers-reduced-motion: reduce`:** turns all GSAP timelines off. Static frames are shown instead (the shrink steps become a static list).

## Light-mode readiness
Components use **semantic tokens only** (`bg-bg`, `text-text`, `text-accent-moss`, `bg-success-bg`…), never raw hex. Values live under `:root[data-theme="dark"]`, the launch default, set on `<html>`. A future `[data-theme="light"]` block only redefines the same variables.

```css
/* web/src/app.css (Tailwind 4) */
@import "tailwindcss";
@theme {
  --font-sans: "Geist", ui-sans-serif, system-ui, sans-serif;
  --font-mono: "JetBrains Mono", ui-monospace, SFMono-Regular, monospace;
  --color-bg: var(--bg);            --color-surface: var(--surface);   --color-raised: var(--raised);
  --color-border: var(--border);    --color-border-strong: var(--border-strong);
  --color-text: var(--text);        --color-text-muted: var(--text-muted); --color-text-subtle: var(--text-subtle);
  --color-accent-moss: var(--accent-moss); --color-accent-moss-strong: var(--accent-moss-strong);
  --color-accent-moss-muted: var(--accent-moss-muted); --color-accent-moss-deep: var(--accent-moss-deep);
  --color-accent-candle: var(--accent-candle); --color-accent-candle-glow: var(--accent-candle-glow);
  --color-info: var(--info);
  --color-success: var(--success);  --color-success-strong: var(--success-strong); --color-success-bg: var(--success-bg);
  --color-danger: var(--danger);    --color-danger-fill: var(--danger-fill);       --color-danger-bg: var(--danger-bg);
  --color-focus: var(--focus);
}
:root, :root[data-theme="dark"] {
  color-scheme: dark;
  --bg:#080d0e; --surface:#0f1719; --raised:#162226; --border:#23353a; --border-strong:#33494f;
  --text:#e6e1d3; --text-muted:#9aa9a6; --text-subtle:#7d8d8b;
  --accent-moss:#8a9e8f; --accent-moss-strong:#9db3a2; --accent-moss-muted:#5d7265; --accent-moss-deep:#26332c;
  --accent-candle:#d9a25b; --accent-candle-glow:#e8b978; --info:#8fb3c0;
  --success:#7cc458; --success-strong:#5fae3e; --success-bg:#1f3a1c;
  --danger:#e0675a; --danger-fill:#c4453a; --danger-bg:#3a1714;
  --focus:#d9a25b;
}
/* :root[data-theme="light"] { ... }  — later; same variable names */
```

## Monaco "Crypt" theme
```ts
// web/src/lib/editor/crypt-theme.ts
import type * as Monaco from 'monaco-editor';
export const cryptTheme: Monaco.editor.IStandaloneThemeData = {
  base: 'vs-dark',
  inherit: true,
  rules: [
    { token: '',            foreground: 'e6e1d3' },               // default text (bone)
    { token: 'comment',     foreground: '7d8d8b', fontStyle: 'italic' }, // ash
    { token: 'keyword',     foreground: 'd9a25b' },               // candle
    { token: 'string',      foreground: '9db3a2' },               // moss-strong
    { token: 'number',      foreground: 'e8b978' },               // candle-glow
    { token: 'type',        foreground: '8fb3c0' },               // steel
    { token: 'type.identifier', foreground: '8fb3c0' },
    { token: 'function',    foreground: 'e6e1d3', fontStyle: 'bold' },
    { token: 'variable',    foreground: 'e6e1d3' },
    { token: 'constant',    foreground: 'e8b978' },
    { token: 'operator',    foreground: '9aa9a6' },               // fog
    { token: 'delimiter',   foreground: '9aa9a6' },
    { token: 'tag',         foreground: '8a9e8f' },
    { token: 'regexp',      foreground: '9db3a2' },
    { token: 'invalid',     foreground: 'e0675a' },               // blood: errors only
  ],
  colors: {
    'editor.background': '#0f1719',
    'editor.foreground': '#e6e1d3',
    'editorLineNumber.foreground': '#5d7265',
    'editorLineNumber.activeForeground': '#9aa9a6',
    'editor.lineHighlightBackground': '#162226',
    'editor.selectionBackground': '#26332c',
    'editor.inactiveSelectionBackground': '#1b2a2e',
    'editorCursor.foreground': '#d9a25b',
    'editorIndentGuide.background1': '#23353a',
    'editorIndentGuide.activeBackground1': '#33494f',
    'editorBracketMatch.border': '#8a9e8f',
    'editorBracketMatch.background': '#26332c',
    'editorError.foreground': '#e0675a',
    'editorWarning.foreground': '#d9a25b',
    'editorInfo.foreground': '#8fb3c0',
    'editorGutter.background': '#0f1719',
    'editorWidget.background': '#162226',
    'editorWidget.border': '#23353a',
    'editorSuggestWidget.selectedBackground': '#26332c',
    'scrollbarSlider.background': '#23353a99',
    'scrollbarSlider.hoverBackground': '#33494fcc',
    'minimap.background': '#0f1719',
  },
};
// usage: monaco.editor.defineTheme('crypt', cryptTheme); monaco.editor.setTheme('crypt');
```
**Notes:**
- Monarch token names differ slightly per language. Check Python, JS, Java and C++ highlighting once in the IDE [U].
- `function` is ignored by some tokenizers, which is harmless.

## Open for Brighton's review
- The moss hex: `#8a9e8f` (a grey-green, as chosen) against a slightly greener `#86a38b`.
- Whether to ship Pirata One at all, or Geist 700 for the wordmark.

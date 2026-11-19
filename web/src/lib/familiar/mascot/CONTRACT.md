# Mascot component contract

`Mascot.svelte` draws the Efficiency Familiar for one measured result. It owns no mood logic: the
caller passes the measured mood and the text. The menu (mute and hide buttons) belongs to
`ide-workspace` (`Familiar.svelte`).

## Props

| Prop             | Type                                 | Who sets it                             | Meaning                                                             |
| ---------------- | ------------------------------------ | --------------------------------------- | ------------------------------------------------------------------- |
| `mood`           | `1`–`5`                              | the page, from the `familiar` event     | the measured mood                                                   |
| `reason`         | string                               | the page                                | the reason code, kept as a `data-reason` attribute                  |
| `headline`       | string                               | the page                                | the result sentence; rendered as text only                          |
| `badge`          | string                               | the page                                | short label, for example "Matches reference!"                       |
| `failed`         | boolean (default false)              | the page                                | true for a failed run; sets the danger text colour                  |
| `effect`         | `none`, `crack_burst`, `laugh_burst` | the page, from the event                | the burst the result asks for                                       |
| `live`           | boolean                              | the page                                | true only for a live result, never for a result loaded from history |
| `muted`          | boolean                              | the parent's menu                       | no GSAP, no bursts; the headline stays visible                      |
| `hidden`         | boolean                              | the parent's menu                       | shows a 16 px dot button instead of the face                        |
| `reducedMotion`  | boolean                              | the page, from `prefers-reduced-motion` | no GSAP, no bursts, no crossfade                                    |
| `seed`           | number                               | the page (the run id, hashed)           | seeds the burst particles so a replay looks the same                |
| `onToggleHidden` | function                             | the parent                              | called when the hidden dot is pressed                               |

## Events the parent must pass back

- Pressing the hidden dot calls `onToggleHidden`. The parent flips `hidden`.
- Mute and hide buttons in the parent flip `muted` and `hidden`. The mascot never changes them itself.

## Text equivalent

The mascot renders an `aria-live="polite"` line in the form
`Familiar: <mood name>. <headline>`. It stays in the DOM when the face is hidden.

## Artwork

The mascot fetches `/familiar/jack-o-lantern.svg` (the approved realistic art, 15 Nov 2026) and draws
it only when every id in `contract.ts` (`REQUIRED_IDS`) is present. That file holds five `<image>`
layers (`fam-mood-1` to `fam-mood-5`), one painted mood each, which load the WebP stills from
`/art/familiar/moods/`. If it is missing or invalid, the mascot falls back to
`/familiar/jack-o-lantern.placeholder.svg`, which is labelled as a placeholder. If both fail, the
badge and headline text remain.

## Motion

GSAP is loaded only by `timelines.ts`, only for a planned burst or a live crossfade, and never under
reduced motion or while muted. Every playback is disposed when its inputs change or the component
unmounts.

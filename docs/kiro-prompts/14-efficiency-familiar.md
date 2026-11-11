# D.14 Spec 12: `efficiency-familiar` (Feature, Requirements-First, PBT)

Paste the block below into the Kiro IDE (new spec / Plan mode as named in the title). Model and mode: see docs/kiro-playbook.md A.2.

```
Create a feature spec named efficiency-familiar (Requirements-First). Context: #[[file:docs/system-design.md]] §8b The Efficiency Familiar (moods, measured layer, speech bubble, limits, Postgres fields, API/run events), #[[file:docs/requirements.md]] FAM-1–FAM-6, #[[file:docs/palette.md]], and the jack-o'-lantern art in web/static/familiar/ (the SVG art and mood animations may already exist from the visuals phase; reuse them).
Rules: the mood is set ONLY by measured facts — correctness plus observed scaling vs reference_complexity, with confidence. Word results as observations ("looks quadratic, medium confidence"); the top mood is "Matches reference complexity", never "optimal".
Build order: (1) the measured layer (bench mode timings measured by the worker → complexity fit vs reference → mood); (2) wire the mood to the existing SVG + GSAP mascot (5 moods, laugh burst, crimson crack-and-burst; reduced motion, mute and hide); (3) the speech bubble: POST /api/runs/{id}/familiar-hint, called once per finished Run/Submit (never on keystrokes), using the hints client (gpt-6-luna, then Groq gpt-oss-20b), grounded only on the code, measured results and reference complexity, labelled "AI hint", unable to change verdict or mood, with a templated fallback; (4) [stretch] the live-typing heuristic in a web worker, shown only as a labelled guess that never changes the mood.
Properties for the fit: on synthetic n / n log n / n² timings with noise it returns the right class or "unsure", never a wrong class with high confidence.
```

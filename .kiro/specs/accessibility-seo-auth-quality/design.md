# Accessibility, SEO and Authenticated Quality Design

## Slices

1. Add a shared route metadata pattern and complete public-route metadata.
2. Correct heading, SVG name, target-size, landmark and contrast defects.
3. Derive signed-in or signed-out copy from the existing session state.
4. Consolidate expected authentication requests and suppress only expected anonymous errors.
5. Reserve stable puzzle and media dimensions to reduce layout shift.

## Testing

- Rendered document tests assert metadata and headings.
- Browser tests cover authenticated and signed-out copy.
- Axe and keyboard checks cover each affected route.
- Network evidence checks duplicate requests and expected console output.
- Lighthouse records SEO, accessibility and layout-shift deltas.

The design does not hide genuine API failures or remove visible focus treatment.

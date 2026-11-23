# Language Consistency Design

## Contract

The problem detail response remains the source of truth for workspace-selectable languages. Its `enabled_languages` value is the intersection of complete starters and enabled judge drivers. The global language response describes available drivers, not a promise that every problem supports every language.

The workspace initialises from a valid problem language, preserves code per language and refuses impossible selections. The server and judge revalidate the pair so a crafted request cannot bypass the UI.

## Testing

- Catalogue tests prove the intersection contract.
- API tests prove every advertised problem language has starter code.
- Workspace tests select each language and assert a non-empty editor.
- Sandbox smoke tests run one known-good starter per driver.
- Production verification checks a published pack through the public API and workspace.

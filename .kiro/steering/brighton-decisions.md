# Brighton's standing decisions (read before every task)

Recorded 14 Nov 2026 by Brighton.

- Design choices already in the specs, steering or docs (fonts, palette, layout, libraries named there) are approved. Use them; do not stop to ask.
- If a choice is not covered, make the most reasonable decision consistent with the specs, record it in the task notes, and continue. Stop for Brighton only on serious security, licensing-risk, cost or production-data decisions.
- Approved: `futures-util` as a direct dependency of `counterspell-server` for SSE streams (run-lifecycle T29). Add the THIRD-PARTY-NOTICES entry.
- Approved: Geist and JetBrains Mono fonts (ide-workspace T05), as in the design docs. Add the THIRD-PARTY-NOTICES entry.
- Approved: convert `packs/_test/` to the new format (grimoire-packs T29); ghost-engine is done.
- Approved (14 Nov 2026): auth T01 uses bundled webpki-roots via reqwest/rustls; allow CDLA-Permissive-2.0 in deny.toml for webpki-roots only, and add the THIRD-PARTY-NOTICES entry.
- grimoire-packs T28: Brighton reviewed the packs; his hardening brief is docs/kiro-prompts/33-starter-pack-hardening.md. Packs stay draft until he approves the result.
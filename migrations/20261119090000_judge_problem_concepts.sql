-- The submit ghost hunt (ide-workspace T25) copies the problem's concept tags
-- into a found ghost's autopsy. `problem_concepts` holds public-level data —
-- the server role already reads it — and the judge's grant matrix simply
-- predates the hunt. Additive grant only; the privilege matrix otherwise stands.
GRANT SELECT ON problem_concepts TO counterspell_judge;

-- Catalogue privileges (problem-catalogue C04, CAT-R1 to CAT-R6).
-- No new role is created. The two run-lifecycle roles are ensured with the same DO pattern.
-- Neither role may write a catalogue table, and neither may read catalogue_events.
-- Idempotent: REVOKE first, then GRANT the matrix from design.md.

DO $$
BEGIN
    CREATE ROLE counterspell_server NOLOGIN;
EXCEPTION WHEN duplicate_object OR unique_violation THEN
    NULL;
END
$$;

DO $$
BEGIN
    CREATE ROLE counterspell_judge NOLOGIN;
EXCEPTION WHEN duplicate_object OR unique_violation THEN
    NULL;
END
$$;

REVOKE ALL ON tracks, concepts, problems, problem_concepts, problem_versions,
              problem_examples, problem_starters, problem_hints,
              hidden_tests, hidden_generation, hidden_sources,
              reference_solutions, known_ghosts, catalogue_events
    FROM PUBLIC, counterspell_server, counterspell_judge;

-- Server: reads the public catalogue only. It can never read hidden data or events.
GRANT SELECT ON tracks, concepts, problems, problem_concepts, problem_versions,
                problem_examples, problem_starters, problem_hints
    TO counterspell_server;

-- Judge: reads the run inputs and the hidden data. It cannot write, and it cannot read events.
GRANT SELECT ON problems, problem_versions, problem_examples, problem_starters,
                hidden_tests, hidden_generation, hidden_sources,
                reference_solutions, known_ghosts
    TO counterspell_judge;

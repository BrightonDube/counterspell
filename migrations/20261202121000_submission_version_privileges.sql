-- Enqueue pins the published catalogue version when it inserts a submission.
-- Keep the server's column-based INSERT boundary for the judge-owned results.
GRANT INSERT (problem_version) ON submissions TO counterspell_server;

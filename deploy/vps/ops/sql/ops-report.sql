-- Ops report query (deploy-ops T17, R9.4 to R9.6). Reads only the columns granted to cs_ops_ro in
-- ops-role.sql. Prints one JSON object with counts and durations, and nothing else.
--
-- Not measured here: submissions whose verdict is output_limit, time_limit or memory_limit. Those
-- counts need the `verdict` column, which R9.4 does not grant. Brighton decides that (see
-- docs/kiro-log/deploy-ops-blockers.md). Infrastructure failures are measured through status.
SELECT row_to_json(report)
FROM (
    SELECT
        (SELECT count(*) FROM jobs WHERE state = 'queued') AS queued_jobs,
        (SELECT coalesce(extract(epoch FROM now() - min(run_after)), 0)
           FROM jobs WHERE state = 'queued') AS oldest_queued_age_seconds,
        (SELECT count(*) FROM jobs
           WHERE state = 'claimed' AND lease_expires_at < now()) AS expired_leases,
        (SELECT count(*) FROM jobs
           WHERE state = 'dead' AND updated_at > now() - interval '24 hours') AS dead_jobs_24h,
        (SELECT count(*) FROM jobs
           WHERE state = 'dead' AND updated_at > now() - interval '1 hour') AS dead_jobs_1h,
        (SELECT count(*) FROM submissions
           WHERE status = 'internal_error' AND finished_at > now() - interval '1 hour') AS internal_errors_1h,
        (SELECT coalesce(percentile_cont(0.95) WITHIN GROUP
                (ORDER BY extract(epoch FROM finished_at - created_at)), 0)
           FROM submissions WHERE finished_at > now() - interval '1 hour') AS p95_seconds_1h
) AS report;

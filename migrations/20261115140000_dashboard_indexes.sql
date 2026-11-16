-- Dashboard indexes (dashboard-fsrs T02). Only the two access paths that the run-lifecycle indexes
-- do not already cover. The user/created-time and user/problem indexes already exist, so they are
-- not repeated here. The indexes are read-only accelerators: PostgreSQL stays the source of truth.

-- Completed runs by owner, newest finish first. Serves the streak, the recent activity and the
-- weak-concept reads, which all start from one learner's finished runs.
CREATE INDEX IF NOT EXISTS submissions_dashboard_user_finished
    ON submissions (user_id, finished_at DESC, id DESC)
    WHERE status = 'completed';

-- Accepted runs by owner and problem. Serves the distinct solved set and the "not yet solved"
-- filter for suggestions. A verdict is only set on completed runs, so the partial predicate is exact.
CREATE INDEX IF NOT EXISTS submissions_dashboard_user_accepted
    ON submissions (user_id, problem_slug)
    WHERE verdict = 'accepted';

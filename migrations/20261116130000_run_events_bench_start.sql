-- Efficiency Familiar bench wiring (language-drivers T09). The bench announces its
-- sizes with `bench_start` before the points, and the event is part of the public
-- contract (design section 8b), so the run_events type check must accept it.

ALTER TABLE run_events
    DROP CONSTRAINT run_events_type_check,
    ADD CONSTRAINT run_events_type_check CHECK (type IN (
        'compiling', 'test', 'ghost_hunting', 'shrink_step', 'ghost',
        'bench_start', 'bench_point', 'complexity', 'familiar', 'restarted',
        'done', 'error'));

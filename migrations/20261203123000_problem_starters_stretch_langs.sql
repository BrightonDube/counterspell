-- language-drivers T16: the starter generator now emits Java and C++ skeletons,
-- so problem_starters must accept them.
ALTER TABLE problem_starters
    DROP CONSTRAINT problem_starters_lang_check,
    ADD CONSTRAINT problem_starters_lang_check
        CHECK (lang IN ('python', 'javascript', 'java', 'cpp'));

ALTER TABLE concept_cards
    ADD COLUMN IF NOT EXISTS stability_fast double precision;

UPDATE concept_cards
    SET stability_fast = stability
    WHERE stability_fast IS NULL;

ALTER TABLE concept_cards
    ALTER COLUMN stability_fast SET NOT NULL;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'concept_cards_stability_fast_nonnegative'
          AND conrelid = 'concept_cards'::regclass
    ) THEN
        ALTER TABLE concept_cards
            ADD CONSTRAINT concept_cards_stability_fast_nonnegative
            CHECK (stability_fast >= 0);
    END IF;
END $$;

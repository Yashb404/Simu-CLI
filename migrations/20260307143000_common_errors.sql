CREATE TABLE IF NOT EXISTS common_errors (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    demo_id UUID NOT NULL REFERENCES demos(id) ON DELETE CASCADE,
    command_text TEXT NOT NULL,
    count BIGINT NOT NULL DEFAULT 1,
    first_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (demo_id, command_text)
);

CREATE INDEX IF NOT EXISTS idx_common_errors_demo_id ON common_errors(demo_id);
CREATE INDEX IF NOT EXISTS idx_common_errors_count ON common_errors(count DESC);

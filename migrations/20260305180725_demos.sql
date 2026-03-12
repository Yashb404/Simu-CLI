-- Demos table - core entity for demo scripts
CREATE TABLE demos (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    project_id UUID REFERENCES projects(id) ON DELETE CASCADE,
    slug TEXT UNIQUE,
    title TEXT NOT NULL,
    engine_mode TEXT NOT NULL CHECK (engine_mode IN ('sequential', 'free_play')),
    theme JSONB NOT NULL,
    settings JSONB NOT NULL,
    steps JSONB NOT NULL,
    published BOOLEAN NOT NULL DEFAULT FALSE,
    version INT NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_demos_owner_id ON demos(owner_id);
CREATE INDEX idx_demos_project_id ON demos(project_id);
CREATE INDEX idx_demos_published ON demos(published);
CREATE UNIQUE INDEX idx_demos_slug ON demos(slug) WHERE slug IS NOT NULL AND published = TRUE;

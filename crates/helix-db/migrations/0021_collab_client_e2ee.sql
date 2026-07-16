-- Client-held E2EE: server stores opaque ciphertext and never decrypts.

ALTER TABLE collab.documents
    ADD COLUMN IF NOT EXISTS client_e2ee BOOLEAN NOT NULL DEFAULT false;

COMMENT ON COLUMN collab.documents.client_e2ee IS
    'When true, content is client-sealed (HC1 envelope); server is blind.';

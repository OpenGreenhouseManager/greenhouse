-- Scripting keys that devices use to call scripting_api. Keys go away with
-- their device.
CREATE TABLE device_key (
    key TEXT PRIMARY KEY,
    device_id UUID NOT NULL REFERENCES device(id) ON DELETE CASCADE
);

CREATE INDEX device_key_device_id_idx ON device_key(device_id);

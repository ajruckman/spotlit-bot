DROP TABLE watch;

CREATE TABLE watch
(
    id                INT         NOT NULL GENERATED ALWAYS AS IDENTITY,

    time_created      TIMESTAMPTZ NOT NULL,
    id_created_by     VARCHAR(64) NOT NULL,
    id_server         VARCHAR(64) NOT NULL,
    id_alert_channel  VARCHAR(64) NOT NULL,

    id_artist         VARCHAR(32) NOT NULL,

    time_last_scanned TIMESTAMPTZ NOT NULL,

    CONSTRAINT watch_pk PRIMARY KEY (id),
    CONSTRAINT watch_artist_uniq UNIQUE (id_server, id_artist)
);

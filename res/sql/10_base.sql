DROP TABLE IF EXISTS watch CASCADE;
DROP TABLE IF EXISTS artist_release CASCADE;
DROP TABLE IF EXISTS artist_release_watch_alerted CASCADE;
DROP VIEW IF EXISTS vw_all_watched_artists CASCADE;
DROP VIEW IF EXISTS vw_unalerted_watches CASCADE;

CREATE TABLE watch
(
    id                INT         NOT NULL GENERATED ALWAYS AS IDENTITY,

    time_created      TIMESTAMPTZ NOT NULL,
    id_created_by     VARCHAR(64) NOT NULL,
    id_server         VARCHAR(64) NOT NULL,
    id_alert_channel  VARCHAR(64) NOT NULL,
    market            CHAR(2)     NOT NULL,

    id_artist         VARCHAR(32) NOT NULL,

    has_initialized   BOOL        NOT NULL,
    time_last_scanned TIMESTAMPTZ NOT NULL,

    CONSTRAINT watch_pk PRIMARY KEY (id),
    CONSTRAINT watch_server_artist_uniq UNIQUE (id_server, id_artist)
);

CREATE TABLE artist_release
(
    id_release             VARCHAR(32)   NOT NULL,
    id_artist              VARCHAR(32)   NOT NULL,
    time_first_seen        TIMESTAMPTZ   NOT NULL,

    artist_ids             VARCHAR(32)[] NOT NULL,
    artist_names           TEXT[]        NOT NULL,
    album_type             VARCHAR(16)   NOT NULL,
    available_markets      CHAR(2)[]     NOT NULL,
    href                   VARCHAR(128)  NOT NULL,
    image_url              VARCHAR(128)  NOT NULL,
    name                   TEXT,
    release_date           VARCHAR(16)   NOT NULL,
    release_date_precision VARCHAR(8)    NOT NULL,

    CONSTRAINT artist_release_pk PRIMARY KEY (id_release)
);

CREATE TABLE artist_release_watch_alerted
(
    id_release VARCHAR(32) NOT NULL,
    id_watch   INT         NOT NULL,

    CONSTRAINT artist_release_watch_alerted_pk PRIMARY KEY (id_release, id_watch),
    CONSTRAINT artist_release_watch_alerted_id_release_fk FOREIGN KEY (id_release) REFERENCES artist_release (id_release),
    CONSTRAINT artist_release_watch_alerted_id_watch_fk FOREIGN KEY (id_watch) REFERENCES watch (id)
);

CREATE VIEW vw_all_watched_artists AS
SELECT DISTINCT id_artist
FROM watch;

CREATE VIEW vw_unalerted_watches AS
SELECT w.id AS id_watch,
       w.has_initialized,
       w.id_server,
       w.id_alert_channel,
       w.market,
       ar.id_release,
       ar.artist_names,
       ar.album_type,
       ar.href,
       ar.image_url,
       ar.name,
       ar.release_date
FROM watch w
INNER JOIN artist_release ar ON w.id_artist = ar.id_artist AND w.market = ANY (ar.available_markets)
LEFT JOIN artist_release_watch_alerted arwa ON w.id = arwa.id_watch AND ar.id_release = arwa.id_release
WHERE arwa.id_watch IS NULL;

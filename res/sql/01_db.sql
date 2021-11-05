CREATE USER spotlit_usr WITH ENCRYPTED PASSWORD 'YF7qEzKsByLi';

GRANT CONNECT ON DATABASE db0 TO spotlit_usr;

\connect db0

CREATE SCHEMA spotlit AUTHORIZATION spotlit_usr;

SET search_path = spotlit;
ALTER ROLE spotlit_usr SET search_path = spotlit;

GRANT CREATE ON SCHEMA spotlit TO spotlit_usr;
GRANT USAGE  ON SCHEMA spotlit TO spotlit_usr;

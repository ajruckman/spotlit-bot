use rspotify::{ClientCredsSpotify, Config, Credentials};

pub struct SpotifyClient {
    client: ClientCredsSpotify,
}

impl SpotifyClient {
    pub async fn new(id: &str, secret: &str) -> anyhow::Result<Self> {
        let mut cfg = Config::default();
        cfg.token_refreshing = true;

        let mut spotify = ClientCredsSpotify::with_config(Credentials::new(&id, &secret), cfg);
        spotify.request_token().await?;

        Ok(Self {
            client: spotify
        })
    }

    pub fn conn(&self) -> &ClientCredsSpotify {
        &self.client
    }
}

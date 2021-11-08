use rspotify::{ClientCredsSpotify, Credentials};

pub struct SpotifyClient {
    client: ClientCredsSpotify,
}

impl SpotifyClient {
    pub async fn new(id: &str, secret: &str) -> anyhow::Result<Self> {
        let mut spotify = ClientCredsSpotify::new(Credentials::new(&id, &secret));
        spotify.request_token().await?;

        Ok(Self {
            client: spotify
        })
    }

    pub fn conn(&self) -> &ClientCredsSpotify {
        &self.client
    }
}

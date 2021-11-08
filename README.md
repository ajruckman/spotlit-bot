## Spotlit <img align="left" width="75" height="75" src="https://i.imgur.com/MXMUPKp.png" alt="Spotlit avatar"/>

A privacy-respecting Discord bot to notify you about new music releases.

[Click here to invite the bot to your Discord server.](https://discord.com/oauth2/authorize?client_id=904887543093420113&scope=bot+applications.commands&permissions=18432)

### How it works

1. Copy the artist link for the artist you want to monitor; for example: https://open.spotify.com/artist/7cae9Fkz2R1NDHWtdnaE8d

2. Determine the ID of the channel you want to send alerts in. The easiest way to do this is to [enable Developer Mode in your app](https://support.discord.com/hc/en-us/articles/206346498-Where-can-I-find-my-User-Server-Message-ID-). After this, you will have the option to right click channels and "Copy ID".

4. Run the `/monitor` command. Specify the artist URL, the market, and channel ID from step 2.
   - The "market" is the 2-letter ISO alpha-2 country code for the country to check releases in.
   - Spotify shows different versions of many albums based on the user's country. This field is used to avoid sending a separate alert for each country the album is released in.
   - Usually, it is OK to just use "US", "GB", or "CA".
   - You can find all country codes [here](https://www.nationsonline.org/oneworld/country_code_list.htm).

![Example of monitor command](https://i.imgur.com/ucknElI.png)

After this, you will receive alerts like this for new releases in the target channel.

![Example of release notification](https://i.imgur.com/hwuZAkr.png)

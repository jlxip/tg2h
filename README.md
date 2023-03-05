# tg2h
This is a really simple proxy from Gemini to HTTP.

Its configuration is given via two environment variables:
- `TG2H_ADDR` contains the remote Gemini server. For example: `gemini://jlxip.net`.
- `TG2H_STYLE` contains the URL (can be relative!) to a generic stylesheet. Defaults to an empty value, so no style is used.

tg2h will send files as-is, with their given MIME type in the `Content-Type` value, for all formats except `text/gemini`, which it translates to HTML. The only thing to note is that the `<title>` is set to the first heading text (`# hello` -> `hello`) if it's the first line of the file.

Want to give it a look? [jlxip.net](https://jlxip.net) uses it.

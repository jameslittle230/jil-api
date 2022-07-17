# jil API

A grab bag API. For when you need a little data.

Figuring out how to make successful requests is an exercise left to the reader.

Currently lives at <https://api.jameslittle.me>

## Management

- `GET /`
- `GET /healthcheck`

## Guestbook

- `GET /guestbook`
- `GET /guestbook?after={uuid}`
- `GET /guestbook/{uuid}`
- `POST /guestbook`

## Other

- `GET /github/stork-stars`
- `POST /slack`

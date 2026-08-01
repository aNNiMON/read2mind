# read2mind

A self-hosted reading manager: save articles and YouTube transcriptions, take notes, track what to read, and get AI-generated summaries and mind maps for saved content.

> [!WARNING]
> **Localhost only.** Currently, there is no authorization/authentication, so anyone who can reach the server can read and modify all data. Do not expose this to the internet or a shared network unless you put it behind an access control layer.
>
> Recommended to use with an external auth layer / reverse proxy, such as:
> - [nginx](https://docs.nginx.com/nginx/admin-guide/security-controls/configuring-http-basic-authentication/) (e.g. with `auth_basic` or `auth_request`)
> - [Caddy](https://caddyproxy.com/) (e.g. with `basic_auth` or `forward_auth`)
> - [Traefik](https://traefik.io/) (e.g. with a `ForwardAuth`/`BasicAuth` middleware)
> - [Pangolin](https://docs.pangolin.net/manage/access-control/rules)

## Features

- **File-oriented**: stores content as a markdown file. Easy to backup/sync.
- **Markdown**: simple, widespread and portable.
- **Notes**: either take the notes while reading the content, or create your own notes.
- **Tasks**: put your tasks into separete task item.  
- **Tags and Statuses**: organize everything you read or want to read with tags and statuses.   
- **Optional AI features**: summary and mind map to quickly understand what the content about.

## Usage

Releases section contains ready-to-use application. All you need is to extract the archive.

By default, the application runs on localhost port 8555 without AI features. To change the port or enable AI features copy one of the example configs and edit it:

```sh
# use OpenAI (requires OPENAI_API_KEY environment variable to be exported)
cp config.toml.openai.example config.toml
# or use a local LLM (e.g. Ollama)
cp config.toml.local-llm.example config.toml
```


## Development

### Structure

- **backend/** — Rust (Axum) HTTP API and SQLite runtime storage.
- **frontend/** — TypeScript + Vue 3 web app, built with Deno.

### Backend

Requires Rust and Cargo to be installed.

```sh
cd backend
cargo run
```

### Frontend

Requires Deno to be installed.

```sh
cd frontend
deno install
deno task dev
```


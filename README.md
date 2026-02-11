# Gemini Proxy (Rust)

OpenAI-compatible proxy for Google Gemini - use your Gemini plan locally like Ollama.

## 🚀 Features

- **OpenAI-compatible API** - Use any OpenAI client library
- **OAuth Authentication** - Use your existing Google account and Gemini plan
- **Streaming Support** - Full SSE streaming support
- **Auto Project Detection** - Automatically manages Google Cloud projects
- **Rust Implementation** - Fast, safe, and efficient
- **Zero Configuration** - Works out of the box

## 📦 Installation

### From Source

```bash
git clone https://github.com/your-repo/gproxy.git
cd gproxy
./install.sh
```

Or directly via cargo:

```bash
cargo install --path .
```

## 🔑 Configuration & Setup (Optional)

### 1. OAuth Credentials

By default, the proxy uses a built-in public client ID. If you prefer to use your own, provide them via environment variables:

```bash
export GEMINI_CLIENT_ID="your-client-id.apps.googleusercontent.com"
export GEMINI_CLIENT_SECRET="your-client-secret"
```

You can create these in the [Google Cloud Console](https://console.cloud.google.com/apis/credentials).

### 2. Authenticate

```bash
gemini-proxy login
```

This will provide a URL for Google OAuth authentication. Visit it in your browser.

### 2. Start the Server

```bash
gemini-proxy start
```

The server runs on `http://localhost:3000` by default.

### 3. Use with OpenAI Client

```python
from openai import OpenAI

client = OpenAI(
    api_key="any-string",
    base_url="http://localhost:3000/v1"
)

response = client.chat.completions.create(
    model="gemini-3-pro-preview",
    messages=[{"role": "user", "content": "Hello!"}]
)

print(response.choices[0].message.content)
```

## 📚 CLI Commands

- `gemini-proxy login` - Authenticate with your Google account
- `gemini-proxy status` - Check authentication and server status
- `gemini-proxy start` - Start the proxy server
- `gemini-proxy logout` - Remove saved credentials
- `gemini-proxy set-project <projectId>` - Set a specific Google Cloud project ID

## 🌐 API Endpoints

- `GET /health` - Check server health
- `GET /v1/models` - List available Gemini models
- `POST /v1/chat/completions` - Create chat completions (OpenAI compatible)

## 🔧 Configuration

Credentials are stored in `~/.gemini-proxy/config.json`.

## 📝 License

MIT

# LLM Provider Support

ChromaAI Dev supports **ALL** LLM providers through its flexible configuration system. Provider API keys are automatically loaded from environment variables matching the pattern `{PROVIDER}_API_KEY` or `{PROVIDER}_KEY`.

## Supported Providers

### Major Foundation Model Providers

| Provider | Environment Variables | Aliases |
|----------|------------------------|---------|
| **OpenAI** | `OPENAI_API_KEY` | openai |
| **Anthropic** / Claude | `ANTHROPIC_API_KEY`, `CLAUDE_API_KEY` | anthropic, claude |
| **Google** / Gemini | `GOOGLE_API_KEY`, `GEMINI_API_KEY` | google, gemini |
| **xAI** / Grok | `XAI_API_KEY`, `GROK_API_KEY` | xai, grok |
| **Cohere** | `COHERE_API_KEY` | cohere |
| **Meta** / Llama | `META_API_KEY`, `LLAMA_API_KEY` | meta, llama |
| **Mistral AI** | `MISTRAL_API_KEY` | mistral |
| **AI21 Labs** / Jurassic | `AI21_API_KEY`, `JURASSIC_API_KEY` | ai21, jurassic |
| **Perplexity** | `PERPLEXITY_API_KEY` | perplexity |
| **DeepSeek** | `DEEPSEEK_API_KEY` | deepseek |
| **OpenCode** / Big Pickle | `OPENCODE_API_KEY`, `BIG_PICKLE_API_KEY` | opencode, big_pickle |

### Cloud Provider LLM Services

| Provider | Environment Variables | Aliases |
|----------|------------------------|---------|
| **AWS Bedrock** | `AWS_API_KEY`, `BEDROCK_API_KEY`, `AMAZON_API_KEY` | aws, bedrock, amazon |
| **Azure OpenAI** | `AZURE_API_KEY`, `AZURE_OPENAI_API_KEY` | azure, azure_openai |
| **Google Cloud Vertex AI** | `GOOGLE_CLOUD_API_KEY`, `VERTEX_API_KEY` | google_cloud, vertex |
| **Alibaba Cloud** (Qwen) | `ALIBABA_API_KEY`, `QWEN_API_KEY` | alibaba, qwen |

### Inference Platforms

| Provider | Environment Variables | Aliases |
|----------|------------------------|---------|
| **Hugging Face** | `HUGGINGFACE_API_KEY` | huggingface |
| **Together AI** | `TOGETHER_API_KEY` | together |
| **Replicate** | `REPLICATE_API_KEY` | replicate |
| **Stability AI** | `STABILITY_API_KEY` | stability |
| **Fireworks AI** | `FIREWORKS_API_KEY` | fireworks |
| **Baseten** | `BASETEN_API_KEY` | baseten |

### Chinese Providers

| Provider | Environment Variables | Aliases |
|----------|------------------------|---------|
| **Baidu** (Ernie) | `BAIDU_API_KEY`, `ERNIE_API_KEY` | baidu, ernie |
| **Zhipu AI** (ChatGLM) | `ZHIPU_API_KEY`, `CHATGLM_API_KEY` | zhipu, chatglm |
| **Moonshot AI** | `MOONSHOT_API_KEY` | moonshot |
| **01.AI** (Yi) | `01AI_API_KEY`, `YI_API_KEY` | 01ai, yi |
| **StepFun** | `STEPFUN_API_KEY` | stepfun |

## Usage

### Environment Variables

Set your API key for any provider:

```bash
# Foundation providers
export OPENAI_API_KEY=sk-...
export ANTHROPIC_API_KEY=sk-ant-...
export COHERE_API_KEY=...
export GEMINI_API_KEY=...

# Cloud providers
export AWS_API_KEY=...
export AZURE_API_KEY=...

# Inference platforms
export HUGGINGFACE_API_KEY=...
export TOGETHER_API_KEY=...

# Chinese providers
export BAIDU_API_KEY=...
export ZHIPU_API_KEY=...

# OpenCode / Big Pickle
export OPENCODE_API_KEY=...
```

### Programmatic Usage

```rust
use chroma_ai_dev::config::{get_config, set_llm_api_key_generic};

// Get API key for any provider
let config = get_config();
let openai_key = config.get_llm_api_key("openai");
let anthropic_key = config.get_llm_api_key("anthropic");

// Set API key for any provider
set_llm_api_key_generic("my_provider".to_string(), "my-key".to_string());

// List all configured providers
let providers = config.llm_providers();
for provider in providers {
    println!("Provider: {}", provider);
}
```

### Provider-specific Setters (Convenience)

```rust
use chroma_ai_dev::config::*;

// Foundation providers
set_openai_api_key("sk-...".to_string());
set_anthropic_api_key("sk-ant-...".to_string());
set_gemini_api_key("...".to_string());
set_grok_api_key("...".to_string());
set_cohere_api_key("...".to_string());

// Cloud providers
set_aws_bedrock_key("...".to_string());
set_azure_openai_key("...".to_string());

// Inference platforms
set_huggingface_api_key("...".to_string());
set_together_api_key("...".to_string());

// Chinese providers
set_baidu_api_key("...".to_string());
set_zhipu_api_key("...".to_string());

// OpenCode / Big Pickle
set_opencode_api_key("...".to_string());
```

## Adding New Providers

To add a new provider, simply set the environment variable:

```bash
export MYPROVIDER_API_KEY=your-key-here
```

Then use it:

```rust
let config = get_config();
let key = config.get_llm_api_key("myprovider");
```

The config loader automatically discovers any environment variable ending with `_API_KEY` or `_KEY`.

## Provider Aliases

Some providers have multiple names for convenience. All aliases point to the same API key:

- `claude` → `anthropic`
- `google` → `gemini`  
- `xai` → `grok`
- `meta` → `llama`
- `ai21` → `jurassic`
- `opencode` → `big_pickle`
- `aws` → `bedrock` → `amazon`
- `azure` → `azure_openai`
- `google_cloud` → `vertex`
- `alibaba` → `qwen`
- `baidu` → `ernie`
- `zhipu` → `chatglm`
- `01ai` → `yi`

## Secure Key Management

**Never hardcode API keys.** Use one of these approaches:

1. **Environment variables** (recommended for dev)
   ```bash
   export OPENAI_API_KEY=$(pass show openai/api-key)
   ```

2. **Secrets management** (production)
   ```bash
   export ANTHROPIC_API_KEY=$(vault read -field=secret kv/anthropic)
   ```

3. **Run-time loading**
   ```rust
   set_anthropic_api_key(std::env::var("ANTHROPIC_API_KEY").unwrap());
   ```
/// A collection of predefined model constants for legacy compatibility.
///
/// Rainy API v3 is catalog-driven (`/api/v1/models` and `/api/v1/models/catalog`), so
/// applications should prefer dynamic discovery over hardcoded model constants.
pub const OPENAI_GPT_4O: &str = "gpt-4o";
pub const OPENAI_GPT_5: &str = "gpt-5";
pub const OPENAI_GPT_5_PRO: &str = "gpt-5-pro";
pub const OPENAI_O3: &str = "o3";
pub const OPENAI_O4_MINI: &str = "o4-mini";

pub const GOOGLE_GEMINI_2_5_PRO: &str = "gemini-2.5-pro";
pub const GOOGLE_GEMINI_2_5_FLASH: &str = "gemini-2.5-flash";
pub const GOOGLE_GEMINI_2_5_FLASH_LITE: &str = "gemini-2.5-flash-lite";
pub const GOOGLE_GEMINI_3_PRO: &str = "gemini-3-pro-preview";
pub const GOOGLE_GEMINI_3_FLASH: &str = "gemini-3-flash-preview";
pub const GOOGLE_GEMINI_3_PRO_IMAGE: &str = "gemini-3-pro-image-preview";

pub const GROQ_LLAMA_3_1_8B_INSTANT: &str = "llama-3.1-8b-instant";
pub const GROQ_LLAMA_3_3_70B_VERSATILE: &str = "llama-3.3-70b-versatile";
pub const KIMI_K2_0925: &str = "moonshotai/kimi-k2-instruct-0905";

pub const CEREBRAS_LLAMA3_1_8B: &str = "cerebras/llama3.1-8b";

pub const ASTRONOMER_1: &str = "astronomer-1";
pub const ASTRONOMER_1_MAX: &str = "astronomer-1-max";
pub const ASTRONOMER_1_5: &str = "astronomer-1.5";
pub const ASTRONOMER_2: &str = "astronomer-2";
pub const ASTRONOMER_2_PRO: &str = "astronomer-2-pro";

#[deprecated(note = "Use OPENAI_GPT_4O instead for OpenAI compatibility")]
pub const GPT_4O: &str = "openai/gpt-4o";
#[deprecated(note = "Use OPENAI_GPT_5 instead for OpenAI compatibility")]
pub const GPT_5: &str = "openai/gpt-5";
#[deprecated(note = "Use GOOGLE_GEMINI_2_5_PRO instead for OpenAI compatibility")]
pub const GEMINI_2_5_PRO: &str = "google/gemini-2.5-pro";
#[deprecated(note = "Use GOOGLE_GEMINI_2_5_FLASH instead for OpenAI compatibility")]
pub const GEMINI_2_5_FLASH: &str = "google/gemini-2.5-flash";
#[deprecated(note = "Use GOOGLE_GEMINI_2_5_FLASH_LITE instead for OpenAI compatibility")]
pub const GEMINI_2_5_FLASH_LITE: &str = "google/gemini-2.5-flash-lite";
#[deprecated(note = "Use GROQ_LLAMA_3_1_8B_INSTANT instead for OpenAI compatibility")]
pub const LLAMA_3_1_8B_INSTANT: &str = "groq/llama-3.1-8b-instant";
#[deprecated(note = "Use CEREBRAS_LLAMA3_1_8B instead for OpenAI compatibility")]
pub const LLAMA3_1_8B: &str = "cerebras/llama3.1-8b";

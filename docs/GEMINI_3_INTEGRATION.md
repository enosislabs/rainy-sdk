# Integración de Gemini 3 Pro - Capacidades de Pensamiento Avanzado

## Resumen

Esta actualización integra soporte completo para los modelos Gemini 3 Pro y Flash de Google, incluyendo sus capacidades avanzadas de pensamiento (thinking), cadenas de razonamiento, y firmas de pensamiento (thought signatures) para preservar el contexto de razonamiento a través de múltiples turnos de conversación.

## Nuevas Características

### 🧠 Capacidades de Pensamiento

Los modelos Gemini 3 incluyen un proceso interno de "pensamiento" que mejora significativamente sus capacidades de razonamiento y planificación multi-paso, siendo altamente efectivos para tareas complejas como:

- Programación avanzada
- Matemáticas complejas
- Análisis de datos
- Razonamiento estratégico
- Resolución de problemas multi-paso

### 🔧 Nuevos Modelos Soportados

```rust
use rainy_sdk::models::model_constants::*;

// Modelos Gemini 3 con capacidades de pensamiento
const GOOGLE_GEMINI_3_PRO: &str = "gemini-3-pro-preview";
const GOOGLE_GEMINI_3_FLASH: &str = "gemini-3-flash-preview";
const GOOGLE_GEMINI_3_PRO_IMAGE: &str = "gemini-3-pro-image-preview";
```

### ⚙️ Configuración de Pensamiento

#### Niveles de Pensamiento (Gemini 3)

```rust
use rainy_sdk::models::{ThinkingConfig, ThinkingLevel};

// Para Gemini 3 Pro: "low" y "high"
let config_pro = ThinkingConfig::gemini_3(ThinkingLevel::High, true);

// Para Gemini 3 Flash: "minimal", "low", "medium", "high"
let config_flash = ThinkingConfig::gemini_3(ThinkingLevel::Medium, true);
```

#### Presupuesto de Pensamiento (Gemini 2.5)

```rust
// Para modelos Gemini 2.5
let config_2_5 = ThinkingConfig::gemini_2_5(-1, true); // Dinámico
let config_budget = ThinkingConfig::gemini_2_5(1024, true); // 1024 tokens
```

### 🔐 Firmas de Pensamiento (Thought Signatures)

Las firmas de pensamiento son representaciones encriptadas del proceso de pensamiento interno del modelo, utilizadas para preservar el contexto de razonamiento a través de interacciones multi-turno.

```rust
use rainy_sdk::models::{ContentPart, EnhancedChatMessage};

// Crear mensaje con firma de pensamiento
let message = EnhancedChatMessage::with_parts(
    MessageRole::Assistant,
    vec![
        ContentPart::text("Déjame pensar en esto sistemáticamente...")
            .as_thought(),
        ContentPart::text("La respuesta requiere considerar múltiples factores...")
            .with_thought_signature("signature_encriptada_aqui"),
    ]
);
```

## Ejemplos de Uso

### 1. Razonamiento Complejo con Alto Nivel de Pensamiento

```rust
use rainy_sdk::{RainyClient, models::*};

let client = RainyClient::with_api_key("tu-api-key")?;

let request = ChatCompletionRequest::new(
    model_constants::GOOGLE_GEMINI_3_PRO,
    vec![ChatMessage::user(
        "Analiza los impactos económicos de implementar una renta básica universal \
         en un país desarrollado. Considera efectos a corto y largo plazo."
    )]
)
.with_thinking_config(ThinkingConfig::high_reasoning())
.with_max_tokens(2000);

let response = client.create_chat_completion(request).await?;
```

### 2. Respuesta Rápida con Pensamiento Mínimo

```rust
let request = ChatCompletionRequest::new(
    model_constants::GOOGLE_GEMINI_3_FLASH,
    vec![ChatMessage::user("Lista 5 lenguajes de programación y sus casos de uso.")]
)
.with_thinking_level(ThinkingLevel::Low)
.with_include_thoughts(false);
```

### 3. Llamadas a Funciones con Firmas de Pensamiento

```rust
// Para Gemini 3, las firmas de pensamiento son OBLIGATORIAS en function calling
let tools = vec![
    Tool {
        r#type: ToolType::Function,
        function: FunctionDefinition {
            name: "obtener_clima".to_string(),
            description: Some("Obtener el clima actual de una ubicación".to_string()),
            parameters: Some(json!({
                "type": "object",
                "properties": {
                    "ubicacion": {
                        "type": "string",
                        "description": "El nombre de la ciudad"
                    }
                },
                "required": ["ubicacion"]
            })),
        },
    }
];

let request = ChatCompletionRequest::new(
    model_constants::GOOGLE_GEMINI_3_PRO,
    vec![ChatMessage::user("¿Cómo está el clima en Madrid?")]
)
.with_thinking_config(ThinkingConfig::gemini_3(ThinkingLevel::High, true))
.with_tools(tools);
```

## Validaciones y Mejores Prácticas

### Validación de Configuración

```rust
// Verificar capacidades del modelo
let request = ChatCompletionRequest::new(
    model_constants::GOOGLE_GEMINI_3_PRO,
    vec![ChatMessage::user("Mensaje de prueba")]
);

println!("Soporta pensamiento: {}", request.supports_thinking());
println!("Requiere firmas de pensamiento: {}", request.requires_thought_signatures());

// Validar configuración
match request.validate_openai_compatibility() {
    Ok(()) => println!("✅ Configuración válida"),
    Err(e) => println!("❌ Error de configuración: {}", e),
}
```

### Configuraciones Predefinidas

```rust
// Para tareas de razonamiento complejo
let config_complejo = ThinkingConfig::high_reasoning();

// Para respuestas rápidas
let config_rapido = ThinkingConfig::fast_response();

// Personalizada para Gemini 3
let config_custom = ThinkingConfig::gemini_3(ThinkingLevel::Medium, true);
```

## Consideraciones Importantes

### 🔒 Firmas de Pensamiento Obligatorias

- **Gemini 3**: Las firmas de pensamiento son **OBLIGATORIAS** para function calling
- **Gemini 2.5**: Las firmas de pensamiento son opcionales pero recomendadas
- Siempre preservar las firmas exactamente como se reciben del modelo

### 💰 Consideraciones de Costos

- El precio incluye tanto tokens de salida como tokens de pensamiento
- Los modelos generan pensamientos completos internamente, luego resúmenes
- El campo `thoughts_token_count` proporciona el conteo de tokens de pensamiento

### ⚡ Optimización de Rendimiento

- **Nivel Alto**: Para análisis estratégico, detección de vulnerabilidades
- **Nivel Bajo**: Para tareas de alto rendimiento con calidad superior a Gemini 2.5 Flash
- **Dinámico**: El modelo ajusta automáticamente según la complejidad

## Migración desde Versiones Anteriores

### Modelos Existentes

Los modelos Gemini 2.5 existentes siguen funcionando sin cambios:

```rust
// Sigue funcionando
const GOOGLE_GEMINI_2_5_PRO: &str = "gemini-2.5-pro";
const GOOGLE_GEMINI_2_5_FLASH: &str = "gemini-2.5-flash";
```

### Nuevas Capacidades Opcionales

Las capacidades de pensamiento son completamente opcionales:

```rust
// Uso básico sin cambios
let request = ChatCompletionRequest::new(
    model_constants::GOOGLE_GEMINI_3_PRO,
    vec![ChatMessage::user("Hola, ¿cómo estás?")]
);

// Con capacidades de pensamiento
let request_avanzado = request
    .with_thinking_level(ThinkingLevel::High)
    .with_include_thoughts(true);
```

## Estructura de Datos Mejorada

### Mensajes Mejorados

```rust
// Mensaje tradicional
let mensaje_simple = ChatMessage::user("Contenido del mensaje");

// Mensaje mejorado con múltiples partes
let mensaje_complejo = EnhancedChatMessage::with_parts(
    MessageRole::Assistant,
    vec![
        ContentPart::text("Parte de texto"),
        ContentPart::function_call("nombre_funcion", json!({"param": "valor"})),
        ContentPart::text("Más texto").with_thought_signature("firma_aqui"),
    ]
);
```

### Estadísticas de Uso Mejoradas

```rust
// Incluye conteo de tokens de pensamiento
pub struct EnhancedUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    pub thoughts_token_count: Option<u32>, // Nuevo campo
}
```

## Ejemplo Completo

Ver `examples/gemini_3_thinking.rs` para un ejemplo completo que demuestra:

1. Razonamiento complejo con alto nivel de pensamiento
2. Respuestas rápidas con pensamiento mínimo
3. Function calling con firmas de pensamiento
4. Formato de mensaje mejorado
5. Validación de capacidades del modelo

## Recursos Adicionales

- [Documentación oficial de Google Gemini Thinking](https://ai.google.dev/gemini-api/docs/thinking)
- [Guía de Thought Signatures](https://ai.google.dev/gemini-api/docs/thought-signatures)
- [Ejemplos de uso avanzado](examples/gemini_3_thinking.rs)

---

Esta integración mantiene total compatibilidad con versiones anteriores mientras proporciona acceso completo a las capacidades de razonamiento avanzado de Gemini 3 Pro.
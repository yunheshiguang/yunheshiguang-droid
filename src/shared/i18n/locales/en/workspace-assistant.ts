export const assistantWorkspaceEn = {
  openai: {
    label: "OpenAI-compatible API",
    description: "OpenAI, DeepSeek, Qwen, Ollama, and other services with an OpenAI-compatible API",
  },
  invalid: {
    baseUrl: "Enter an address that starts with http:// or https://",
    modelMissing: "Enter a model name",
    keyMissing: "Enter an API key (not needed for services on this computer)",
    key: "The API key can only contain letters, digits, and punctuation",
    providerUnknown: "Unknown AI provider",
    settingTooLong: "Each setting can be at most {{max}} characters",
    promptRequired: "Describe what to do",
    promptTooLong: "The description can be at most {{max}} characters",
  },
  noProvider: "No AI provider is available",
  requestFailed: "The AI service request failed: {{message}}",
  refused: "The model refused the request: {{message}}",
  emptyAnswer: "The model returned no answer",
  unreadable: "Could not parse the model's answer: {{message}}",
  storageFailed: "Could not save the AI assistant settings: {{message}}",
  draft: {
    empty: "The draft has no usable steps",
    tooLong: "The draft is too long. Only the first {{max}} steps were kept",
    unknownAction: "Step {{step}} used an unknown action and was removed",
    scriptMissing: "This step runs a script that is not in the script library",
  },
  script: {
    empty: "The model generated an empty script",
    tooLong: "The script can be at most {{max}} characters",
  },
  audit: {
    updated: "Changed the AI assistant settings",
    flow: "Drafted a flow with {{model}}",
    script: "Drafted a script with {{model}}",
  },
} as const;

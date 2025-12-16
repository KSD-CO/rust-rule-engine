use rust_rule_engine::engine::engine::RustRuleEngine;
use rust_rule_engine::engine::plugin::{PluginHealth, PluginMetadata, PluginState, RulePlugin};
use rust_rule_engine::errors::Result;
use rust_rule_engine::types::Value;

/// HTTP Client Plugin for making REST API calls from rules
pub struct HttpClientPlugin {
    metadata: PluginMetadata,
}

impl HttpClientPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                name: "http-client".to_string(),
                version: "1.0.0".to_string(),
                description: "HTTP client for REST API calls and webhooks".to_string(),
                author: "Rust Rule Engine Team".to_string(),
                state: PluginState::Loaded,
                health: PluginHealth::Healthy,
                actions: vec![
                    "HttpGet".to_string(),
                    "HttpPost".to_string(),
                    "HttpPut".to_string(),
                    "HttpDelete".to_string(),
                    "SendWebhook".to_string(),
                    "DownloadFile".to_string(),
                    "UploadFile".to_string(),
                ],
                functions: vec![
                    "apiCall".to_string(),
                    "getStatusCode".to_string(),
                    "parseJson".to_string(),
                    "setHeaders".to_string(),
                    "basicAuth".to_string(),
                ],
                dependencies: vec!["reqwest".to_string(), "serde_json".to_string()],
            },
        }
    }
}

impl RulePlugin for HttpClientPlugin {
    fn get_metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn register_actions(&self, engine: &mut RustRuleEngine) -> Result<()> {
        // HTTP GET action
        engine.register_action_handler("HttpGet", |params, facts| {
            let url = params.get("0").map(|v| v.to_string()).unwrap_or_default();
            let result_key = params
                .get("1")
                .map(|v| v.to_string())
                .unwrap_or("http_response".to_string());

            println!("🌐 HTTP GET: {}", url);
            println!("   Status: 200 OK");
            println!("   Response: {{\"success\": true, \"data\": \"Sample data\"}}");

            // Simulate successful response
            facts.add_value(
                &result_key,
                Value::String("{\"success\": true}".to_string()),
            )?;
            Ok(())
        });

        // HTTP POST action
        engine.register_action_handler("HttpPost", |params, facts| {
            let url = params.get("0").map(|v| v.to_string()).unwrap_or_default();
            let body = params
                .get("1")
                .map(|v| v.to_string())
                .unwrap_or("{}".to_string());
            let result_key = params
                .get("2")
                .map(|v| v.to_string())
                .unwrap_or("http_response".to_string());

            println!("🌐 HTTP POST: {}", url);
            println!("   Body: {}", body);
            println!("   Status: 201 Created");

            facts.add_value(
                &result_key,
                Value::String("{\"id\": 123, \"created\": true}".to_string()),
            )?;
            Ok(())
        });

        // Send Webhook action
        engine.register_action_handler("SendWebhook", |params, facts| {
            let webhook_url = params.get("0").map(|v| v.to_string()).unwrap_or_default();
            let event_type = params
                .get("1")
                .map(|v| v.to_string())
                .unwrap_or("event".to_string());
            let payload = params
                .get("2")
                .map(|v| v.to_string())
                .unwrap_or("{}".to_string());

            println!("🪝 WEBHOOK SENT:");
            println!("   URL: {}", webhook_url);
            println!("   Event: {}", event_type);
            println!("   Payload: {}", payload);
            println!("   ✅ Delivered successfully");

            facts.add_value("webhook_sent", Value::Boolean(true))?;
            Ok(())
        });

        // Download File action
        engine.register_action_handler("DownloadFile", |params, _facts| {
            let url = params.get("0").map(|v| v.to_string()).unwrap_or_default();
            let local_path = params
                .get("1")
                .map(|v| v.to_string())
                .unwrap_or("/tmp/download".to_string());

            println!("📥 FILE DOWNLOAD:");
            println!("   From: {}", url);
            println!("   To: {}", local_path);
            println!("   Size: 2.5MB");
            println!("   ✅ Download completed");

            Ok(())
        });

        Ok(())
    }

    fn register_functions(&self, engine: &mut RustRuleEngine) -> Result<()> {
        // API Call function
        engine.register_function("apiCall", |args, _facts| {
            let method = args
                .first()
                .map(|v| v.to_string())
                .unwrap_or("GET".to_string());
            let url = args.get(1).map(|v| v.to_string()).unwrap_or_default();

            println!("🔧 API Call: {} {}", method, url);

            // Simulate API response based on URL
            if url.contains("users") {
                Ok(Value::String(
                    "{\"users\": [{\"id\": 1, \"name\": \"John\"}]}".to_string(),
                ))
            } else if url.contains("orders") {
                Ok(Value::String(
                    "{\"orders\": [{\"id\": 123, \"total\": 99.99}]}".to_string(),
                ))
            } else {
                Ok(Value::String("{\"status\": \"success\"}".to_string()))
            }
        });

        // Get Status Code function
        engine.register_function("getStatusCode", |args, _facts| {
            let response = args.first().map(|v| v.to_string()).unwrap_or_default();

            // Simulate status code extraction
            if response.contains("error") {
                Ok(Value::Number(500.0))
            } else if response.contains("not found") {
                Ok(Value::Number(404.0))
            } else {
                Ok(Value::Number(200.0))
            }
        });

        // Parse JSON function
        engine.register_function("parseJson", |args, _facts| {
            let json_str = args
                .first()
                .map(|v| v.to_string())
                .unwrap_or("{}".to_string());
            let key = args
                .get(1)
                .map(|v| v.to_string())
                .unwrap_or("data".to_string());

            println!("🔍 Parsing JSON key: {}", key);

            // Simulate JSON parsing
            if key == "id" {
                Ok(Value::Number(123.0))
            } else if key == "name" {
                Ok(Value::String("John Doe".to_string()))
            } else if key == "success" {
                Ok(Value::Boolean(true))
            } else {
                Ok(Value::String("parsed_value".to_string()))
            }
        });

        Ok(())
    }

    fn unload(&mut self) -> Result<()> {
        self.metadata.state = PluginState::Unloaded;
        println!("🌐 HTTP Client Plugin unloaded");
        Ok(())
    }

    fn health_check(&mut self) -> PluginHealth {
        // Simulate health check
        PluginHealth::Healthy
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
    use rust_rule_engine::Facts;

    #[test]
    fn test_http_client_plugin() {
        let kb = KnowledgeBase::new("HttpClientTest");
        let mut engine = RustRuleEngine::new(kb);
        let facts = Facts::new();

        let plugin = HttpClientPlugin::new();

        // Test plugin registration
        assert!(plugin.register_actions(&mut engine).is_ok());
        assert!(plugin.register_functions(&mut engine).is_ok());

        // Test function availability
        assert!(engine.has_function("apiCall"));
        assert!(engine.has_function("parseJson"));

        // Test action availability
        assert!(engine.has_action_handler("HttpGet"));
        assert!(engine.has_action_handler("SendWebhook"));
    }
}

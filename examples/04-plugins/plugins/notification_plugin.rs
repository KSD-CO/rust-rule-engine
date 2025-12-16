use rust_rule_engine::engine::engine::RustRuleEngine;
use rust_rule_engine::engine::plugin::{PluginHealth, PluginMetadata, PluginState, RulePlugin};
use rust_rule_engine::errors::Result;
use rust_rule_engine::types::Value;

/// Notification Plugin for sending alerts, emails, SMS, and push notifications
pub struct NotificationPlugin {
    metadata: PluginMetadata,
}

impl NotificationPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                name: "notification".to_string(),
                version: "1.0.0".to_string(),
                description: "Multi-channel notification system for alerts and communications"
                    .to_string(),
                author: "Rust Rule Engine Team".to_string(),
                state: PluginState::Loaded,
                health: PluginHealth::Healthy,
                actions: vec![
                    "SendEmail".to_string(),
                    "SendSMS".to_string(),
                    "SendPushNotification".to_string(),
                    "SendSlackMessage".to_string(),
                    "SendDiscordMessage".to_string(),
                    "CreateAlert".to_string(),
                    "SendWebhookNotification".to_string(),
                    "ScheduleNotification".to_string(),
                    "BroadcastMessage".to_string(),
                ],
                functions: vec![
                    "formatMessage".to_string(),
                    "validateEmail".to_string(),
                    "validatePhoneNumber".to_string(),
                    "generateTemplate".to_string(),
                    "calculateDeliveryTime".to_string(),
                    "getDeliveryStatus".to_string(),
                    "createNotificationId".to_string(),
                ],
                dependencies: vec![
                    "lettre".to_string(),
                    "reqwest".to_string(),
                    "uuid".to_string(),
                    "chrono".to_string(),
                ],
            },
        }
    }
}

impl RulePlugin for NotificationPlugin {
    fn get_metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn register_actions(&self, engine: &mut RustRuleEngine) -> Result<()> {
        // Send Email action
        engine.register_action_handler("SendEmail", |params, facts| {
            let to = params.get("0").map(|v| v.to_string()).unwrap_or_default();
            let subject = params
                .get("1")
                .map(|v| v.to_string())
                .unwrap_or("Notification".to_string());
            let body = params.get("2").map(|v| v.to_string()).unwrap_or_default();
            let template = params
                .get("3")
                .map(|v| v.to_string())
                .unwrap_or("default".to_string());

            println!("📧 SENDING EMAIL:");
            println!("   To: {}", to);
            println!("   Subject: {}", subject);
            println!("   Template: {}", template);
            println!(
                "   Body preview: {}...",
                if body.len() > 50 { &body[..50] } else { &body }
            );

            // Simulate email delivery
            let message_id = format!(
                "email_{}",
                (std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    % 10000)
            );

            println!("   ✅ Email sent successfully");
            println!("   📬 Message ID: {}", message_id);

            facts.add_value("email_sent", Value::Boolean(true))?;
            facts.add_value("email_message_id", Value::String(message_id))?;
            facts.add_value("delivery_status", Value::String("delivered".to_string()))?;
            Ok(())
        });

        // Send SMS action
        engine.register_action_handler("SendSMS", |params, facts| {
            let phone = params.get("0").map(|v| v.to_string()).unwrap_or_default();
            let message = params.get("1").map(|v| v.to_string()).unwrap_or_default();
            let sender_id = params
                .get("2")
                .map(|v| v.to_string())
                .unwrap_or("SYSTEM".to_string());

            println!("📱 SENDING SMS:");
            println!("   To: {}", phone);
            println!("   From: {}", sender_id);
            println!("   Message: {}", message);
            println!("   Length: {} characters", message.len());

            // Check message length
            if message.len() > 160 {
                println!("   ⚠️  Message over 160 chars - will be split into multiple SMS");
                facts.add_value("sms_parts", Value::Number((message.len() / 160 + 1) as f64))?;
            } else {
                facts.add_value("sms_parts", Value::Number(1.0))?;
            }

            let sms_id = format!(
                "sms_{}",
                (std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    % 10000)
            );

            println!("   ✅ SMS sent successfully");
            println!("   📱 Message ID: {}", sms_id);

            facts.add_value("sms_sent", Value::Boolean(true))?;
            facts.add_value("sms_message_id", Value::String(sms_id))?;
            Ok(())
        });

        // Send Push Notification action
        engine.register_action_handler("SendPushNotification", |params, facts| {
            let device_token = params.get("0").map(|v| v.to_string()).unwrap_or_default();
            let title = params
                .get("1")
                .map(|v| v.to_string())
                .unwrap_or("Notification".to_string());
            let body = params.get("2").map(|v| v.to_string()).unwrap_or_default();
            let platform = params
                .get("3")
                .map(|v| v.to_string())
                .unwrap_or("ios".to_string());

            println!("📲 SENDING PUSH NOTIFICATION:");
            println!("   Platform: {}", platform.to_uppercase());
            println!(
                "   Device: {}...{}",
                &device_token[..8],
                &device_token[device_token.len() - 8..]
            );
            println!("   Title: {}", title);
            println!("   Body: {}", body);

            // Simulate platform-specific delivery
            let service = match platform.as_str() {
                "ios" => "Apple Push Notification Service (APNs)",
                "android" => "Firebase Cloud Messaging (FCM)",
                _ => "Generic Push Service",
            };

            println!("   🚀 Via: {}", service);

            let notification_id = format!(
                "push_{}",
                (std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    % 10000)
            );

            println!("   ✅ Push notification sent");
            println!("   📱 Notification ID: {}", notification_id);

            facts.add_value("push_sent", Value::Boolean(true))?;
            facts.add_value("push_notification_id", Value::String(notification_id))?;
            facts.add_value("push_platform", Value::String(platform))?;
            Ok(())
        });

        // Send Slack Message action
        engine.register_action_handler("SendSlackMessage", |params, facts| {
            let channel = params
                .get("0")
                .map(|v| v.to_string())
                .unwrap_or("#general".to_string());
            let message = params.get("1").map(|v| v.to_string()).unwrap_or_default();
            let username = params
                .get("2")
                .map(|v| v.to_string())
                .unwrap_or("Rule Engine Bot".to_string());
            let emoji = params
                .get("3")
                .map(|v| v.to_string())
                .unwrap_or(":robot_face:".to_string());

            println!("💬 SENDING SLACK MESSAGE:");
            println!("   Channel: {}", channel);
            println!("   Username: {} {}", username, emoji);
            println!("   Message: {}", message);

            // Simulate message formatting
            let formatted_message = if message.contains("alert") || message.contains("error") {
                format!("🚨 *ALERT*: {}", message)
            } else if message.contains("success") || message.contains("completed") {
                format!("✅ *SUCCESS*: {}", message)
            } else {
                format!("ℹ️ {}", message)
            };

            println!("   📝 Formatted: {}", formatted_message);

            let slack_ts = format!(
                "1709{}",
                (std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    % 100000)
            );

            println!("   ✅ Message posted to Slack");
            println!("   💬 Timestamp: {}", slack_ts);

            facts.add_value("slack_sent", Value::Boolean(true))?;
            facts.add_value("slack_timestamp", Value::String(slack_ts))?;
            facts.add_value("slack_channel", Value::String(channel))?;
            Ok(())
        });

        // Create Alert action
        engine.register_action_handler("CreateAlert", |params, facts| {
            let alert_type = params
                .get("0")
                .map(|v| v.to_string())
                .unwrap_or("info".to_string());
            let title = params
                .get("1")
                .map(|v| v.to_string())
                .unwrap_or("System Alert".to_string());
            let description = params.get("2").map(|v| v.to_string()).unwrap_or_default();
            let severity = params
                .get("3")
                .map(|v| v.to_string())
                .unwrap_or("medium".to_string());

            println!("🚨 CREATING ALERT:");
            println!("   Type: {}", alert_type.to_uppercase());
            println!("   Severity: {}", severity.to_uppercase());
            println!("   Title: {}", title);
            println!("   Description: {}", description);

            // Determine alert priority and channels
            let (priority, channels) = match severity.as_str() {
                "critical" => (1, "email,sms,slack,push"),
                "high" => (2, "email,slack,push"),
                "medium" => (3, "email,slack"),
                "low" => (4, "slack"),
                _ => (3, "email"),
            };

            println!(
                "   🎯 Priority: {} ({})",
                priority,
                match priority {
                    1 => "CRITICAL - Immediate action required",
                    2 => "HIGH - Response within 1 hour",
                    3 => "MEDIUM - Response within 4 hours",
                    4 => "LOW - Response within 24 hours",
                    _ => "NORMAL",
                }
            );
            println!("   📡 Notification channels: {}", channels);

            let alert_id = format!(
                "alert_{}",
                (std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    % 10000)
            );

            println!("   ✅ Alert created with ID: {}", alert_id);

            facts.add_value("alert_created", Value::Boolean(true))?;
            facts.add_value("alert_id", Value::String(alert_id))?;
            facts.add_value("alert_priority", Value::Number(priority as f64))?;
            facts.add_value("notification_channels", Value::String(channels.to_string()))?;
            Ok(())
        });

        // Schedule Notification action
        engine.register_action_handler("ScheduleNotification", |params, facts| {
            let notification_type = params
                .get("0")
                .map(|v| v.to_string())
                .unwrap_or("email".to_string());
            let schedule_time = params
                .get("1")
                .map(|v| v.to_string())
                .unwrap_or("now".to_string());
            let recipient = params.get("2").map(|v| v.to_string()).unwrap_or_default();
            let message = params.get("3").map(|v| v.to_string()).unwrap_or_default();

            println!("⏰ SCHEDULING NOTIFICATION:");
            println!("   Type: {}", notification_type.to_uppercase());
            println!("   Scheduled for: {}", schedule_time);
            println!("   Recipient: {}", recipient);
            println!("   Message: {}", message);

            // Calculate delivery time
            let delivery_time = if schedule_time == "now" {
                "Immediate delivery".to_string()
            } else if schedule_time.contains("minutes") {
                format!("Delivery in {}", schedule_time)
            } else if schedule_time.contains("hours") {
                format!("Delivery in {}", schedule_time)
            } else {
                format!("Scheduled for: {}", schedule_time)
            };

            println!("   📅 {}", delivery_time);

            let job_id = format!(
                "job_{}",
                (std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    % 10000)
            );

            println!("   ✅ Notification scheduled");
            println!("   🆔 Job ID: {}", job_id);

            facts.add_value("notification_scheduled", Value::Boolean(true))?;
            facts.add_value("schedule_job_id", Value::String(job_id))?;
            facts.add_value("delivery_time", Value::String(delivery_time))?;
            Ok(())
        });

        Ok(())
    }

    fn register_functions(&self, engine: &mut RustRuleEngine) -> Result<()> {
        // Format Message function
        engine.register_function("formatMessage", |args, _facts| {
            let template = args
                .first()
                .map(|v| v.to_string())
                .unwrap_or("default".to_string());
            let data = args
                .get(1)
                .map(|v| v.to_string())
                .unwrap_or("{}".to_string());

            // Simulate template formatting
            let formatted = match template.as_str() {
                "alert" => format!("🚨 ALERT: {}", data),
                "welcome" => format!("👋 Welcome! {}", data),
                "reminder" => format!("⏰ Reminder: {}", data),
                "success" => format!("✅ Success: {}", data),
                "error" => format!("❌ Error: {}", data),
                _ => format!("📧 {}", data),
            };

            println!("🎨 Message formatted with template: {}", template);

            Ok(Value::String(formatted))
        });

        // Validate Email function
        engine.register_function("validateEmail", |args, _facts| {
            let email = args.first().map(|v| v.to_string()).unwrap_or_default();

            // Simple email validation
            let is_valid = email.contains("@")
                && email.contains(".")
                && !email.starts_with("@")
                && !email.ends_with("@")
                && email.len() > 5;

            println!(
                "📧 Email validation: {} -> {}",
                email,
                if is_valid { "✅ Valid" } else { "❌ Invalid" }
            );

            Ok(Value::Boolean(is_valid))
        });

        // Validate Phone Number function
        engine.register_function("validatePhoneNumber", |args, _facts| {
            let phone = args.first().map(|v| v.to_string()).unwrap_or_default();

            // Simple phone validation
            let cleaned = phone
                .chars()
                .filter(|c| c.is_ascii_digit())
                .collect::<String>();
            let is_valid = cleaned.len() >= 10 && cleaned.len() <= 15;

            println!(
                "📱 Phone validation: {} -> {}",
                phone,
                if is_valid { "✅ Valid" } else { "❌ Invalid" }
            );

            Ok(Value::Boolean(is_valid))
        });

        // Calculate Delivery Time function
        engine.register_function("calculateDeliveryTime", |args, _facts| {
            let notification_type = args
                .first()
                .map(|v| v.to_string())
                .unwrap_or("email".to_string());
            let priority = args
                .get(1)
                .map(|v| v.to_string())
                .unwrap_or("medium".to_string());

            // Simulate delivery time calculation
            let delivery_seconds = match (notification_type.as_str(), priority.as_str()) {
                ("push", "critical") => 1,
                ("sms", "critical") => 2,
                ("email", "critical") => 5,
                ("push", _) => 3,
                ("sms", _) => 10,
                ("email", _) => 30,
                ("slack", _) => 2,
                _ => 60,
            };

            println!(
                "⏱️  Estimated delivery time: {} seconds for {} notification ({})",
                delivery_seconds, notification_type, priority
            );

            Ok(Value::Number(delivery_seconds as f64))
        });

        // Get Delivery Status function
        engine.register_function("getDeliveryStatus", |args, _facts| {
            let message_id = args.first().map(|v| v.to_string()).unwrap_or_default();

            // Simulate status check
            let status = if message_id.contains("email") {
                "delivered"
            } else if message_id.contains("sms") {
                "sent"
            } else if message_id.contains("push") {
                "delivered"
            } else {
                "pending"
            };

            println!("📊 Delivery status for {}: {}", message_id, status);

            Ok(Value::String(status.to_string()))
        });

        // Create Notification ID function
        engine.register_function("createNotificationId", |args, _facts| {
            let notification_type = args
                .first()
                .map(|v| v.to_string())
                .unwrap_or("notification".to_string());

            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let id = format!("{}_{}", notification_type, timestamp % 100000);

            println!("🆔 Generated notification ID: {}", id);

            Ok(Value::String(id))
        });

        Ok(())
    }

    fn unload(&mut self) -> Result<()> {
        self.metadata.state = PluginState::Unloaded;
        println!("📢 Notification Plugin unloaded");
        Ok(())
    }

    fn health_check(&mut self) -> PluginHealth {
        println!("🏥 Notification health check: All channels operational");
        PluginHealth::Healthy
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
    use rust_rule_engine::Facts;

    #[test]
    fn test_notification_plugin() {
        let kb = KnowledgeBase::new("NotificationTest");
        let mut engine = RustRuleEngine::new(kb);
        let facts = Facts::new();

        let plugin = NotificationPlugin::new();

        // Test plugin registration
        assert!(plugin.register_actions(&mut engine).is_ok());
        assert!(plugin.register_functions(&mut engine).is_ok());

        // Test function availability
        assert!(engine.has_function("formatMessage"));
        assert!(engine.has_function("validateEmail"));
        assert!(engine.has_function("validatePhoneNumber"));

        // Test action availability
        assert!(engine.has_action_handler("SendEmail"));
        assert!(engine.has_action_handler("SendSMS"));
        assert!(engine.has_action_handler("SendPushNotification"));
        assert!(engine.has_action_handler("CreateAlert"));
    }
}

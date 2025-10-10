#!/bin/bash

echo "🚀 Testing Real AI-Powered Rule Engine"
echo "======================================"

# Check if the server is running
echo "🔍 Checking if AI Rule Engine API is running..."
if curl -s http://localhost:3000/api/v1/health > /dev/null; then
    echo "✅ API is running"
else
    echo "❌ API is not running. Please start it first:"
    echo "   cargo run --example ai_rest_api_production"
    exit 1
fi

echo ""
echo "📋 API Documentation"
echo "===================="
curl -s http://localhost:3000/ | jq '.'

echo ""
echo "🏥 Health Check"
echo "==============="
curl -s http://localhost:3000/api/v1/health | jq '.'

echo ""
echo "🤖 Testing OpenAI Sentiment Analysis"
echo "===================================="
curl -X POST "http://localhost:3000/api/v1/rules/execute" \
  -H "Content-Type: application/json" \
  -d '{
    "facts": {
      "CustomerMessage": {
        "text": "This product is absolutely amazing! I love it so much!",
        "provider": "openai"
      }
    },
    "ai_providers": ["openai"],
    "enable_caching": true,
    "max_cost": 1.0
  }' | jq '.'

echo ""
echo "🤗 Testing Hugging Face Sentiment Analysis"
echo "=========================================="
curl -X POST "http://localhost:3000/api/v1/rules/execute" \
  -H "Content-Type: application/json" \
  -d '{
    "facts": {
      "CustomerMessage": {
        "text": "The service was terrible and I am very disappointed.",
        "provider": "huggingface"
      }
    },
    "ai_providers": ["huggingface"],
    "enable_caching": true
  }' | jq '.'

echo ""
echo "🧠 Testing Anthropic Claude Decision Making"
echo "==========================================="
curl -X POST "http://localhost:3000/api/v1/rules/execute" \
  -H "Content-Type: application/json" \
  -d '{
    "facts": {
      "Decision": {
        "question": "Should we approve this customer for premium features?",
        "context": "VIP_customer_excellent_payment_history_5_years_active"
      }
    },
    "ai_providers": ["anthropic"],
    "enable_caching": true
  }' | jq '.'

echo ""
echo "📊 AI Service Statistics"
echo "========================"
curl -s http://localhost:3000/api/v1/ai/stats | jq '.'

echo ""
echo "🔄 Testing Cache Performance (same request)"
echo "==========================================="
echo "First request (cache miss):"
time curl -X POST "http://localhost:3000/api/v1/rules/execute" \
  -H "Content-Type: application/json" \
  -d '{
    "facts": {
      "CustomerMessage": {
        "text": "Cache test message for performance",
        "provider": "openai"
      }
    }
  }' | jq '.duration_ms'

echo ""
echo "Second request (cache hit):"
time curl -X POST "http://localhost:3000/api/v1/rules/execute" \
  -H "Content-Type: application/json" \
  -d '{
    "facts": {
      "CustomerMessage": {
        "text": "Cache test message for performance",
        "provider": "openai"
      }
    }
  }' | jq '.duration_ms'

echo ""
echo "💰 Cost Analysis"
echo "==============="
curl -s http://localhost:3000/api/v1/ai/stats | jq '{
  "total_cost_estimate": .total_cost_estimate,
  "total_requests": .total_requests,
  "cost_per_request": (.total_cost_estimate / .total_requests),
  "cache_savings": {
    "cache_hit_rate": .cache_performance.cache_hit_rate,
    "estimated_cost_saved": (.total_cost_estimate * .cache_performance.cache_hit_rate)
  }
}'

echo ""
echo "🎯 Performance Summary"
echo "====================="
curl -s http://localhost:3000/api/v1/ai/stats | jq '{
  "provider_usage": .provider_usage,
  "error_rate": .error_rate,
  "cache_performance": .cache_performance
}'

echo ""
echo "✅ Real AI Integration Test Complete!"
echo "====================================="
echo ""
echo "🔧 Next Steps:"
echo "   1. Add your real API keys to environment variables"
echo "   2. Monitor costs and set appropriate limits" 
echo "   3. Implement rate limiting for production"
echo "   4. Add more sophisticated caching strategies"
echo "   5. Set up monitoring and alerting"
echo ""
echo "📚 Available Examples:"
echo "   - cargo run --example real_ai_integration"
echo "   - cargo run --example production_ai_service"
echo "   - cargo run --example ai_rest_api_production"

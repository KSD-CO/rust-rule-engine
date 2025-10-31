#!/bin/bash

echo "🚀 Starting Rust Rule Engine REST API Demo with Analytics Monitoring"
echo "=================================================="

# Start the server in background
echo "📡 Starting server..."
cargo run --example rest_api_monitoring &
SERVER_PID=$!

# Wait for server to start
echo "⏳ Waiting for server to initialize..."
sleep 5

echo ""
echo "🌐 Server Status:"
curl -s "http://localhost:3000/api/v1/health" | jq '.'

echo ""
echo "📊 Initial Analytics Dashboard:"
curl -s "http://localhost:3000/api/v1/analytics/dashboard" | jq '.overall_stats'

echo ""
echo "🔄 Executing sample rules..."

# Execute some test requests
for i in {1..5}; do
    echo "   Request $i..."
    curl -s -X POST "http://localhost:3000/api/v1/rules/execute" \
        -H "Content-Type: application/json" \
        -d '{
            "facts": {
                "Customer": {
                    "Age": 35,
                    "IsNew": false,
                    "OrderCount": 75,
                    "TotalSpent": 15000.0,
                    "YearsActive": 3,
                    "Email": "customer@example.com"
                },
                "Order": {
                    "Amount": 750.0,
                    "CustomerEmail": "customer@example.com"
                }
            }
        }' > /dev/null
    
    # Vary some data for different analytics
    curl -s -X POST "http://localhost:3000/api/v1/rules/execute" \
        -H "Content-Type: application/json" \
        -d '{
            "facts": {
                "Customer": {
                    "Age": 25,
                    "IsNew": true,
                    "OrderCount": 1,
                    "TotalSpent": 100.0,
                    "YearsActive": 0,
                    "Email": "newcustomer@example.com"
                },
                "Order": {
                    "Amount": 50.0,
                    "CustomerEmail": "newcustomer@example.com"
                }
            }
        }' > /dev/null
done

echo ""
echo "📈 Updated Analytics Dashboard:"
curl -s "http://localhost:3000/api/v1/analytics/dashboard" | jq '.'

echo ""
echo "📊 Analytics Stats:"
curl -s "http://localhost:3000/api/v1/analytics/stats" | jq '.'

echo ""
echo "🔍 Recent Activity:"
curl -s "http://localhost:3000/api/v1/analytics/recent" | jq '.'

echo ""
echo "💡 Optimization Recommendations:"
curl -s "http://localhost:3000/api/v1/analytics/recommendations" | jq '.'

echo ""
echo "📋 Full API Documentation:"
curl -s "http://localhost:3000" | jq '.'

echo ""
echo "=================================================="
echo "✅ Demo completed! Server is still running on http://localhost:3000"
echo "📊 Analytics Dashboard: http://localhost:3000/api/v1/analytics/dashboard"
echo ""
echo "Press Ctrl+C to stop the server (PID: $SERVER_PID)"
echo "Or run: kill $SERVER_PID"

wait $SERVER_PID

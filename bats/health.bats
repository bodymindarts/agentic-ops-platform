#!/usr/bin/env bats

@test "graphql health query returns ok" {
  result=$(curl -sf http://localhost:4200/graphql \
    -X POST \
    -H "Content-Type: application/json" \
    -d '{"query":"{ health { status } }"}')

  echo "Response: $result"
  [[ "$(echo "$result" | jq -r '.data.health.status')" == "ok" ]]
}

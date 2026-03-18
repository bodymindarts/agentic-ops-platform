#!/usr/bin/env bats

@test "auth/login redirects to github.com" {
  result=$(curl -s -o /dev/null -w "%{http_code}:%{redirect_url}" \
    http://localhost:4200/auth/login)

  http_code=$(echo "$result" | cut -d: -f1)
  redirect_url=$(echo "$result" | cut -d: -f2-)

  echo "HTTP code: $http_code"
  echo "Redirect URL: $redirect_url"
  [[ "$http_code" == "307" ]]
  [[ "$redirect_url" == https://github.com/login/oauth/authorize* ]]
}

@test "auth/me without session returns 401" {
  http_code=$(curl -s -o /dev/null -w "%{http_code}" \
    http://localhost:4200/auth/me)

  echo "HTTP code: $http_code"
  [[ "$http_code" == "401" ]]
}

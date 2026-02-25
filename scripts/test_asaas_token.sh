#!/usr/bin/env bash
set -euo pipefail

# Test script que usa ASAAS_TOKEN do ambiente.
# Uso:
#   source scripts/asaas_env.sh
#   ./scripts/test_asaas_token.sh

: "${ASAAS_TOKEN:?ASAAS_TOKEN não definido. Rode: source scripts/asaas_env.sh}"

echo "== Sandbox: listar webhooks =="
curl -i -s 'https://sandbox.asaas.com/api/v3/webhook/configurations' \
  -H 'Content-Type: application/json' \
  -H "access_token: $ASAAS_TOKEN"

echo "\n== Sandbox: listar pagamentos (limit=1) =="
curl -i -s 'https://sandbox.asaas.com/api/v3/payments?limit=1' \
  -H 'Content-Type: application/json' \
  -H "access_token: $ASAAS_TOKEN"

echo "\n== Produção: listar webhooks =="
curl -i -s 'https://api.asaas.com/api/v3/webhook/configurations' \
  -H 'Content-Type: application/json' \
  -H "access_token: $ASAAS_TOKEN"

# Opcional: testar usando Authorization: Bearer
echo "\n== Sandbox: listar pagamentos (Authorization header) =="
curl -i -s 'https://sandbox.asaas.com/api/v3/payments?limit=1' \
  -H 'Content-Type: application/json' \
  -H "Authorization: Bearer $ASAAS_TOKEN"

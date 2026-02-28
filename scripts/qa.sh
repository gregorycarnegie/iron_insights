#!/usr/bin/env bash
set -euo pipefail

DATA_DIR="${1:-data}"
SITE_DIR="${2:-docs}"
BASE_URL="${3:-}"
SLICE_KEY="${4:-}"

fail() {
  echo "[qa] ERROR: $*" >&2
  exit 1
}

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || fail "required command not found: $1"
}

need_cmd jq
need_cmd awk

[[ -d "$DATA_DIR" ]] || fail "data directory not found: $DATA_DIR"
LATEST_JSON="$DATA_DIR/latest.json"
[[ -s "$LATEST_JSON" ]] || fail "missing or empty latest.json: $LATEST_JSON"

VERSION="$(jq -r '.version // empty' "$LATEST_JSON")"
[[ -n "$VERSION" ]] || fail "latest.json missing .version"

VERSION_DIR="$DATA_DIR/$VERSION"
[[ -d "$VERSION_DIR" ]] || fail "version directory missing: $VERSION_DIR"

INDEX_JSON="$VERSION_DIR/index.json"
[[ -s "$INDEX_JSON" ]] || fail "missing or empty index.json: $INDEX_JSON"

ENTRY_COUNT="$(jq '.slices | length' "$INDEX_JSON")"
[[ "$ENTRY_COUNT" -gt 0 ]] || fail "index has no slices"

echo "[qa] Version: $VERSION"
echo "[qa] Slice entries: $ENTRY_COUNT"

missing=0
invalid=0
meta_total_sum=0

while IFS= read -r row_b64; do
  row_json="$(printf '%s' "$row_b64" | base64 -d)"

  key="$(printf '%s' "$row_json" | jq -r '.key')"
  meta_rel="$(printf '%s' "$row_json" | jq -r '.value.meta')"
  hist_rel="$(printf '%s' "$row_json" | jq -r '.value.hist')"
  heat_rel="$(printf '%s' "$row_json" | jq -r '.value.heat')"

  for rel in "$meta_rel" "$hist_rel" "$heat_rel"; do
    [[ "$rel" != /* ]] || {
      echo "[qa] invalid absolute path in index ($key): $rel" >&2
      invalid=$((invalid + 1))
      continue
    }
    [[ -s "$VERSION_DIR/$rel" ]] || {
      echo "[qa] missing/empty file for $key: $rel" >&2
      missing=$((missing + 1))
    }
  done

  if [[ -s "$VERSION_DIR/$meta_rel" ]]; then
    total="$(jq -r '.hist.total // 0' "$VERSION_DIR/$meta_rel")"
    bins="$(jq -r '.hist.bins // 0' "$VERSION_DIR/$meta_rel")"
    h_width="$(jq -r '.heat.width // 0' "$VERSION_DIR/$meta_rel")"
    h_height="$(jq -r '.heat.height // 0' "$VERSION_DIR/$meta_rel")"

    [[ "$bins" -ge 1 ]] || {
      echo "[qa] bad histogram bins for $key: $bins" >&2
      invalid=$((invalid + 1))
    }

    if [[ "$total" =~ ^[0-9]+$ ]]; then
      meta_total_sum=$((meta_total_sum + total))
    else
      echo "[qa] non-numeric hist.total for $key: $total" >&2
      invalid=$((invalid + 1))
    fi

    if [[ "$h_width" -eq 0 || "$h_height" -eq 0 ]]; then
      echo "[qa] warning: zero-dimension heatmap for $key (${h_width}x${h_height})"
    fi
  fi
done < <(jq -r '.slices | to_entries[] | @base64' "$INDEX_JSON")

[[ "$missing" -eq 0 ]] || fail "found $missing missing/empty referenced files"
[[ "$invalid" -eq 0 ]] || fail "found $invalid invalid slice entries"
[[ "$meta_total_sum" -gt 0 ]] || fail "aggregate hist.total is zero"

bin_bytes=0
json_bytes=0
file_count=0
while IFS= read -r -d '' f; do
  size="$(wc -c < "$f")"
  file_count=$((file_count + 1))
  if [[ "$f" == *.bin ]]; then
    bin_bytes=$((bin_bytes + size))
  else
    json_bytes=$((json_bytes + size))
  fi
done < <(find "$VERSION_DIR" -type f \( -name '*.bin' -o -name '*.json' \) -print0)

total_bytes=$((bin_bytes + json_bytes))

fmt_bytes() {
  local b="$1"
  awk -v b="$b" 'BEGIN {
    if (b >= 1024*1024*1024) { printf "%.2f GB", b/(1024*1024*1024); exit }
    if (b >= 1024*1024) { printf "%.2f MB", b/(1024*1024); exit }
    if (b >= 1024) { printf "%.2f KB", b/1024; exit }
    printf "%d B", b
  }'
}

sample_row=""
if [[ -n "$SLICE_KEY" ]]; then
  sample_row="$(jq -rc --arg k "$SLICE_KEY" '.slices[$k] | select(.)' "$INDEX_JSON")"
  if [[ -z "$sample_row" ]]; then
    echo "[qa] warning: requested slice key not found, using first slice."
  fi
fi
if [[ -z "$sample_row" ]]; then
  sample_row="$(jq -rc '.slices | to_entries[0] | {key: .key, value: .value}' "$INDEX_JSON")"
else
  sample_row="$(jq -rc --arg k "$SLICE_KEY" '.slices | {key: $k, value: .[$k]}' "$INDEX_JSON")"
fi

sample_name="$(printf '%s' "$sample_row" | jq -r '.key')"
sample_meta_rel="$(printf '%s' "$sample_row" | jq -r '.value.meta')"
sample_hist_rel="$(printf '%s' "$sample_row" | jq -r '.value.hist')"
sample_heat_rel="$(printf '%s' "$sample_row" | jq -r '.value.heat')"

latest_bytes="$(wc -c < "$LATEST_JSON")"
index_bytes="$(wc -c < "$INDEX_JSON")"
sample_meta_bytes="$(wc -c < "$VERSION_DIR/$sample_meta_rel")"
sample_hist_bytes="$(wc -c < "$VERSION_DIR/$sample_hist_rel")"
sample_heat_bytes="$(wc -c < "$VERSION_DIR/$sample_heat_rel")"
sample_data_bytes=$((latest_bytes + index_bytes + sample_meta_bytes + sample_hist_bytes + sample_heat_bytes))

site_budget_bytes=0
if [[ -d "$SITE_DIR" ]]; then
  while IFS= read -r -d '' sf; do
    site_budget_bytes=$((site_budget_bytes + $(wc -c < "$sf")))
  done < <(find "$SITE_DIR" -type f \( -name '*.html' -o -name '*.css' -o -name '*.js' -o -name '*.wasm' \) -print0)
else
  echo "[qa] SiteDir not found ($SITE_DIR), skipping static payload summary."
fi

first_view_bytes=$((site_budget_bytes + sample_data_bytes))

echo "[qa] Aggregate hist.total sum: $meta_total_sum"
echo "[qa] Files checked: $file_count"
echo "[qa] Data payload: total=$(fmt_bytes "$total_bytes") (bin=$(fmt_bytes "$bin_bytes"), json=$(fmt_bytes "$json_bytes"))"
echo "[qa] Sample slice: $sample_name"
echo "[qa] Sample data request budget: $(fmt_bytes "$sample_data_bytes") (latest+index+meta+hist+heat)"
if [[ "$site_budget_bytes" -gt 0 ]]; then
  echo "[qa] Site static payload (.html/.css/.js/.wasm): $(fmt_bytes "$site_budget_bytes")"
fi
if [[ "$first_view_bytes" -gt 0 ]]; then
  echo "[qa] Estimated first-view payload: $(fmt_bytes "$first_view_bytes")"
fi

if [[ -n "$BASE_URL" ]]; then
  need_cmd curl
  base="${BASE_URL%/}"
  echo "[qa] URL timing probe:"
  urls=(
    "$base/data/latest.json"
    "$base/data/$VERSION/index.json"
    "$base/data/$VERSION/$sample_meta_rel"
    "$base/data/$VERSION/$sample_hist_rel"
    "$base/data/$VERSION/$sample_heat_rel"
  )
  for u in "${urls[@]}"; do
    line="$(curl -L -sS -o /dev/null -w '%{http_code} %{time_total} %{size_download}' "$u" || true)"
    code="$(printf '%s' "$line" | awk '{print $1}')"
    time_s="$(printf '%s' "$line" | awk '{print $2}')"
    size_b="$(printf '%s' "$line" | awk '{print $3}')"
    if [[ -z "$code" || "$code" == "000" ]]; then
      echo "[qa]  FAIL   -- ms       --  $u"
    else
      time_ms="$(awk -v t="$time_s" 'BEGIN { printf "%.0f", t*1000 }')"
      echo "[qa]  $code  ${time_ms}ms  $(fmt_bytes "$size_b")  $u"
    fi
  done
fi

echo "[qa] OK"

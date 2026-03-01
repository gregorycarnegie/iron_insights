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

mode="$(jq -r 'if has("slices") then "flat" elif has("shards") then "sharded" else "unknown" end' "$INDEX_JSON")"
[[ "$mode" != "unknown" ]] || fail "index.json missing .slices or .shards"

slug() {
  printf '%s' "$1" | tr '[:upper:]' '[:lower:]' | sed -E 's/[^a-z0-9-]/_/g'
}

lift_name_from_code() {
  case "$1" in
    S) echo "squat" ;;
    B) echo "bench" ;;
    D) echo "deadlift" ;;
    T) echo "total" ;;
    *) return 1 ;;
  esac
}

field_from_key() {
  local key="$1"
  local name="$2"
  awk -F'=' -v n="$name" '
    {
      for (i=1; i<=NF; i++) {
        if (i % 2 == 1 && $i == n) { print $(i+1); exit }
      }
    }' <<<"$(printf '%s' "$key" | tr '|' '=')"
}

paths_from_key() {
  local key="$1"
  local sex equip wc age tested tested_dir lift_code lift
  sex="$(field_from_key "$key" "sex")"
  equip="$(field_from_key "$key" "equip")"
  wc="$(field_from_key "$key" "wc")"
  age="$(field_from_key "$key" "age")"
  tested="$(field_from_key "$key" "tested")"
  if [[ "${tested,,}" == "yes" ]]; then
    tested_dir="tested"
  else
    tested_dir="$(slug "$tested")"
  fi
  lift_code="$(field_from_key "$key" "lift")"
  lift="$(lift_name_from_code "$lift_code")" || return 1
  local base
  base="$(slug "$sex")/$(slug "$equip")/$(slug "$wc")/$(slug "$age")/$tested_dir/$lift"
  printf 'meta/%s.json\thist/%s.bin\theat/%s.bin' "$base" "$base" "$base"
}

index_root_bytes="$(wc -c < "$INDEX_JSON")"
index_budget_all_bytes="$index_root_bytes"
index_budget_sample_bytes="$index_root_bytes"

slice_source_files=()
slice_source_rels=()
if [[ "$mode" == "flat" ]]; then
  slice_source_files+=("$INDEX_JSON")
  slice_source_rels+=("index.json")
else
  while IFS=$'\t' read -r shard_key shard_rel; do
    [[ -n "$shard_rel" ]] || fail "empty shard path for $shard_key"
    [[ "$shard_rel" != /* ]] || fail "invalid absolute shard path for $shard_key: $shard_rel"
    shard_abs="$VERSION_DIR/$shard_rel"
    [[ -s "$shard_abs" ]] || fail "missing or empty shard file for $shard_key: $shard_rel"
    slice_source_files+=("$shard_abs")
    slice_source_rels+=("$shard_rel")
    index_budget_all_bytes=$((index_budget_all_bytes + $(wc -c < "$shard_abs")))
  done < <(jq -r '.shards | to_entries[] | "\(.key)\t\(.value)"' "$INDEX_JSON")
  [[ "${#slice_source_files[@]}" -gt 0 ]] || fail "sharded index has no shard files"
fi

entries_tsv="$(mktemp)"
trap 'rm -f "$entries_tsv"' EXIT

for i in "${!slice_source_files[@]}"; do
  src="${slice_source_files[$i]}"
  rel="${slice_source_rels[$i]}"
  slice_type="$(jq -r '.slices | type' "$src")"
  if [[ "$slice_type" == "object" ]]; then
    jq -r --arg rel "$rel" '.slices | to_entries[] | [.key, .value.meta, .value.hist, .value.heat, $rel] | @tsv' "$src" >> "$entries_tsv"
  elif [[ "$slice_type" == "array" ]]; then
    while IFS= read -r key; do
      [[ -n "$key" ]] || continue
      if ! paths="$(paths_from_key "$key")"; then
        fail "invalid compact slice key: $key"
      fi
      printf '%s\t%s\t%s\n' "$key" "$paths" "$rel" >> "$entries_tsv"
    done < <(jq -r '.slices[]' "$src")
  else
    fail "unsupported .slices type in $src: $slice_type"
  fi
done

ENTRY_COUNT="$(wc -l < "$entries_tsv" | awk '{print $1}')"
[[ "$ENTRY_COUNT" -gt 0 ]] || fail "index has no slices"

echo "[qa] Version: $VERSION"
echo "[qa] Slice entries: $ENTRY_COUNT"

missing=0
invalid=0
meta_total_sum=0

while IFS=$'\t' read -r key meta_rel hist_rel heat_rel shard_rel; do
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
done < "$entries_tsv"

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

sample_line=""
if [[ -n "$SLICE_KEY" ]]; then
  sample_line="$(awk -F'\t' -v k="$SLICE_KEY" '$1==k {print; exit}' "$entries_tsv")"
  if [[ -z "$sample_line" ]]; then
    echo "[qa] warning: requested slice key not found, using first slice."
  fi
fi
if [[ -z "$sample_line" ]]; then
  sample_line="$(awk -F'\t' '$1 ~ /^sex=F\|equip=All\|wc=[^|]+\|age=24-34\|tested=All\|lift=B$/ {print; exit}' "$entries_tsv")"
fi
if [[ -z "$sample_line" ]]; then
  sample_line="$(awk -F'\t' '$1 ~ /^sex=F\|equip=Raw\|wc=[^|]+\|age=24-34\|tested=All\|lift=B$/ {print; exit}' "$entries_tsv")"
fi
if [[ -z "$sample_line" ]]; then
  sample_line="$(head -n1 "$entries_tsv")"
fi
IFS=$'\t' read -r sample_name sample_meta_rel sample_hist_rel sample_heat_rel sample_shard_rel <<<"$sample_line"

if [[ "$mode" == "sharded" ]]; then
  [[ -n "$sample_shard_rel" ]] || fail "failed to resolve shard index for sample slice: $sample_name"
  index_budget_sample_bytes=$((index_root_bytes + $(wc -c < "$VERSION_DIR/$sample_shard_rel")))
else
  index_budget_sample_bytes="$index_root_bytes"
fi

latest_bytes="$(wc -c < "$LATEST_JSON")"
sample_meta_bytes="$(wc -c < "$VERSION_DIR/$sample_meta_rel")"
sample_hist_bytes="$(wc -c < "$VERSION_DIR/$sample_hist_rel")"
sample_heat_bytes="$(wc -c < "$VERSION_DIR/$sample_heat_rel")"
sample_data_bytes=$((latest_bytes + index_budget_sample_bytes + sample_meta_bytes + sample_hist_bytes + sample_heat_bytes))

male_probe_line="$(awk -F'\t' '$1 ~ /^sex=M\|equip=All\|wc=[^|]+\|age=24-34\|tested=All\|lift=B$/ {print; exit}' "$entries_tsv")"
if [[ -z "$male_probe_line" ]]; then
  male_probe_line="$(awk -F'\t' '$1 ~ /^sex=M\|equip=Raw\|wc=[^|]+\|age=24-34\|tested=All\|lift=B$/ {print; exit}' "$entries_tsv")"
fi
if [[ -z "$male_probe_line" ]]; then
  male_probe_line="$(awk -F'\t' '$1 ~ /^sex=M\|equip=All\|/ {print; exit}' "$entries_tsv")"
fi
if [[ -z "$male_probe_line" ]]; then
  male_probe_line="$(awk -F'\t' '$1 ~ /^sex=M\|equip=Raw\|/ {print; exit}' "$entries_tsv")"
fi
male_probe_name=""
male_probe_meta_rel=""
male_probe_hist_rel=""
male_probe_heat_rel=""
male_probe_shard_rel=""
if [[ -n "$male_probe_line" ]]; then
  IFS=$'\t' read -r male_probe_name male_probe_meta_rel male_probe_hist_rel male_probe_heat_rel male_probe_shard_rel <<<"$male_probe_line"
fi

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
if [[ "$mode" == "sharded" ]]; then
  echo "[qa] Sample data request budget: $(fmt_bytes "$sample_data_bytes") (latest+index_root+index_shard+meta+hist+heat)"
else
  echo "[qa] Sample data request budget: $(fmt_bytes "$sample_data_bytes") (latest+index+meta+hist+heat)"
fi
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
  urls=()
  labels=()
  urls+=("$base/data/latest.json")
  labels+=("base")
  urls+=("$base/data/$VERSION/index.json")
  labels+=("base")
  if [[ "$mode" == "sharded" && -n "$sample_shard_rel" ]]; then
    urls+=("$base/data/$VERSION/$sample_shard_rel")
    labels+=("sample")
  fi
  urls+=("$base/data/$VERSION/$sample_meta_rel")
  labels+=("sample")
  urls+=("$base/data/$VERSION/$sample_hist_rel")
  labels+=("sample")
  urls+=("$base/data/$VERSION/$sample_heat_rel")
  labels+=("sample")

  if [[ -n "$male_probe_name" && "$male_probe_name" != "$sample_name" ]]; then
    echo "[qa] Probe sample (M/All): $male_probe_name"
    if [[ "$mode" == "sharded" && -n "$male_probe_shard_rel" ]]; then
      urls+=("$base/data/$VERSION/$male_probe_shard_rel")
      labels+=("m_all")
    fi
    urls+=("$base/data/$VERSION/$male_probe_meta_rel")
    labels+=("m_all")
    urls+=("$base/data/$VERSION/$male_probe_hist_rel")
    labels+=("m_all")
    urls+=("$base/data/$VERSION/$male_probe_heat_rel")
    labels+=("m_all")
  fi
  for i in "${!urls[@]}"; do
    u="${urls[$i]}"
    label="${labels[$i]}"
    line="$(curl -L -sS -o /dev/null -w '%{http_code} %{time_total} %{size_download}' "$u" || true)"
    code="$(printf '%s' "$line" | awk '{print $1}')"
    time_s="$(printf '%s' "$line" | awk '{print $2}')"
    size_b="$(printf '%s' "$line" | awk '{print $3}')"
    if [[ -z "$code" || "$code" == "000" ]]; then
      echo "[qa]  [$label] FAIL   -- ms       --  $u"
    else
      time_ms="$(awk -v t="$time_s" 'BEGIN { printf "%.0f", t*1000 }')"
      echo "[qa]  [$label] $code  ${time_ms}ms  $(fmt_bytes "$size_b")  $u"
    fi
  done
fi

echo "[qa] OK"

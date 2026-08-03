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
  # Keep all TSV columns non-empty to avoid bash read collapsing empty tab fields.
  jq -r --arg rel "$rel" '
    .slices
    | to_entries[]
    | [
        .key,
        (if ((.value.bin // "") | length) > 0 then .value.bin else "-" end),
        $rel,
        (if .value.summary.total == null then "-" else (.value.summary.total | tostring) end)
      ]
    | @tsv
  ' "$src" >> "$entries_tsv"
done

ENTRY_COUNT="$(wc -l < "$entries_tsv" | awk '{print $1}')"
[[ "$ENTRY_COUNT" -gt 0 ]] || fail "index has no slices"

echo "[qa] Version: $VERSION"
echo "[qa] Slice entries: $ENTRY_COUNT"

missing=0
invalid=0
summary_total_sum=0

while IFS=$'\t' read -r key bin_rel shard_rel summary_total; do
  # Inlined slices carry no .bin file; the placeholder keeps the TSV columns
  # non-empty so bash's read does not collapse the adjacent tabs.
  [[ "$bin_rel" == "-" ]] && bin_rel=""
  [[ "$summary_total" == "-" ]] && summary_total=""

  if [[ -n "$bin_rel" ]]; then
    if [[ "$bin_rel" == /* ]]; then
      echo "[qa] invalid absolute path in index ($key): $bin_rel" >&2
      invalid=$((invalid + 1))
    elif [[ ! -s "$VERSION_DIR/$bin_rel" ]]; then
      echo "[qa] missing/empty file for $key: $bin_rel" >&2
      missing=$((missing + 1))
    fi
  fi

  if [[ "$summary_total" =~ ^[0-9]+$ ]]; then
    summary_total_sum=$((summary_total_sum + summary_total))
  elif [[ -n "$summary_total" ]]; then
    echo "[qa] non-numeric summary.total for $key: $summary_total" >&2
    invalid=$((invalid + 1))
  fi
done < "$entries_tsv"

[[ "$missing" -eq 0 ]] || fail "found $missing missing/empty referenced files"
[[ "$invalid" -eq 0 ]] || fail "found $invalid invalid slice entries"
[[ "$summary_total_sum" -gt 0 ]] || fail "aggregate summary.total is zero"

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
IFS=$'\t' read -r sample_name sample_bin_rel sample_shard_rel sample_summary_total <<<"$sample_line"
[[ "$sample_bin_rel" == "-" ]] && sample_bin_rel=""
[[ "$sample_summary_total" == "-" ]] && sample_summary_total=""

if [[ "$mode" == "sharded" ]]; then
  [[ -n "$sample_shard_rel" ]] || fail "failed to resolve shard index for sample slice: $sample_name"
  index_budget_sample_bytes=$((index_root_bytes + $(wc -c < "$VERSION_DIR/$sample_shard_rel")))
else
  index_budget_sample_bytes="$index_root_bytes"
fi

latest_bytes="$(wc -c < "$LATEST_JSON")"
if [[ -n "$sample_bin_rel" && -s "$VERSION_DIR/$sample_bin_rel" ]]; then
  sample_bin_bytes="$(wc -c < "$VERSION_DIR/$sample_bin_rel")"
else
  # Inlined slice: the payload is already counted inside the index shard.
  sample_bin_bytes=0
fi
sample_data_bytes=$((latest_bytes + index_budget_sample_bytes + sample_bin_bytes))

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
male_probe_bin_rel=""
male_probe_shard_rel=""
if [[ -n "$male_probe_line" ]]; then
  IFS=$'\t' read -r male_probe_name male_probe_bin_rel male_probe_shard_rel male_probe_summary_total <<<"$male_probe_line"
  [[ "$male_probe_bin_rel" == "-" ]] && male_probe_bin_rel=""
  [[ "$male_probe_summary_total" == "-" ]] && male_probe_summary_total=""
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

echo "[qa] Aggregate summary.total sum: $summary_total_sum"
echo "[qa] Files checked: $file_count"
echo "[qa] Data payload: total=$(fmt_bytes "$total_bytes") (bin=$(fmt_bytes "$bin_bytes"), json=$(fmt_bytes "$json_bytes"))"
echo "[qa] Sample slice: $sample_name"
if [[ "$mode" == "sharded" ]]; then
  echo "[qa] Sample data request budget: $(fmt_bytes "$sample_data_bytes") (latest+index_root+index_shard+bin)"
else
  echo "[qa] Sample data request budget: $(fmt_bytes "$sample_data_bytes") (latest+index+bin)"
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
  if [[ -n "$sample_bin_rel" ]]; then
    urls+=("$base/data/$VERSION/$sample_bin_rel")
    labels+=("sample")
  fi

  if [[ -n "$male_probe_name" && "$male_probe_name" != "$sample_name" ]]; then
    echo "[qa] Probe sample (M/All): $male_probe_name"
    if [[ "$mode" == "sharded" && -n "$male_probe_shard_rel" ]]; then
      urls+=("$base/data/$VERSION/$male_probe_shard_rel")
      labels+=("m_all")
    fi
    if [[ -n "$male_probe_bin_rel" ]]; then
      urls+=("$base/data/$VERSION/$male_probe_bin_rel")
      labels+=("m_all")
    fi
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

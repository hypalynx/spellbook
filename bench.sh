#!/bin/bash

# Benchmark llama-server on localhost:7777
# Usage: ./bench.sh [num_runs]

RUNS="${1:-5}"
PROMPT="The future of artificial intelligence is"

echo "Running $RUNS benchmarks..."
echo ""

declare -a tps_list

for i in $(seq 1 "$RUNS"); do
    echo -n "Run $i/$RUNS ... "

    response=$(curl -s -w "\n%{time_total}" \
        -X POST "http://localhost:7777/v1/completions" \
        -H "Content-Type: application/json" \
        -d "{\"prompt\": \"$PROMPT\", \"n_predict\": 512, \"stream\": false}")

    elapsed=$(echo "$response" | tail -1)
    json=$(echo "$response" | head -n -1)

    tokens=$(echo "$json" | grep -o '"completion_tokens":[0-9]*' | cut -d: -f2)
    [ -z "$tokens" ] && tokens=$(echo "$json" | grep -o '"tokens_predicted":[0-9]*' | cut -d: -f2)

    tps=$(echo "scale=2; $tokens / $elapsed" | bc)
    tps_list+=("$tps")

    echo "$tokens tokens in ${elapsed}s = ${tps} t/s"
done

echo ""
avg=$(echo "scale=2; ($(IFS=+; echo "${tps_list[*]}" | sed 's/^/(/')) ) / $RUNS" | bc)
min=$(echo "${tps_list[@]}" | tr ' ' '\n' | sort -n | head -1)
max=$(echo "${tps_list[@]}" | tr ' ' '\n' | sort -n | tail -1)

echo "Average: ${avg} t/s (min: ${min}, max: ${max})"

#!/bin/bash

mkdir -p out
cat list.txt | while read store_name; do
    out_path="out/${store_name}.json"
    echo "WORKING ON STORE: ${store_name}"
    if ! [ -f "$out_path" ]; then
        ./target/debug/map_dump scrape -p 32 "$store_name" "$out_path"
    fi
    echo "FINISHED STORE: ${store_name}"
done

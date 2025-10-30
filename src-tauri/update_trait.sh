#!/bin/bash
cd ..
input_file="src-core/src/market_data/market_data_traits.rs"
output_file="src-core/src/market_data/market_data_traits.rs.new"

# Take lines 1-49
head -n 49 "$input_file" > "$output_file"

# Add lines 98 onwards
tail -n +98 "$input_file" >> "$output_file"

# Replace original
mv "$output_file" "$input_file"
echo "Trait file updated successfully"

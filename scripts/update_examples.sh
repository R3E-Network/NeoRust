#!/bin/bash

for toml_file in $(find examples -name "Cargo.toml" | grep -v target); do
  echo "Updating $toml_file..."
  
  # Check if [features] section already exists
  if grep -q "\[features\]" "$toml_file"; then
    echo "Features section already exists in $toml_file, skipping..."
    continue
  fi
  
  # Get the dependencies section line number
  dep_line=$(grep -n "\[dependencies\]" "$toml_file" | cut -d ":" -f1)
  
  # Find the neo3 dependency line
  neo3_line=$(grep -n "neo3" "$toml_file" | head -1 | cut -d ":" -f1)
  
  # Backup the file
  cp "$toml_file" "$toml_file.bak"
  
  # Update neo3 dependency to include features = []
  sed -i.tmp "s/neo3 = {.*}/neo3 = { path = \"..\/..\/\", package = \"neo3\", features = [] }/g" "$toml_file"
  
  # Find the end of dependencies section
  next_section_line=$(grep -n "^\[" "$toml_file" | awk -v deps="$dep_line" '$1 > deps {print $1; exit}' | cut -d ":" -f1)
  
  if [ -z "$next_section_line" ]; then
    # If no next section, append at the end
    cat >> "$toml_file" << 'EOF'

[features]
default = []
futures = ["neo3/futures"]
ledger = ["neo3/ledger"]
aws = ["neo3/aws"]
sgx = ["neo3/sgx"]
sgx_deps = ["neo3/sgx_deps", "sgx"]
EOF
  else
    # Insert before the next section
    next_section_line=$((next_section_line - 1))
    { head -n "$next_section_line" "$toml_file"; cat << 'EOF'

[features]
default = []
futures = ["neo3/futures"]
ledger = ["neo3/ledger"]
aws = ["neo3/aws"]
sgx = ["neo3/sgx"]
sgx_deps = ["neo3/sgx_deps", "sgx"]
EOF
      tail -n +$((next_section_line + 1)) "$toml_file"; } > "$toml_file.new"
    mv "$toml_file.new" "$toml_file"
  fi
  
  # Remove temporary files
  rm -f "$toml_file.tmp"
done 
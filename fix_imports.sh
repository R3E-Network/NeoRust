#!/bin/bash

# Fix imports across all CLI files
find ./neo-cli/src -name "*.rs" -type f -exec sed -i '' 's/neo3::neo_serializers/neo3::prelude/g' {} \;
find ./neo-cli/src -name "*.rs" -type f -exec sed -i '' 's/neo3::neo_providers/neo3::prelude/g' {} \;
find ./neo-cli/src -name "*.rs" -type f -exec sed -i '' 's/neo3::neo_builders/neo3::neo_builder/g' {} \;
find ./neo-cli/src -name "*.rs" -type f -exec sed -i '' 's/neo3::types::Signer/neo3::prelude::Signer/g' {} \;
find ./neo-cli/src -name "*.rs" -type f -exec sed -i '' 's/neo3::types::SignerScope/neo3::prelude::SignerScope/g' {} \;
find ./neo-cli/src -name "*.rs" -type f -exec sed -i '' 's/neo3::transaction/neo3::prelude/g' {} \;
find ./neo-cli/src -name "*.rs" -type f -exec sed -i '' 's/neo3::script/neo3::neo_builder::script/g' {} \;
find ./neo-cli/src -name "*.rs" -type f -exec sed -i '' 's/neo3::neo_types::signer/neo3::prelude/g' {} \;
find ./neo-cli/src -name "*.rs" -type f -exec sed -i '' 's/neo3::neo_types::hash::H160/neo3::prelude::H160/g' {} \;
find ./neo-cli/src -name "*.rs" -type f -exec sed -i '' 's/neo3::neo_contract::nep17/neo3::neo_contract/g' {} \;
find ./neo-cli/src -name "*.rs" -type f -exec sed -i '' 's/use neo3::types::/use neo3::neo_types::/g' {} \;

# Fix specific method and type signatures
find ./neo-cli/src -name "*.rs" -type f -exec sed -i '' 's/ContractParameter::Map/ContractParameter::map/g' {} \;

echo "Imports fixed!"

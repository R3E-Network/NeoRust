# Neo N3 Real-World Reference Data

This file provides essential reference data for working with Neo N3 mainnet and testnet, including contract addresses, RPC endpoints, and NeoFS endpoints.

## Native Contract Addresses

Native contracts are the built-in contracts that provide core functionality to the Neo N3 blockchain. These addresses are consistent across both mainnet and testnet:

| Contract Name       | Contract Address (Script Hash)                |
|---------------------|----------------------------------------------|
| ContractManagement  | 0xfffdc93764dbaddd97c48f252a53ea4643faa3fd   |
| StdLib             | 0xacce6fd80d44e1796aa0c2c625e9e4e0ce39efc0   |
| CryptoLib          | 0x726cb6e0cd8628a1350a611384688911ab75f51b   |
| LedgerContract     | 0xda65b600f7124ce6c79950c1772a36403104f2be   |
| NeoToken           | 0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5   |
| GasToken           | 0xd2a4cff31913016155e38e474a2c06d08be276cf   |
| PolicyContract     | 0xcc5e4edd9f5f8dba8bb65734541df7a1c081c67b   |
| RoleManagement     | 0x49cf4e5378ffcd4dec034fd98a174c5491e395e2   |
| OracleContract     | 0xfe924b7cfe89ddd271abaf7210a80a7e11178758   |
| NameService        | 0x7a8fcf0392cd625647907afa8e45cc66872b596b   |

## Famous Contract Addresses

### Mainnet

| Contract Name       | Contract Address (Script Hash)                | Description              |
|---------------------|----------------------------------------------|--------------------------|
| Flamingo FLM Token  | 0x4d9eab13620fe3569ba3b0e56e2877739e4145e3   | FLM governance token    |
| Flamingo Finance    | 0x1a4e5b62b908c758417eb525ecba58752a947f2b   | DeFi platform           |
| GhostMarket         | 0xced5862a6c2f0c70b82b8017e845fb1a31c62c9c   | NFT marketplace         |
| NeoBurger DAO       | 0x48c40d4666f93408be1bef038b6722404d9a4c2a   | Governance platform     |
| NeoCompound         | 0xcd21f4a5dc6a6da341764e7dc9f15f8b38880f49   | GAS staking platform    |
| Neo Name Service    | 0x7a8fcf0392cd625647907afa8e45cc66872b596b   | Domain name service     |
| Poly Network Bridge | 0xd8dd5a0871eb44992cda9c6b49b3954206d6c8a5   | Cross-chain bridge      |

### Testnet

| Contract Name       | Contract Address (Script Hash)                | Description              |
|---------------------|----------------------------------------------|--------------------------|
| Testnet NNS         | 0x50ac1c37690cc2cfc594472833cf57e299e1d367   | Name service on testnet |
| Testnet Faucet      | 0xd65c5d2764b3850a7f7ab14e04f866e9ceab46e1   | Token distribution      |

## Network RPC Endpoints

### Mainnet

Public RPC endpoints for Neo N3 Mainnet:

| Provider           | Endpoint URL                              | Features                    |
|--------------------|-------------------------------------------|----------------------------|
| NeoEconomic Space  | https://mainnet1.neo.coz.io:443           | Full node with all plugins |
| NeoEconomic Space  | https://mainnet2.neo.coz.io:443           | Backup node               |
| NeoEconomic Space  | https://mainnet3.neo.coz.io:443           | Backup node               |
| NGD                | https://mainnet.neo.org                   | Official Neo node         |
| NGD                | https://mainnet1.neo.org                  | Official Neo node         |
| NGD                | https://mainnet2.neo.org                  | Official Neo node         |
| NGD                | https://mainnet3.neo.org                  | Official Neo node         |
| NeoSPCC            | https://rpc01.mainnet.neofs.devops.nspcc.ru | NeoFS infrastructure     |
| NeoSPCC            | https://rpc02.mainnet.neofs.devops.nspcc.ru | NeoFS infrastructure     |
| NeoTrace           | https://n3.neotrace.io                    | Explorer infrastructure   |
| EDGE - NEO Global  | https://edge.n.neo.org:443                | High-availability node    |
| Libre              | https://n3.neoline.io:443                 | NeoLine wallet node       |

### Testnet

Public RPC endpoints for Neo N3 Testnet:

| Provider           | Endpoint URL                              | Features                    |
|--------------------|-------------------------------------------|----------------------------|
| NeoEconomic Space  | https://testnet1.neo.coz.io:443           | Full node with all plugins |
| NeoEconomic Space  | https://testnet2.neo.coz.io:443           | Backup node               |
| NGD                | https://testnet.neo.org                   | Official Neo node         |
| NGD                | https://testnet1.neo.org                  | Official Neo node         |
| NGD                | https://testnet2.neo.org                  | Official Neo node         |
| NGD                | https://testnet3.neo.org                  | Official Neo node         |
| NeoSPCC            | https://rpc01.testnet.neofs.devops.nspcc.ru | NeoFS infrastructure     |
| NeoSPCC            | https://rpc02.testnet.neofs.devops.nspcc.ru | NeoFS infrastructure     |
| NeoTrace           | https://n3-testnet.neotrace.io            | Explorer infrastructure   |

## NeoFS Endpoints

### Mainnet

| Service Type       | Endpoint URL                              | Details                      |
|--------------------|-------------------------------------------|------------------------------|
| gRPC API          | grpc.mainnet.fs.neo.org:8082              | Primary gRPC endpoint        |
| HTTP Gateway      | https://http.mainnet.fs.neo.org           | HTTP access to NeoFS         |
| REST API          | https://rest.mainnet.fs.neo.org           | RESTful API access           |
| gRPC Backup       | grpc1.mainnet.fs.neo.org:8082             | Backup gRPC endpoint         |
| gRPC Backup       | grpc2.mainnet.fs.neo.org:8082             | Backup gRPC endpoint         |

### Testnet

| Service Type       | Endpoint URL                              | Details                      |
|--------------------|-------------------------------------------|------------------------------|
| gRPC API          | grpc.testnet.fs.neo.org:8082              | Primary gRPC endpoint        |
| HTTP Gateway      | https://http.testnet.fs.neo.org           | HTTP access to NeoFS         |
| REST API          | https://rest.testnet.fs.neo.org           | RESTful API access           |
| gRPC Backup       | grpc1.testnet.fs.neo.org:8082             | Backup gRPC endpoint         |
| gRPC Backup       | grpc2.testnet.fs.neo.org:8082             | Backup gRPC endpoint         |

## Popular Block Explorers

| Name              | URL                                       | Features                      |
|-------------------|-------------------------------------------|------------------------------|
| Dora              | https://dora.coz.io/                     | CoZ Explorer with advanced analytics |
| NeoTube           | https://neo3.neotube.io/                 | User-friendly explorer       |
| NeoTrace          | https://neotrace.io/                     | Explorer with contract monitoring |
| Neo-Explorer      | https://explorer.onegate.space/          | OneGate ecosystem explorer    |

## Additional Resources

- [Neo Documentation](https://docs.neo.org)
- [NEO Developer Resources](https://developers.neo.org)
- [NeoFS Documentation](https://docs.neo.org/docs/n3/Advances/neofs/introduction/Overview.html)
- [Neo Testnet Faucet](https://neowish.ngd.network/)
- [Neo Discord](https://discord.io/neo)
- [Neo GitHub](https://github.com/neo-project/)

## Software Development Kits

- [NeoFS SDK - Go](https://github.com/nspcc-dev/neofs-sdk-go)
- [Neo-Go](https://github.com/nspcc-dev/neo-go)
- [NeoRust](https://github.com/R3E-Network/NeoRust) (This Repository)

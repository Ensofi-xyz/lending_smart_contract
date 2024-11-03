This is the lending contract for the Ensofi Project.

## Development

### Prerequisites

- `sui` cli
- node v18
- npm

### Deployment checklist

- [ ] Switch to desired env (dev or prod)

```bash
cp .env.prod .env
cp Move.prod.toml Move.toml
```

- [ ] Check SUI enviroment

#### To check active env

```bash
sui client active-env
```

#### To activate mainnet

```bash
sui client switch --env mainnet
```

- [ ] Check deployer address

#### To check active address

```bash
sui client active-address
```

#### To activate address

```bash
sui client switch --address <address>
```

#### To import new wallet (private key with scheme is ed25519)

```bash
sui keytool import <private_key> ed25519 --alias <alias>
```

- [ ] Check Admin and Operator address (Move.toml)

```bash
admin="0x8b0447dbf083b3f713e09470b70f001c7da3b74c20acc037a5568ed5246f3786"
operator="0x9f6f133896d17d07aeb229ecf8ddf87d6d1839bcaacd5e47f4ae94c6d9ee5c14"
```

- [ ] Publish

```bash
./scripts/bash/publish.sh
```

- [ ] Update env (package) (see .env.example)
- [ ] Initialize system

```bash
node ./scripts/init_system.js
```

- [ ] Initialize asset tier

```bash
node ./scripts/init_asset_tier.sh
```

- [ ] Add configuration token (USDC and SUI)

```bash
node ./scripts/add_configuration_token.js
```

# xlend-enso-lending

## Build and deploy program to mainnet

### Build program 

Go into Anchor.toml and add the program id mainnet below

```
[programs.mainnet]
enso_lending = "<program_id_in_mainnet>"
```

Update the path file of wallet that is authority of program and change cluster to mainnet
```
[provider]
cluster = "mainnet"
wallet = "<path_file_of_wallet>"
```

Build command
```bash
$ anchor build -- --feature mainnet
```

### Deploy

```bash
$ anchor deploy --program-name enso-lending --program-keypair /Users/minhnguyen/Documents/Working/Ensofi/xlend-smart-contract/target/deploy/enso_lending-keypair.json
```

## Trouble shooting when deploy

### 1.Insufficient fund 
```
...
Error: Account a23QvEb6Q3adWurmUVbJ9YfAa6tiEzZBdXB8cvmhViS has insufficient funds for spend...
```
This error occur because the authority wallet does not have enough fund to action the transaction, please send fund to the wallet and redeployed again

### 2.Account data to small
```
...
Error: Deploying program failed: RPC response error -32002: Transaction simulation failed: Error processing Instruction 0: account data too small for instruction [3 log messages]
```
This error occur because the space of program deployed in onchain is less the the space that program need when deployed

Step to extends more space to deployed 

Get the current bytes of the program will deployed with this command
```bash
$ ls -l  ./target/deploy
```
Result
```
total 1544
-rw-------  1 minhnguyen  staff     226 Jul 16 01:27 enso_lending-keypair.json
-rwxr-xr-x  1 minhnguyen  staff  785208 Jul 16 02:07 enso_lending.so
```

Go to sol scan or solana explorer to get the current space of program
1. Navigate to the explorer, find the program by pasted program id
2. Get the id of Executable Data Account and pasted into explorer
3. Get the number in field Data Size (Bytes)

Subtract the current bytes in local with current data size had find
```
Extend addition bytes = 100 + <current_bytes_of_program_build_in_local> - <current_bytes_of_program_in_onchain>
```
Command extend program

```bash
$ solana program extend <program_id> <Addition bytes> --keypair <file_path_of_wallet>
```

And the redeployed with the command above

## Note
### Issue when deploy with Anchor
- Blockhash expired, and maximum retry is only 5 times
```bash
Blockhash expired. 0 retries remaining
Error: Data writes to account failed: Custom error: Max retries exceeded
There was a problem deploying: Output { status: ExitStatus(unix_wait_status(256)), stdout: "", stderr: "" }.
```
- Cannot resume deploy with buffers
- To close buffers to get back lamports
```bash
solana program close --authority "{deployer wallet.json}" --buffers
```
### Deploy with solana cli
1. Check solana config
```bash
solana config get
```
2. Set solana config
```bash
solana config set -u "{rpc url}" -k "{deployer wallet.json}"
```
3. Deploy: set priority fee & increase attempts
- Refer: https://solana.com/docs/programs/deploying
```bash
solana program deploy "{deployer wallet.json}" --with-compute-unit-price 5000 --max-sign-attempts 1000 --use-rpc
```
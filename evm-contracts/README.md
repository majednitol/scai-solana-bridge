# Sample Hardhat Project

This project demonstrates a basic Hardhat use case. It comes with a sample contract, a test for that contract, and a Hardhat Ignition module that deploys that contract.

Try running some of the following tasks:

```shell
npx hardhat help
npx hardhat test
REPORT_GAS=true npx hardhat test
npx hardhat node
npx hardhat ignition deploy ./ignition/modules/Lock.js
```
npx hardhat run ./ignition/modules/deploy.js --network customL2

npx hardhat run ./deploy.js --network sepolia

Integrate Solana off-chain signing with your validator keys and feed them to this bridge.

Optionally, add EIP‑712 structured data signing for even more security and cross-chain consistency.


✅ Locking SCAI

✅ Reverting zero lock amounts

✅ Executing unlocks with valid validator signatures

✅ Preventing replay attacks

✅ Rejecting insufficient validator signatures

✅ Rejecting expired unlock messages


solana-test-validator

anchor keys sync
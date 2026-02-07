// scripts/deploy.js
const hre = require("hardhat");
const { upgrades } = hre;

async function main() {
  const [deployer] = await hre.ethers.getSigners();
  console.log("Deploying contracts with:", deployer.address);

  // 1️⃣ Deploy MessageVerifier
  const Verifier = await hre.ethers.getContractFactory("MessageVerifier");
  const verifier = await Verifier.deploy(); // ethers v6: returns deployed contract directly
  console.log("MessageVerifier deployed at:", verifier.target);

  // 2️⃣ Deploy ValidatorRegistry with initial validators
  const Validator = await hre.ethers.getContractFactory("ValidatorRegistry");
  const validators = [
    "0x717aBdEAb84d50cD1063E7DA8498965C69489b6f",
    "0xfdC7004944C3d86DaCE1CCD175fE78ba24B7AFFf",
    "0xE3a8f890FB281f74955be194a7FC842777Ff6b83"
  ];
  const threshold = 2;
  const validatorRegistry = await Validator.deploy(validators, threshold);
  console.log("ValidatorRegistry deployed at:", validatorRegistry.target);

  // 3️⃣ Deploy SCAITokenLocker proxy
  const Locker = await hre.ethers.getContractFactory("SCAITokenLocker");
  const locker = await upgrades.deployProxy(Locker, [verifier.target], { initializer: 'initialize' });
  await locker.waitForDeployment();
  console.log("SCAITokenLocker deployed at:", locker.target);

  // 4️⃣ Deploy BridgeManager proxy
  const Bridge = await hre.ethers.getContractFactory("BridgeManager");
  const bridge = await upgrades.deployProxy(Bridge, [validatorRegistry.target, verifier.target], { initializer: 'initialize' });
  await bridge.waitForDeployment();
  console.log("BridgeManager deployed at:", bridge.target);
}

// Run the deployment script
main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });

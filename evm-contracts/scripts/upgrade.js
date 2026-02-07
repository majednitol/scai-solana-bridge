import { ethers, upgrades } from "hardhat";

async function main() {
  const Locker = await ethers.getContractFactory("SCAITokenLocker");
  const locker = await upgrades.upgradeProxy("0xLockerAddress...", Locker);
  console.log("SCAITokenLocker upgraded to:", locker.address);

  const Bridge = await ethers.getContractFactory("BridgeManager");
  const bridge = await upgrades.upgradeProxy("0xBridgeAddress...", Bridge);
  console.log("BridgeManager upgraded to:", bridge.address);
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});

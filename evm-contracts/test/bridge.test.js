const { expect } = require("chai");
const hre = require("hardhat");
const { ethers, upgrades } = hre;

describe("SCAI Bridge EVM Contracts", function () {
  let owner, user, validator1, validator2;
  let verifier, validatorRegistry, locker, bridge;
  let chainId;

  beforeEach(async function () {
    [owner, user, validator1, validator2] = await ethers.getSigners();
    chainId = (await ethers.provider.getNetwork()).chainId;

    // ---------------- Deploy ValidatorRegistry ----------------
    const Validator = await ethers.getContractFactory("ValidatorRegistry");
    validatorRegistry = await Validator.deploy(
      [validator1.address, validator2.address],
      2
    );
    await validatorRegistry.waitForDeployment();

    // ---------------- Deploy MessageVerifier ----------------
    const Verifier = await ethers.getContractFactory("MessageVerifier");
    verifier = await Verifier.deploy(validatorRegistry.target); // pass ValidatorRegistry address
    await verifier.waitForDeployment();

    // ---------------- Deploy SCAITokenLocker ----------------
    const Locker = await ethers.getContractFactory("SCAITokenLocker");
    locker = await upgrades.deployProxy(
      Locker,
      [verifier.target], // pass MessageVerifier address
      { initializer: "initialize" }
    );
    await locker.waitForDeployment();

    // ---------------- Deploy BridgeManager ----------------
    const Bridge = await ethers.getContractFactory("BridgeManager");
    bridge = await upgrades.deployProxy(
      Bridge,
      [validatorRegistry.target, verifier.target, 3600], // expiry = 1 hour
      { initializer: "initialize" }
    );
    await bridge.waitForDeployment();

    // Fund BridgeManager with ETH
    await owner.sendTransaction({
      to: bridge.target,
      value: ethers.parseEther("5"),
    });
  });

  // --------------------------- LOCK TESTS ---------------------------
  it("locks SCAI successfully", async function () {
    const tx = await locker.connect(user).lock({ value: ethers.parseEther("1") });
    await tx.wait();
    expect(await locker.totalLocked()).to.equal(ethers.parseEther("1"));
  });

  it("reverts if lock amount is zero", async function () {
    await expect(
      locker.connect(user).lock({ value: 0 })
    ).to.be.revertedWith("SCAITokenLocker: Must lock >0");
  });

  // --------------------------- UNLOCK TESTS ---------------------------

  // Helper: create ECDSA signatures from validators
// Helper: sign unlock message with validators
// Helper: sign unlock message with validators
async function signUnlock(recipient, amount, orderId, nonce, timestamp) {
  const msgHash = await verifier.hashMessage(
    chainId,
    recipient,
    amount,
    orderId,
    nonce,
    timestamp
  );
console.log("msgHash",msgHash);
  // Convert hash to bytes
  const msgBytes = ethers.getBytes(msgHash);
  console.log("msgBytes",msgBytes);
  // Sign with Ethereum Signed Message prefix
  const sig1 = await validator1.signMessage(msgBytes);
  const sig2 = await validator2.signMessage(msgBytes);
  console.log("sig1",sig1);
  console.log("sig2",sig2);
  return [sig1, sig2];
}



it("executes unlock with valid validator signatures", async function () {
  const orderId = ethers.keccak256(ethers.toUtf8Bytes("order-1"));
  const amount = ethers.parseEther("1");
  const nonce = 1;
  const timestamp = Math.floor(Date.now() / 1000);

  // Lock first
  await locker.connect(user).lock({ value: amount });

  // Sign
  const sigs = await signUnlock(user.address, amount, orderId, nonce, timestamp);

  const before = await ethers.provider.getBalance(user.address);

  // Execute unlock
  await bridge.executeUnlock(
    orderId,
    user.address,
    amount,
    nonce,
    timestamp,
    sigs
  );

  const after = await ethers.provider.getBalance(user.address);
  expect(after).to.be.gt(before);
});


  it("prevents replay attack (double unlock)", async function () {
    const orderId = ethers.keccak256(ethers.toUtf8Bytes("order-replay"));
    const amount = ethers.parseEther("1");
    const nonce = 1;
    const timestamp = Math.floor(Date.now() / 1000);

    // Lock first
    await locker.connect(user).lock({ value: amount });

    const sigs = await signUnlock(user.address, amount, orderId, nonce, timestamp);

    await bridge.executeUnlock(
      orderId,
      user.address,
      amount,
      nonce,
      timestamp,
      sigs
    );

    await expect(
      bridge.executeUnlock(
        orderId,
        user.address,
        amount,
        nonce,
        timestamp,
        sigs
      )
    ).to.be.revertedWith("BridgeManager: Already executed");
  });

  it("rejects unlock with insufficient validator signatures", async function () {
    const orderId = ethers.keccak256(ethers.toUtf8Bytes("order-bad-sigs"));
    const amount = ethers.parseEther("1");
    const nonce = 1;
    const timestamp = Math.floor(Date.now() / 1000);

    // Lock first
    await locker.connect(user).lock({ value: amount });

    const msgHash = await verifier.hashMessage(
      chainId,
      user.address,
      amount,
      orderId,
      nonce,
      timestamp
    );

    // Only one validator signs
    const sig1 = await validator1.signMessage(ethers.getBytes(msgHash));

    await expect(
      bridge.executeUnlock(
        orderId,
        user.address,
        amount,
        nonce,
        timestamp,
        [sig1]
      )
    ).to.be.revertedWith("BridgeManager: Invalid validator signatures");
  });

  it("rejects expired unlock message", async function () {
    const orderId = ethers.keccak256(ethers.toUtf8Bytes("order-expired"));
    const amount = ethers.parseEther("1");
    const nonce = 1;
    const timestamp = Math.floor(Date.now() / 1000) - 7200; // 2 hours ago

    // Lock first
    await locker.connect(user).lock({ value: amount });

    const sigs = await signUnlock(user.address, amount, orderId, nonce, timestamp);

    await expect(
      bridge.executeUnlock(
        orderId,
        user.address,
        amount,
        nonce,
        timestamp,
        sigs
      )
    ).to.be.revertedWith("BridgeManager: Expired message");
  });
});

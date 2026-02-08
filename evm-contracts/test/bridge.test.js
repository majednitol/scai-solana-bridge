const { expect } = require("chai");
const { ethers, upgrades } = require("hardhat");

describe("SCAI Bridge EVM Contracts", function () {
  let owner, user, validator1, validator2;
  let validatorRegistry, verifier, locker, bridge;
  let chainId;

  beforeEach(async function () {
    [owner, user, validator1, validator2] = await ethers.getSigners();
    chainId = (await ethers.provider.getNetwork()).chainId;

    // ---------------- Deploy ValidatorRegistry (Upgradeable) ----------------
  // ---------------- Deploy ValidatorRegistry (Upgradeable) ----------------
const ValidatorRegistry = await ethers.getContractFactory("ValidatorRegistry");
validatorRegistry = await upgrades.deployProxy(
  ValidatorRegistry,
  [[validator1.address, validator2.address], 2], // args for initialize()
  { initializer: "initialize" }
);
// wait for deployment
await validatorRegistry.waitForDeployment(); // <-- use .deployed(), not .waitForDeployment()
console.log("ValidatorRegistry deployed at:", validatorRegistry.address);

// ---------------- Deploy MessageVerifier (Non-upgradeable) ----------------
const MessageVerifier = await ethers.getContractFactory("MessageVerifier");
verifier = await MessageVerifier.deploy(validatorRegistry.address); // pass validatorRegistry address
await verifier.deployed(); // <-- use .deployed()
console.log("MessageVerifier deployed at:", verifier.address);

// ---------------- Deploy SCAITokenLocker (Upgradeable) ----------------
const SCAITokenLocker = await ethers.getContractFactory("SCAITokenLocker");
locker = await upgrades.deployProxy(
  SCAITokenLocker,
  [verifier.address, validatorRegistry.address],
  { initializer: "initialize" }
);
await locker.deployed();
console.log("SCAITokenLocker deployed at:", locker.address);

// ---------------- Deploy BridgeManager (Upgradeable) ----------------
const BridgeManager = await ethers.getContractFactory("BridgeManager");
bridge = await upgrades.deployProxy(
  BridgeManager,
  [chainId, validatorRegistry.address, verifier.address],
  { initializer: "initialize" }
);
await bridge.deployed();
console.log("BridgeManager deployed at:", bridge.address);


    // ---------------- Fund BridgeManager with ETH ----------------
    await owner.sendTransaction({
      to: bridge.address,
      value: ethers.utils.parseEther("5"),
    });
  });

  // --------------------------- LOCK TESTS ---------------------------
  describe("SCAITokenLocker", function () {
    it("locks SCAI successfully", async function () {
      const amount = ethers.utils.parseEther("1");
      const tx = await locker.connect(user).lock({ value: amount });
      const receipt = await tx.wait();

      const event = receipt.events.find((e) => e.event === "Locked");
      expect(event.args.sender).to.equal(user.address);
      expect(event.args.amount).to.equal(amount);
      expect(event.args.orderId).to.not.be.undefined;

      const orderId = event.args.orderId;
      expect(await locker.lockedOrders(orderId)).to.equal(amount);
      expect(await locker.totalLocked()).to.equal(amount);
    });

    it("reverts if lock amount is zero", async function () {
      await expect(
        locker.connect(user).lock({ value: 0 })
      ).to.be.revertedWith("SCAITokenLocker: Must lock >0");
    });
  });

  // --------------------------- UNLOCK TESTS ---------------------------
  async function signUnlock(recipient, amount, orderId, nonce, timestamp) {
    const msgHash = await verifier.hashMessage(
      chainId,
      recipient,
      amount,
      orderId,
      nonce,
      timestamp
    );

    const msgBytes = ethers.utils.arrayify(msgHash);
    const sig1 = await validator1.signMessage(msgBytes);
    const sig2 = await validator2.signMessage(msgBytes);
    return [sig1, sig2];
  }

  it("executes unlock with valid validator signatures", async function () {
    const orderId = ethers.utils.keccak256(ethers.utils.toUtf8Bytes("order-1"));
    const amount = ethers.utils.parseEther("1");
    const nonce = 1;
    const timestamp = Math.floor(Date.now() / 1000);

    // Lock first
    await locker.connect(user).lock({ value: amount });

    // Sign
    const sigs = await signUnlock(user.address, amount, orderId, nonce, timestamp);

    const before = await ethers.provider.getBalance(user.address);

    // Execute unlock
    await bridge.executeUnlock(orderId, user.address, amount, nonce, timestamp, sigs);

    const after = await ethers.provider.getBalance(user.address);
    expect(after).to.be.gt(before);
  });

  it("prevents replay attack (double unlock)", async function () {
    const orderId = ethers.utils.keccak256(ethers.utils.toUtf8Bytes("order-replay"));
    const amount = ethers.utils.parseEther("1");
    const nonce = 1;
    const timestamp = Math.floor(Date.now() / 1000);

    await locker.connect(user).lock({ value: amount });

    const sigs = await signUnlock(user.address, amount, orderId, nonce, timestamp);

    await bridge.executeUnlock(orderId, user.address, amount, nonce, timestamp, sigs);

    await expect(
      bridge.executeUnlock(orderId, user.address, amount, nonce, timestamp, sigs)
    ).to.be.revertedWith("BridgeManager: Already executed");
  });

  it("rejects unlock with insufficient validator signatures", async function () {
    const orderId = ethers.utils.keccak256(ethers.utils.toUtf8Bytes("order-bad-sigs"));
    const amount = ethers.utils.parseEther("1");
    const nonce = 1;
    const timestamp = Math.floor(Date.now() / 1000);

    await locker.connect(user).lock({ value: amount });

    const msgHash = await verifier.hashMessage(chainId, user.address, amount, orderId, nonce, timestamp);
    const sig1 = await validator1.signMessage(ethers.utils.arrayify(msgHash));

    await expect(
      bridge.executeUnlock(orderId, user.address, amount, nonce, timestamp, [sig1])
    ).to.be.revertedWith("BridgeManager: Invalid validator signatures");
  });

  it("rejects expired unlock message", async function () {
    const orderId = ethers.utils.keccak256(ethers.utils.toUtf8Bytes("order-expired"));
    const amount = ethers.utils.parseEther("1");
    const nonce = 1;
    const timestamp = Math.floor(Date.now() / 1000) - 7200; // 2 hours ago

    await locker.connect(user).lock({ value: amount });

    const sigs = await signUnlock(user.address, amount, orderId, nonce, timestamp);

    await expect(
      bridge.executeUnlock(orderId, user.address, amount, nonce, timestamp, sigs)
    ).to.be.revertedWith("BridgeManager: Expired message");
  });
});

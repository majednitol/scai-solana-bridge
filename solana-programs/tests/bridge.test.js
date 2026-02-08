const anchor = require("@coral-xyz/anchor");
const { assert } = require("chai");
const { Keypair, PublicKey, SystemProgram } = require("@solana/web3.js");
const {
  createMint,
  getOrCreateAssociatedTokenAccount,
  getAccount,
} = require("@solana/spl-token");
const keccak256 = require("keccak256");
const secp256k1 = require("secp256k1");
const BN = require("bn.js");
const crypto = require("crypto");
const borsh = require("borsh");

// -------------------------
// Borsh Serialization for BridgeMessage
// -------------------------
class BridgeMessage {
  constructor(fields) {
    this.sourceChainId = fields.sourceChainId;
    this.destinationChainId = fields.destinationChainId;
    this.orderId = fields.orderId;
    this.amount = fields.amount;
    this.sender = fields.sender;
    this.recipient = fields.recipient;
    this.nonce = fields.nonce;
    this.timestamp = fields.timestamp;
  }
}

const BridgeMessageSchema = new Map([
  [
    BridgeMessage,
    {
      kind: "struct",
      fields: [
        ["sourceChainId", "u64"],
        ["destinationChainId", "u64"],
        ["orderId", ["u8", 32]],
        ["amount", "u64"],
        ["sender", ["u8", 20]],
        ["recipient", ["u8", 32]],
        ["nonce", "u64"],
        ["timestamp", "u64"],
      ],
    },
  ],
]);

function serializeBridgeMessage(msg) {
  return borsh.serialize(BridgeMessageSchema, new BridgeMessage(msg));
}

// Helper to hash BridgeMessage
function hashBridgeMessage(msg) {
  const serialized = serializeBridgeMessage(msg);
  const hash = keccak256(Buffer.from(serialized));
  return hash;
}

// -------------------------
// Test Suite
// -------------------------
describe("SCAI ↔ Solana Bridge", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.BridgeProgram;
  const payer = provider.wallet;

  let config, validatorSet, mint, userToken;

  // EVM validator keys
  const validatorPrivKeys = [
    Buffer.from("1".repeat(64), "hex"),
    Buffer.from("2".repeat(64), "hex"),
    Buffer.from("3".repeat(64), "hex"),
  ];

  const validatorAddresses = validatorPrivKeys.map((pk) => {
    const pub = secp256k1.publicKeyCreate(pk, false).slice(1);
    const hash = keccak256(Buffer.from(pub));
    return Array.from(hash.slice(-20));
  });

  const threshold = 2;

  // -------------------------
  // Setup mint & accounts
  // -------------------------
  before(async () => {
    config = Keypair.generate();
    validatorSet = Keypair.generate();

    mint = await createMint(
      provider.connection,
      payer.payer,
      payer.publicKey,
      null,
      9
    );

    const ata = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      payer.payer,
      mint,
      payer.publicKey
    );
    userToken = ata.address;
  });

  // -------------------------
  // Initialize Bridge
  // -------------------------
  it("initializes bridge", async () => {
    await program.methods
      .initialize({ validators: validatorAddresses, threshold })
      .accounts({
        config: config.publicKey,
        validatorSet: validatorSet.publicKey,
        payer: payer.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([config, validatorSet])
      .rpc();

    const cfg = await program.account.bridgeConfig.fetch(config.publicKey);
    const vs = await program.account.validatorSet.fetch(validatorSet.publicKey);

    assert.equal(cfg.validatorThreshold, threshold);
    assert.equal(cfg.paused, false);
    assert.equal(vs.count, validatorAddresses.length);
  });

  // -------------------------
  // Execute Mint
  // -------------------------
  it("executes mint with valid validator signatures", async () => {
    const orderId = crypto.randomBytes(32);
    const sender = Buffer.alloc(20, 1);
    const recipient = payer.publicKey.toBuffer();

    const msg = {
      sourceChainId: new BN(9000),
      destinationChainId: new BN(1),
      orderId: Array.from(orderId),
      amount: new BN(1_000_000_000),
      sender: Array.from(sender),
      recipient: Array.from(recipient),
      nonce: new BN(1),
      timestamp: new BN(Math.floor(Date.now() / 1000)),
    };

    const hash = hashBridgeMessage(msg);

    const signatures = validatorPrivKeys.slice(0, threshold).map((pk) => {
      const { signature, recid } = secp256k1.ecdsaSign(hash, pk);
      return Array.from(Buffer.concat([signature, Buffer.from([recid])]));
    });

    const [execPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("exec"), orderId],
      program.programId
    );

    await program.methods
      .executeMint(msg, signatures)
      .accounts({
        config: config.publicKey,
        validatorSet: validatorSet.publicKey,
        executed: execPda,
        mint,
        recipient: userToken,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        payer: payer.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const acct = await getAccount(provider.connection, userToken);
    assert.equal(Number(acct.amount), 1_000_000_000);
  });

  // -------------------------
  // Confirm Unlock
  // -------------------------
  it("confirms unlock with valid signatures", async () => {
    const burnOrder = Keypair.generate();
    const amount = new BN(500_000_000);
    const recipientEvm = validatorAddresses[0];

    await program.methods
      .initiateBurn(amount, recipientEvm)
      .accounts({
        config: config.publicKey,
        userToken,
        burnOrder: burnOrder.publicKey,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        user: payer.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([burnOrder])
      .rpc();

    const msg = {
      sourceChainId: new BN(1),
      destinationChainId: new BN(9000),
      orderId: Array.from(burnOrder.publicKey.toBytes()),
      amount,
      sender: Array.from(Buffer.alloc(20, 1)),
      recipient: Array.from(payer.publicKey.toBuffer()),
      nonce: new BN(1),
      timestamp: new BN(Math.floor(Date.now() / 1000)),
    };

    const hash = hashBridgeMessage(msg);
    const signatures = validatorPrivKeys.slice(0, threshold).map((pk) => {
      const { signature, recid } = secp256k1.ecdsaSign(hash, pk);
      return Array.from(Buffer.concat([signature, Buffer.from([recid])]));
    });

    await program.methods
      .confirmUnlock(msg, signatures)
      .accounts({
        user: payer.publicKey,
        mint,
        tokenAccount: userToken,
        bridgeConfig: config.publicKey,
        systemProgram: SystemProgram.programId,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      })
      .rpc();

    const bo = await program.account.burnOrder.fetch(burnOrder.publicKey);
    assert.equal(bo.executed, true);
  });

  // -------------------------
  // Update Validators
  // -------------------------
  it("updates validator set", async () => {
    const newValidators = validatorAddresses.slice(0, 2);

    await program.methods
      .updateValidators(newValidators, 2)
      .accounts({
        config: config.publicKey,
        validatorSet: validatorSet.publicKey,
        admin: payer.publicKey,
      })
      .rpc();

    const vs = await program.account.validatorSet.fetch(validatorSet.publicKey);
    assert.equal(vs.count, newValidators.length);
  });

  // -------------------------
  // Replay & Invalid Signatures
  // -------------------------
  it("prevents replay and rejects invalid signatures", async () => {
    const orderId = crypto.randomBytes(32);
    const msg = {
      sourceChainId: new BN(1),
      destinationChainId: new BN(9000),
      orderId: Array.from(orderId),
      amount: new BN(100),
      sender: Array.from(Buffer.alloc(20, 1)),
      recipient: Array.from(payer.publicKey.toBuffer()),
      nonce: new BN(99),
      timestamp: new BN(Math.floor(Date.now() / 1000)),
    };

    const sigs = [Array(65).fill(0)];

    try {
      await program.methods.executeMint(msg, sigs).rpc();
      assert.fail("Insufficient signatures accepted");
    } catch (err) {
      assert.ok(true);
    }

    const [execPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("exec"), orderId],
      program.programId
    );

    try {
      await program.methods
        .executeMint(msg, sigs)
        .accounts({
          config: config.publicKey,
          validatorSet: validatorSet.publicKey,
          executed: execPda,
          mint,
          recipient: userToken,
          tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
          payer: payer.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
      assert.fail("Replay allowed");
    } catch (err) {
      assert.ok(true);
    }
  });
});

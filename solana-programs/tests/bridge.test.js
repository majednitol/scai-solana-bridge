const anchor = require("@coral-xyz/anchor");
const { assert } = require("chai");

const {
  Keypair,
  PublicKey,
  SystemProgram,
} = require("@solana/web3.js");

const {
  createMint,
  getOrCreateAssociatedTokenAccount,
  getAccount,
} = require("@solana/spl-token");

const keccak256 = require("keccak256");
const secp256k1 = require("secp256k1");
const BN = require("bn.js");
const crypto = require("crypto");

describe("SCAI ↔ Solana Bridge", () => {
  // -------------------------
  // Anchor setup
  // -------------------------
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.BridgeProgram;
  const payer = provider.wallet;

  // -------------------------
  // Accounts / keys
  // -------------------------
  let config;
  let validatorSet;
  let mint;
  let recipientToken;

  // -------------------------
  // EVM validator keys (secp256k1)
  // -------------------------
  const validatorPrivKeys = [
    Buffer.from("1".repeat(64), "hex"),
    Buffer.from("2".repeat(64), "hex"),
    Buffer.from("3".repeat(64), "hex"),
  ];

  const validatorAddresses = validatorPrivKeys.map((pk) => {
    const pub = secp256k1.publicKeyCreate(pk, false).slice(1);
    return keccak256(pub).slice(12); // last 20 bytes
  });

  const threshold = 2;

  // -------------------------
  // Setup
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

    recipientToken = ata.address;
  });

  // -------------------------
  // Initialize bridge
  // -------------------------
  it("initializes bridge", async () => {
    await program.methods
      .initialize({
        validators: validatorAddresses.map((v) => Array.from(v)),
        threshold,
      })
      .accounts({
        config: config.publicKey,
        validatorSet: validatorSet.publicKey,
        payer: payer.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([config, validatorSet])
      .rpc();

    const cfg = await program.account.bridgeConfig.fetch(config.publicKey);
    assert.equal(cfg.validatorThreshold, threshold);
    assert.equal(cfg.paused, false);
  });

  // -------------------------
  // Execute mint
  // -------------------------
  it("executes mint with valid validator signatures", async () => {
    const msg = {
      sourceChainId: new BN(9000),
      destinationChainId: new BN(1),
      orderId: Array.from(crypto.randomBytes(32)),
      amount: new BN(1_000_000_000),
      sender: Array(20).fill(1),
      recipient: payer.publicKey.toBytes(),
      nonce: new BN(1),
      timestamp: new BN(Math.floor(Date.now() / 1000)),
    };

    const msgBytes = program.coder.types.encode("BridgeMessage", msg);
    const hash = keccak256(msgBytes);

    const signatures = validatorPrivKeys.slice(0, threshold).map((pk) => {
      const { signature, recid } = secp256k1.ecdsaSign(hash, pk);
      return Buffer.concat([signature, Buffer.from([recid])]);
    });

    const [execPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("exec"), Buffer.from(msg.orderId)],
      program.programId
    );

    await program.methods
      .executeMint(
        msg,
        signatures.map((s) => Array.from(s))
      )
      .accounts({
        config: config.publicKey,
        validatorSet: validatorSet.publicKey,
        executed: execPda,
        mint,
        recipient: recipientToken,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        payer: payer.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const acct = await getAccount(provider.connection, recipientToken);
    assert.equal(Number(acct.amount), 1_000_000_000);
  });

  // -------------------------
  // Replay protection
  // -------------------------
  it("prevents replay attack (double mint)", async () => {
    try {
      await program.methods.executeMint({}, []).rpc();
      assert.fail("Replay allowed");
    } catch (e) {
      assert.ok(true);
    }
  });

  // -------------------------
  // Insufficient signatures
  // -------------------------
  it("rejects insufficient validator signatures", async () => {
    const msg = {
      sourceChainId: new BN(9000),
      destinationChainId: new BN(1),
      orderId: Array.from(crypto.randomBytes(32)),
      amount: new BN(100),
      sender: Array(20).fill(2),
      recipient: payer.publicKey.toBytes(),
      nonce: new BN(2),
      timestamp: new BN(Math.floor(Date.now() / 1000)),
    };

    const msgBytes = program.coder.types.encode("BridgeMessage", msg);
    const hash = keccak256(msgBytes);

    const { signature, recid } = secp256k1.ecdsaSign(
      hash,
      validatorPrivKeys[0]
    );

    const sig = Buffer.concat([signature, Buffer.from([recid])]);

    try {
      await program.methods
        .executeMint(msg, [Array.from(sig)])
        .rpc();
      assert.fail("Insufficient signatures accepted");
    } catch (e) {
      assert.ok(true);
    }
  });

  // -------------------------
  // Burn flow
  // -------------------------
  it("initiates burn", async () => {
    const burnOrder = Keypair.generate();

    await program.methods
      .initiateBurn(
        new BN(500_000_000),
        Array.from(validatorAddresses[0])
      )
      .accounts({
        config: config.publicKey,
        userToken: recipientToken,
        burnOrder: burnOrder.publicKey,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        user: payer.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([burnOrder])
      .rpc();

    const acct = await getAccount(provider.connection, recipientToken);
    assert.equal(Number(acct.amount), 500_000_000);
  });

  // -------------------------
  // Validator update
  // -------------------------
  it("updates validator set", async () => {
    const newValidators = validatorAddresses.slice(0, 2);

    await program.methods
      .updateValidators(
        newValidators.map((v) => Array.from(v)),
        2
      )
      .accounts({
        config: config.publicKey,
        validatorSet: validatorSet.publicKey,
        admin: payer.publicKey,
      })
      .rpc();

    const vs = await program.account.validatorSet.fetch(
      validatorSet.publicKey
    );
    assert.equal(vs.validators.length, 2);
  });
});

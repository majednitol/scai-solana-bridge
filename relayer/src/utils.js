import fs from 'fs';
import { Connection, Keypair, Transaction, sendAndConfirmTransaction } from '@solana/web3.js';
import { Program, AnchorProvider, Wallet } from '@coral-xyz/anchor';
import * as anchor from '@coral-xyz/anchor';
import { logger } from './utils.js';

// Load Solana payer keypair
export const loadKeypair = (path) => {
  const secret = JSON.parse(fs.readFileSync(path, 'utf8'));
  return Keypair.fromSecretKey(Uint8Array.from(secret));
};

// Create Anchor provider
export const getAnchorProvider = (rpc, payerPath) => {
  const keypair = loadKeypair(payerPath);
  const wallet = new Wallet(keypair);
  const connection = new Connection(rpc, 'confirmed');
  return new AnchorProvider(connection, wallet, { preflightCommitment: 'confirmed' });
};

// Submit bridge message to Solana program
export const submitToSolana = async (programId, bridgeMsg, payerPath, rpc) => {
  try {
    const provider = getAnchorProvider(rpc, payerPath);
    anchor.setProvider(provider);

    const idl = JSON.parse(fs.readFileSync('./idl/bridge_program.json', 'utf8')); 
    const program = new Program(idl, programId, provider);

    const tx = await program.methods
      .executeMint(bridgeMsg, []) 
      .accounts({
        config: program.programId,
        validatorSet: program.programId, 
        executed: Keypair.generate().publicKey, 
        mint: bridgeMsg.mint,
        recipient: bridgeMsg.recipient,
        tokenProgram: anchor.SPL_TOKEN_PROGRAM_ID,
        payer: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    logger.info(`Submitted to Solana: ${tx}`);
    return tx;
  } catch (err) {
    logger.error(`Solana submission error: ${err}`);
    throw err;
  }
};

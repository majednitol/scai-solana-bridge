import { Watcher } from './watcher.js';
import { Submitter } from './submitter.js';
import { loadConfig, logger, submitToSolana } from './utils.js';
import { ethers } from 'ethers';

async function main() {
  const network = process.env.NETWORK || 'testnet';
  const config = loadConfig(network);

  // Setup EVM contract
  const evmProvider = new ethers.JsonRpcProvider(config.evm.rpc);
  const evmBridge = new ethers.Contract(
    config.evm.bridgeAddress,
    config.evm.bridgeAbi,
    evmProvider
  );

  const submitter = new Submitter(config.evm);
  const watcher = new Watcher(config.evm);

  // Watch EVM Burn events
  await watcher.watch(evmBridge, 'Burn', async (event) => {
    logger.info(`New Burn event: ${JSON.stringify(event.args)}`);

    try {
      const bridgeMsg = {
        orderId: event.args.orderId,
        recipient: event.args.recipient,
        amount: event.args.amount.toString(),
        mint: config.solana.mintAddress,
      };

      // Submit to Solana program
      const tx = await submitToSolana(
        config.solana.programId,
        bridgeMsg,
        config.solana.payerKeypair,
        config.solana.rpc
      );

      logger.info(`Bridge executed on Solana: ${tx}`);
    } catch (err) {
      logger.error(`Failed to submit to Solana: ${err}`);
    }
  });
}

main().catch((err) => {
  logger.error(`Relayer failed: ${err}`);
  process.exit(1);
});

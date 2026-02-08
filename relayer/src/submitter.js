import { ethers } from 'ethers';
import { logger } from './utils.js';

export class Submitter {
  constructor(config) {
    this.provider = new ethers.JsonRpcProvider(config.rpc);
    this.wallet = new ethers.Wallet(config.privateKey, this.provider);
  }

  async submitTransaction(contract, method, args = [], gasLimit = 300_000) {
    try {
      const tx = await contract.connect(this.wallet)[method](...args, { gasLimit });
      logger.info(`Submitted tx: ${tx.hash}`);
      const receipt = await tx.wait();
      logger.info(`Tx confirmed in block ${receipt.blockNumber}`);
      return receipt;
    } catch (err) {
      logger.error(`Submitter error: ${err}`);
      throw err;
    }
  }
}

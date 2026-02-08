import { ethers } from 'ethers';
import { logger, loadConfig } from './utils.js';

export class Watcher {
  constructor(config) {
    this.config = config;
    this.provider = new ethers.JsonRpcProvider(config.rpc);
    this.lastBlock = config.startBlock || 0;
  }

  async getEvents(contract, eventName, fromBlock, toBlock) {
    try {
      const filter = contract.filters[eventName]();
      const events = await contract.queryFilter(filter, fromBlock, toBlock);
      return events;
    } catch (err) {
      logger.error(`Failed to fetch events: ${err}`);
      return [];
    }
  }

  async watch(contract, eventName, callback, pollInterval = 5000) {
    logger.info(`Watcher started for event ${eventName}`);
    while (true) {
      try {
        const latestBlock = await this.provider.getBlockNumber();
        if (latestBlock > this.lastBlock) {
          const events = await this.getEvents(contract, eventName, this.lastBlock + 1, latestBlock);
          for (const e of events) await callback(e);
          this.lastBlock = latestBlock;
        }
      } catch (err) {
        logger.error(`Watcher error: ${err}`);
      }
      await new Promise((resolve) => setTimeout(resolve, pollInterval));
    }
  }
}

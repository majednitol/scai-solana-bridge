require("@nomicfoundation/hardhat-toolbox");
require('@openzeppelin/hardhat-upgrades'); 
/** @type import('hardhat/config').HardhatUserConfig */
module.exports = {
  solidity: "0.8.28",
    networks: {
      hardhat: {
      chainId: 1337
      },
      sepolia: {
      url: "https://eth-sepolia.g.alchemy.com/v2/iQ_8RwrWNQWD7MLe5YNZJ",
      accounts: ["d56a8b6b3b47b74df2b4ae8a80faeafc4c56efd16ab106096be835ca86e30f6d"]
    },
    securechain: {
      url: "https://mainnet-rpc.scai.network",
      accounts: ["d56a8b6b3b47b74df2b4ae8a80faeafc4c56efd16ab106096be835ca86e30f6d"]
    }
  }
};

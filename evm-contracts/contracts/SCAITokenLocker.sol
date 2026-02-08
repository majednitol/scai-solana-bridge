// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/utils/ReentrancyGuardUpgradeable.sol";
import "./ValidatorRegistry.sol";
import "./MessageVerifier.sol";


contract SCAITokenLocker is UUPSUpgradeable, OwnableUpgradeable, ReentrancyGuardUpgradeable {


    uint256 public totalLocked;

    // Mapping of lock orderId => locked amount
    mapping(bytes32 => uint256) public lockedOrders;

    // Mapping of orderId / VAA => executed
    mapping(bytes32 => bool) public executedVAAs;

    // Wormhole components
    MessageVerifier public verifier;
    ValidatorRegistry public validatorRegistry;


    event Locked(address indexed sender, uint256 amount, bytes32 indexed orderId);
    event Unlocked(address indexed recipient, uint256 amount, bytes32 indexed orderId);

   
    function initialize(address _verifier, address _validatorRegistry) external initializer {
        require(_verifier != address(0), "SCAITokenLocker: zero verifier");
        require(_validatorRegistry != address(0), "SCAITokenLocker: zero validatorRegistry");

        verifier = MessageVerifier(_verifier);
        validatorRegistry = ValidatorRegistry(_validatorRegistry);

        __Ownable_init(msg.sender);
        __ReentrancyGuard_init();
    }

    function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}

    receive() external payable {}


    function lock() external payable nonReentrant returns (bytes32) {
        require(msg.value > 0, "SCAITokenLocker: Must lock >0");

        totalLocked += msg.value;

        bytes32 orderId = keccak256(
            abi.encodePacked(msg.sender, msg.value, block.timestamp, block.number)
        );

        lockedOrders[orderId] = msg.value;

        emit Locked(msg.sender, msg.value, orderId);
        return orderId;
    }

    struct UnlockParams {
        address recipient;
        uint256 amount;
        bytes32 orderId;
        bytes[] signatures;
        uint8 vaaVersion;
        uint32 guardianSetIndex;
        uint16 emitterChainId;
        bytes32 emitterAddress;
        uint64 sequence;
        bytes payload;
    }

    function unlock(UnlockParams calldata p) external nonReentrant {
        require(p.recipient != address(0), "SCAITokenLocker: zero recipient");
        require(p.amount > 0, "SCAITokenLocker: zero amount");
        require(!executedVAAs[p.orderId], "SCAITokenLocker: Already executed");

        uint256 lockedAmount_ = lockedOrders[p.orderId];
        require(lockedAmount_ >= p.amount, "SCAITokenLocker: Insufficient locked funds");

        bytes32 vaaHash = verifier.hashVAA(
            p.vaaVersion,
            p.guardianSetIndex,
            p.emitterChainId,
            p.emitterAddress,
            p.sequence,
            p.payload
        );

    
        require(
            validatorRegistry.verifySignatures(p.guardianSetIndex, vaaHash, p.signatures),
            "SCAITokenLocker: Invalid signatures"
        );

        // Mark as executed before transfer (reentrancy protection)
        executedVAAs[p.orderId] = true;
        totalLocked -= p.amount;
        lockedOrders[p.orderId] = lockedAmount_ - p.amount;

        // Transfer ETH
        (bool sent, ) = payable(p.recipient).call{value: p.amount}("");
        require(sent, "SCAITokenLocker: ETH transfer failed");

        emit Unlocked(p.recipient, p.amount, p.orderId);
    }


    function isVAAConsumed(bytes32 orderId) external view returns (bool) {
        return executedVAAs[orderId];
    }
}

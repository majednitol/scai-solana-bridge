// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/utils/ReentrancyGuardUpgradeable.sol";
import "./MessageVerifier.sol";

/**
 * @title SCAITokenLocker
 * @notice Locks native SCAI tokens on Secure Chain AI
 */
contract SCAITokenLocker is UUPSUpgradeable, OwnableUpgradeable, ReentrancyGuardUpgradeable {
    uint256 public totalLocked;
    mapping(bytes32 => uint256) public lockedOrders;
    mapping(bytes32 => bool) public executedOrders;
    MessageVerifier public verifier;

    event Locked(address indexed sender, uint256 amount, bytes32 indexed orderId);
    event Unlocked(address indexed recipient, uint256 amount, bytes32 indexed orderId);

    function initialize(address _verifier) public initializer {
        require(_verifier != address(0), "SCAITokenLocker: zero verifier");
        verifier = MessageVerifier(_verifier);

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

    function unlock(
        address recipient,
        uint256 amount,
        bytes32 orderId,
        bytes[] calldata signatures,
        uint256 nonce,
        uint256 timestamp
    ) external nonReentrant {
        require(recipient != address(0), "SCAITokenLocker: zero recipient");
        require(amount > 0, "SCAITokenLocker: zero amount");
        require(!executedOrders[orderId], "SCAITokenLocker: Already executed");
        require(block.timestamp - timestamp < 15 minutes, "SCAITokenLocker: Expired message");

        uint256 lockedAmount_ = lockedOrders[orderId];
        require(lockedAmount_ >= amount, "SCAITokenLocker: Invalid amount");

        bytes32 messageHash = verifier.hashMessage(
            block.chainid,
            recipient,
            amount,
            orderId,
            nonce,
            timestamp
        );

        require(verifier.verifySignatures(messageHash, signatures), "SCAITokenLocker: Invalid signatures");

        executedOrders[orderId] = true;
        totalLocked -= amount;
        lockedOrders[orderId] = lockedAmount_ - amount;

        (bool sent, ) = payable(recipient).call{value: amount}("");
        require(sent, "SCAITokenLocker: ETH transfer failed");

        emit Unlocked(recipient, amount, orderId);
    }
}

// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";
import "@openzeppelin/contracts/utils/cryptography/MessageHashUtils.sol";
import "@openzeppelin/contracts/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import "./ValidatorRegistry.sol";
import "./MessageVerifier.sol";

/**
 * @title BridgeManager
 */
contract BridgeManager is UUPSUpgradeable, OwnableUpgradeable {
    using MessageHashUtils for bytes32;

    ValidatorRegistry public validatorRegistry;
    MessageVerifier public verifier;

    mapping(bytes32 => bool) public executedOrders;
    uint256 public messageExpiry;

    event UnlockExecuted(bytes32 indexed orderId, address indexed recipient, uint256 amount);

    function initialize(
        address _validatorRegistry,
        address _verifier,
        uint256 _messageExpiry
    ) public initializer {
        validatorRegistry = ValidatorRegistry(_validatorRegistry);
        verifier = MessageVerifier(_verifier);
        messageExpiry = _messageExpiry;
        __Ownable_init(msg.sender);
    }

    function _authorizeUpgrade(address) internal override onlyOwner {}

    receive() external payable {}

    function executeUnlock(
        bytes32 orderId,
        address recipient,
        uint256 amount,
        uint256 nonce,
        uint256 timestamp,
        bytes[] calldata signatures
    ) external {
        require(recipient != address(0), "BridgeManager: zero recipient");
        require(amount > 0, "BridgeManager: zero amount");
        require(!executedOrders[orderId], "BridgeManager: Already executed");
        require(timestamp + messageExpiry >= block.timestamp, "BridgeManager: Expired message");

        bytes32 msgHash = verifier.hashMessage(
            block.chainid,
            recipient,
            amount,
            orderId,
            nonce,
            timestamp
        );

        // Convert to Ethereum Signed Message hash
        bytes32 ethSigned = msgHash.toEthSignedMessageHash();

        require(
            validatorRegistry.verifySignatures(ethSigned, signatures),
            "BridgeManager: Invalid validator signatures"
        );

        executedOrders[orderId] = true;

        (bool sent, ) = payable(recipient).call{value: amount}("");
        require(sent, "BridgeManager: ETH transfer failed");

        emit UnlockExecuted(orderId, recipient, amount);
    }
}

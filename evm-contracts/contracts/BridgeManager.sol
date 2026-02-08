// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/utils/ReentrancyGuardUpgradeable.sol";
import "./ValidatorRegistry.sol";
import "./MessageVerifier.sol";

contract BridgeManager is
    UUPSUpgradeable,
    OwnableUpgradeable,
    ReentrancyGuardUpgradeable
{
    // Contracts
    ValidatorRegistry public validatorRegistry; // Guardian set registry
    MessageVerifier public verifier;            // VAA hashing logic

    // State
    mapping(uint16 => mapping(bytes32 => uint64)) public lastSequence;
    mapping(bytes32 => bool) public consumedVAAs;
    uint16 public chainId; // Local chain ID

    // Events
    event VAAConsumed(
        bytes32 indexed vaaHash,
        uint16 indexed emitterChainId,
        bytes32 indexed emitterAddress,
        uint64 sequence
    );

    event ETHUnlocked(
        address indexed recipient,
        uint256 amount,
        bytes32 indexed orderId
    );

    /// @notice Initialize function (upgradeable)
    function initialize(
        uint16 _chainId,
        address _validatorRegistry,
        address _verifier
    ) external initializer {
        require(_chainId != 0, "BridgeManager: invalid chainId");
        require(_validatorRegistry != address(0), "BridgeManager: zero registry");
        require(_verifier != address(0), "BridgeManager: zero verifier");

        chainId = _chainId;
        validatorRegistry = ValidatorRegistry(_validatorRegistry);
        verifier = MessageVerifier(_verifier);

        __Ownable_init(msg.sender);         // initialize OwnableUpgradeable
        __ReentrancyGuard_init(); // initialize ReentrancyGuardUpgradeable
    }

    /// @notice Authorize upgrades (UUPS)
    function _authorizeUpgrade(address) internal override onlyOwner {}

    /// @notice Receive ETH
    receive() external payable {}

    /**
     * @notice Execute a VAA payload
     */
    function executeVAA(
        uint8 version,
        uint32 guardianSetIndex,
        uint16 emitterChainId,
        bytes32 emitterAddress,
        uint64 sequence,
        bytes calldata payload,
        bytes[] calldata signatures
    ) external nonReentrant {
        // Compute VAA hash
        bytes32 vaaHash = verifier.hashVAA(
            version,
            guardianSetIndex,
            emitterChainId,
            emitterAddress,
            sequence,
            payload
        );

        // Replay protection
        require(!consumedVAAs[vaaHash], "BridgeManager: VAA already consumed");
        require(sequence > lastSequence[emitterChainId][emitterAddress], "BridgeManager: sequence too low");

        // Verify signatures
        bool valid = validatorRegistry.verifySignatures(
            guardianSetIndex,
            vaaHash,
            signatures
        );
        require(valid, "BridgeManager: Invalid guardian signatures");

        // Mark consumed
        consumedVAAs[vaaHash] = true;
        lastSequence[emitterChainId][emitterAddress] = sequence;

        // Execute the payload
        _executePayload(payload);

        emit VAAConsumed(vaaHash, emitterChainId, emitterAddress, sequence);
    }

    /**
     * @notice Internal payload execution
     */
    function _executePayload(bytes calldata payload) internal {
        (uint8 payloadId, bytes32 orderId, address recipient, uint256 amount) =
            abi.decode(payload, (uint8, bytes32, address, uint256));

        require(payloadId == 1, "BridgeManager: Unknown payload type");
        require(recipient != address(0), "BridgeManager: zero recipient");
        require(amount > 0, "BridgeManager: zero amount");

        (bool sent, ) = payable(recipient).call{value: amount}("");
        require(sent, "BridgeManager: ETH transfer failed");

        emit ETHUnlocked(recipient, amount, orderId);
    }

    /// @notice Check if a VAA has been consumed
    function isVAAConsumed(bytes32 vaaHash) external view returns (bool) {
        return consumedVAAs[vaaHash];
    }

    /// @notice Get last sequence for emitter
    function getLastSequence(uint16 emitterChainId, bytes32 emitterAddress) external view returns (uint64) {
        return lastSequence[emitterChainId][emitterAddress];
    }
}

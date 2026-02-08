// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";

/**
 * @title ValidatorRegistry
 * @notice Manages guardian sets and verifies VAA signatures.
 */
contract ValidatorRegistry is Initializable, OwnableUpgradeable {

    struct GuardianSet {
        address[] guardians;
        uint256 expirationTime; // unix timestamp when this set expires
    }

    // Current active guardian set index
    uint32 public currentSetIndex;

    // Mapping from set index to GuardianSet
    mapping(uint32 => GuardianSet) public guardianSets;

    // Number of signatures required to validate a VAA
    mapping(uint32 => uint256) public thresholds;

    // Events
    event GuardianSetUpdated(uint32 indexed index, address[] guardians, uint256 expirationTime);
    event ThresholdUpdated(uint32 indexed index, uint256 threshold);

    
    function initialize(address[] memory _guardians, uint256 _threshold) external initializer {
        __Ownable_init(msg.sender);

        require(_guardians.length > 0, "ValidatorRegistry: empty guardians");
        require(_threshold > 0 && _threshold <= _guardians.length, "ValidatorRegistry: invalid threshold");

        currentSetIndex = 0;
        guardianSets[currentSetIndex] = GuardianSet({
            guardians: _guardians,
            expirationTime: type(uint256).max
        });
        thresholds[currentSetIndex] = _threshold;

        emit GuardianSetUpdated(currentSetIndex, _guardians, type(uint256).max);
        emit ThresholdUpdated(currentSetIndex, _threshold);
    }


    function updateGuardianSet(
        address[] calldata _guardians,
        uint256 _expirationTime,
        uint256 _threshold
    ) external onlyOwner {
        require(_guardians.length > 0, "ValidatorRegistry: empty guardians");
        require(_expirationTime > block.timestamp, "ValidatorRegistry: expiration must be future");
        require(_threshold > 0 && _threshold <= _guardians.length, "ValidatorRegistry: invalid threshold");

        currentSetIndex += 1;
        guardianSets[currentSetIndex] = GuardianSet({
            guardians: _guardians,
            expirationTime: _expirationTime
        });
        thresholds[currentSetIndex] = _threshold;

        emit GuardianSetUpdated(currentSetIndex, _guardians, _expirationTime);
        emit ThresholdUpdated(currentSetIndex, _threshold);
    }

    /// @notice Verify signatures against a guardian set
    function verifySignatures(
        uint32 setIndex,
        bytes32 hash,
        bytes[] calldata signatures
    ) external view returns (bool) {
        GuardianSet memory set = guardianSets[setIndex];
        uint256 threshold = thresholds[setIndex];
        uint256 sigCount = signatures.length;

        if (sigCount < threshold) return false;

        uint256 seenBitmap = 0;
        uint256 validCount = 0;

        for (uint256 i = 0; i < sigCount; i++) {
            address signer = _recoverSigner(hash, signatures[i]);

            // Find signer index in set
            int256 signerIndex = -1;
            for (uint256 j = 0; j < set.guardians.length; j++) {
                if (set.guardians[j] == signer) {
                    signerIndex = int256(j);
                    break;
                }
            }
            if (signerIndex < 0) continue; // not in guardian set

            // Prevent duplicates using bitmap
            if ((seenBitmap & (1 << uint256(signerIndex))) != 0) continue;

            seenBitmap |= (1 << uint256(signerIndex));
            validCount++;

            if (validCount >= threshold) return true;
        }

        return false;
    }

    /// @notice Recover signer address from a hash and signature
    function _recoverSigner(bytes32 hash, bytes memory signature) internal pure returns (address) {
        require(signature.length == 65, "ValidatorRegistry: invalid signature length");
        bytes32 r;
        bytes32 s;
        uint8 v;

        assembly {
            r := mload(add(signature, 32))
            s := mload(add(signature, 64))
            v := byte(0, mload(add(signature, 96)))
        }

        if (v < 27) v += 27;
        require(v == 27 || v == 28, "ValidatorRegistry: invalid v");

        address signer = ecrecover(hash, v, r, s);
        require(signer != address(0), "ValidatorRegistry: invalid signer");
        return signer;
    }

    /// @notice Get number of guardians in a set
    function guardianCount(uint32 setIndex) external view returns (uint256) {
        return guardianSets[setIndex].guardians.length;
    }

    /// @notice Get guardian address by index
    function getGuardian(uint32 setIndex, uint256 index) external view returns (address) {
        require(index < guardianSets[setIndex].guardians.length, "ValidatorRegistry: index out of bounds");
        return guardianSets[setIndex].guardians[index];
    }
}

// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/**
 * @title ValidatorRegistry
 * @notice Maintains a semi-permissioned validator set for the bridge
 */
contract ValidatorRegistry {
    address[] public validators;
    uint256 public threshold;

    mapping(address => bool) public isValidator;

    event ValidatorAdded(address validator);
    event ValidatorRemoved(address validator);
    event ThresholdUpdated(uint256 newThreshold);

    constructor(address[] memory _validators, uint256 _threshold) {
        require(_validators.length > 0, "ValidatorRegistry: Empty validators");
        require(_threshold > 0, "ValidatorRegistry: Threshold zero");
        require(_threshold <= _validators.length, "ValidatorRegistry: Threshold too high");

        threshold = _threshold;

        for (uint256 i = 0; i < _validators.length; i++) {
            address validator = _validators[i];
            require(validator != address(0), "ValidatorRegistry: Zero address");
            require(!isValidator[validator], "ValidatorRegistry: Duplicate validator");

            isValidator[validator] = true;
            validators.push(validator);

            emit ValidatorAdded(validator);
        }
    }

    function verifySignatures(bytes32 messageHash, bytes[] calldata signatures) external view returns (bool) {
        uint256 sigCount = signatures.length;
        if (sigCount < threshold) return false;

        uint256 validCount = 0;
        address[] memory seen = new address[](sigCount);

        for (uint256 i = 0; i < sigCount; i++) {
            address signer = _recoverSigner(messageHash, signatures[i]);
            if (!isValidator[signer]) continue;

            // prevent duplicate validator signatures
            bool duplicate = false;
            for (uint256 j = 0; j < validCount; j++) {
                if (seen[j] == signer) {
                    duplicate = true;
                    break;
                }
            }

            if (duplicate) continue;

            seen[validCount] = signer;
            validCount++;

            if (validCount >= threshold) {
                return true;
            }
        }

        return false;
    }

    function _recoverSigner(bytes32 hash, bytes memory signature) internal pure returns (address) {
        require(signature.length == 65, "ValidatorRegistry: Invalid signature length");
        bytes32 r;
        bytes32 s;
        uint8 v;

        assembly {
            r := mload(add(signature, 32))
            s := mload(add(signature, 64))
            v := byte(0, mload(add(signature, 96)))
        }

        if (v < 27) v += 27;
        require(v == 27 || v == 28, "ValidatorRegistry: Invalid v");

        address signer = ecrecover(hash, v, r, s);
        require(signer != address(0), "ValidatorRegistry: Invalid signer");

        return signer;
    }

    function validatorCount() external view returns (uint256) {
        return validators.length;
    }
}

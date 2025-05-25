// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import {ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";
import {
    IERC20,
    SafeERC20
} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";

/// @title DodoV2SwapAdapter
/// @dev Adapter for swapping tokens on DodoV2 pools.
contract DodoV2SwapAdapter is ISwapAdapter {
    using SafeERC20 for IERC20;

    constructor() {}

    receive() external payable {}

    /// @inheritdoc ISwapAdapter
    function price(
        bytes32 poolId,
        address sellToken,
        address,
        uint256[] memory specifiedAmounts
    ) external view override returns (Fraction[] memory calculatedPrices) {
        calculatedPrices = new Fraction[](specifiedAmounts.length);

        IDODOV2Pool pool = IDODOV2Pool(address(bytes20(poolId)));

        for (uint256 i = 0; i < specifiedAmounts.length; i++) {
            calculatedPrices[i] = priceAt(pool, sellToken, specifiedAmounts[i]);
        }
        return calculatedPrices;
    }

    /// @notice Calculate the price of a token at a specified amount.
    /// @param pool The pool to calculate the price for.
    /// @param sellToken The token to calculate the price for.
    /// @param sellAmount The amount of the token to calculate the price for.
    /// @return calculatedPrice The calculated price of the token.
    function priceAt(IDODOV2Pool pool, address sellToken, uint256 sellAmount)
        public
        view
        returns (Fraction memory calculatedPrice)
    {
        bool isBaseTokenIn = (sellToken == address(pool._BASE_TOKEN_()));
        if (isBaseTokenIn) {
            (uint256 receiveQuoteAmount,,,) =
                pool.querySellBase(msg.sender, sellAmount);
            calculatedPrice = Fraction({
                numerator: receiveQuoteAmount,
                denominator: sellAmount
            });
        } else {
            (uint256 receiveBaseAmount,,,) =
                pool.querySellQuote(msg.sender, sellAmount);
            calculatedPrice = Fraction({
                numerator: receiveBaseAmount,
                denominator: sellAmount
            });
        }
    }

    /// @inheritdoc ISwapAdapter
    function swap(
        bytes32 poolId,
        address sellToken,
        address,
        OrderSide side,
        uint256 specifiedAmount
    ) external returns (Trade memory trade) {
        if (specifiedAmount == 0) {
            return trade;
        }
        IDODOV2Pool pool = IDODOV2Pool(address(bytes20(poolId)));
        bool isBaseTokenIn = sellToken == address(pool._BASE_TOKEN_());
        uint256 gasBefore = gasleft();
        if (side == OrderSide.Sell) {
            IERC20(sellToken).safeTransferFrom(
                msg.sender, address(pool), specifiedAmount
            );
            if (isBaseTokenIn) {
                trade.calculatedAmount = pool.sellBase(msg.sender);
            } else {
                trade.calculatedAmount = pool.sellQuote(msg.sender);
            }
        } else {
            revert NotImplemented("DodoV2SwapAdapter.BuyOrder");
        }
        trade.gasUsed = gasBefore - gasleft();
        trade.price = priceAt(pool, sellToken, specifiedAmount);
        return trade;
    }

    /// @inheritdoc ISwapAdapter
    function getLimits(bytes32 poolId, address sellToken, address)
        external
        view
        override
        returns (uint256[] memory limits)
    {
        IDODOV2Pool pool = IDODOV2Pool(address(bytes20(poolId)));
        (uint256 baseReserve, uint256 quoteReserve) = pool.getVaultReserve();
        limits = new uint256[](2);
        if (sellToken == address(pool._BASE_TOKEN_())) {
            limits[0] = baseReserve;
            limits[1] = quoteReserve;
        } else if (sellToken == address(pool._QUOTE_TOKEN_())) {
            limits[0] = quoteReserve;
            limits[1] = baseReserve;
        }
    }

    /// @inheritdoc ISwapAdapter
    function getCapabilities(bytes32, address, address)
        external
        pure
        override
        returns (Capability[] memory capabilities)
    {
        capabilities = new Capability[](2);
        capabilities[0] = Capability.SellOrder;
        capabilities[1] = Capability.PriceFunction;
    }

    /// @inheritdoc ISwapAdapter
    function getTokens(bytes32 poolId)
        external
        view
        override
        returns (address[] memory tokens)
    {
        tokens = new address[](2);
        IDODOV2Pool pool = IDODOV2Pool(address(bytes20(poolId)));
        tokens[0] = address(pool._BASE_TOKEN_());
        tokens[1] = address(pool._QUOTE_TOKEN_());
    }

    /// @inheritdoc ISwapAdapter
    function getPoolIds(uint256, uint256)
        external
        pure
        override
        returns (bytes32[] memory)
    {
        revert NotImplemented("DodoV2SwapAdapter.getPoolIds");
    }
}

interface IDODOV2Pool {
    enum RState {
        ONE,
        ABOVE_ONE,
        BELOW_ONE
    }

    function _BASE_TOKEN_() external view returns (address);
    function _QUOTE_TOKEN_() external view returns (address);

    function querySellBase(address trader, uint256 payBaseAmount)
        external
        view
        returns (
            uint256 receiveQuoteAmount,
            uint256 mtFee,
            RState newRState,
            uint256 newBaseTarget
        );

    function querySellQuote(address trader, uint256 payQuoteAmount)
        external
        view
        returns (
            uint256 receiveBaseAmount,
            uint256 mtFee,
            RState newRState,
            uint256 newQuoteTarget
        );

    function sellBase(address to) external returns (uint256);
    function sellQuote(address to) external returns (uint256);
    function getVaultReserve()
        external
        view
        returns (uint256 baseReserve, uint256 quoteReserve);
}

// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import "./AdapterTest.sol";
import "forge-std/Test.sol";
import "src/interfaces/ISwapAdapterTypes.sol";
import "src/libraries/FractionMath.sol";
import "src/dodo-v2/DodoV2SwapAdapter.sol";
import "forge-std/console.sol";

contract DodoV2SwapAdapterTest is AdapterTest {
    using FractionMath for Fraction;

    DodoV2SwapAdapter adapter;
    address constant USDT = 0xdAC17F958D2ee523a2206206994597C13D831ec7;
    address constant USDC = 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48;
    address constant USDT_USDC_POOL = 0xB9A4406982D990648093C71eFF9F1f63A040152e;

    uint256 constant TEST_ITERATIONS = 10;

    uint256 constant USDT_BALANCE = 100_000 * 1e6;
    uint256 constant USDC_BALANCE = 100_000 * 1e6;

    function setUp() public {
        uint256 forkBlock = 22545882;
        vm.createSelectFork(vm.rpcUrl("mainnet"), forkBlock);
        adapter = new DodoV2SwapAdapter();

        vm.label(address(adapter), "DodoV2SwapAdapter");
        vm.label(USDC, "USDC");
        vm.label(USDT, "USDT");
        vm.label(USDT_USDC_POOL, "USDT_USDC_POOL");
    }

    function testGetLimits() public view {
        bytes32 pair = bytes32(bytes20(USDT_USDC_POOL));
        uint256[] memory limits = adapter.getLimits(pair, USDT, USDC);

        assertEq(limits.length, 2);
        assertGt(limits[0], 0, "Limit for sell token should be greater than 0");
        assertGt(limits[1], 0, "Limit for buy token should be greater than 0");
    }

    function testPriceFuzz(uint256 amount0, uint256 amount1) public view {
        bytes32 pair = bytes32(bytes20(USDT_USDC_POOL));
        uint256[] memory limits = adapter.getLimits(pair, USDT, USDC);
        vm.assume(amount0 < limits[0]);
        vm.assume(amount1 < limits[0]);
        vm.assume(amount0 > 1e16);
        vm.assume(amount1 > 1e16);

        uint256[] memory amounts = new uint256[](2);
        amounts[0] = amount0;
        amounts[1] = amount1;

        Fraction[] memory prices = adapter.price(pair, USDT, USDC, amounts);

        for (uint256 i = 0; i < prices.length; i++) {
            assertGt(prices[i].numerator, 0);
            assertGt(prices[i].denominator, 0);
        }
    }

    function testPrice() public view {
        bytes32 pair = bytes32(bytes20(USDT_USDC_POOL));
        uint256[] memory amounts = new uint256[](1);
        amounts[0] = 100e6; // 100 USDT

        Fraction[] memory prices = adapter.price(pair, USDT, USDC, amounts);

        assertEq(prices.length, 1);
        assertGt(
            prices[0].numerator, 0, "Price numerator should be greater than 0"
        );
        assertGt(
            prices[0].denominator,
            0,
            "Price denominator should be greater than 0"
        );
    }

    function testPriceDecreasing() public view {
        bytes32 pair = bytes32(bytes20(USDT_USDC_POOL));
        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);

        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = 100 * (i + 1) * 10 ** 6;
        }

        Fraction[] memory prices = adapter.price(pair, USDT, USDC, amounts);

        for (uint256 i = 0; i < TEST_ITERATIONS - 1; i++) {
            assertGe(prices[i].compareFractions(prices[i + 1]), 1);
            assertGt(prices[i].denominator, 0);
            assertGt(prices[i + 1].denominator, 0);
        }
    }

    function testSwapSell() public {
        bytes32 pair = bytes32(bytes20(USDT_USDC_POOL));
        uint256 amount = 100e6; // 10 USDT

        deal(USDT, address(this), USDT_BALANCE);
        deal(USDC, address(this), USDC_BALANCE);

        // Approve adapter to spend WETH
        vm.prank(address(this));
        IERC20(USDT).approve(address(adapter), amount);

        Trade memory trade =
            adapter.swap(pair, USDT, USDC, OrderSide.Sell, amount);

        assertGt(
            trade.calculatedAmount,
            0,
            "Calculated amount should be greater than 0"
        );
        assertGt(
            trade.price.numerator, 0, "Price numerator should be greater than 0"
        );
        assertGt(
            trade.price.denominator,
            0,
            "Price denominator should be greater than 0"
        );
        assertGt(trade.gasUsed, 0, "Gas used should be greater than 0");
    }

    function testSwapBuy() public {
        bytes32 pair = bytes32(bytes20(USDT_USDC_POOL));
        uint256 amount = 100e18; // buy 100 GHO

        deal(USDT, address(this), USDT_BALANCE);
        deal(USDC, address(this), USDC_BALANCE);

        // Approve adapter to spend USDC
        vm.prank(address(this));
        IERC20(USDC).approve(address(adapter), USDC_BALANCE);

        vm.expectRevert(
            abi.encodeWithSelector(
                NotImplemented.selector, "DodoV2SwapAdapter.BuyOrder"
            )
        );
        adapter.swap(pair, USDC, USDT, OrderSide.Buy, amount);
    }

    function testSwapFuzz(uint256 specifiedAmount) public {
        OrderSide side = OrderSide.Sell;

        bytes32 pair = bytes32(bytes20(USDT_USDC_POOL));
        uint256[] memory limits = adapter.getLimits(pair, USDT, USDC);

        // specify sell usdt amount
        vm.assume(specifiedAmount < limits[0]);

        deal(USDT, address(this), specifiedAmount);
        IERC20(USDT).approve(address(adapter), specifiedAmount);

        uint256 usdt_balance = IERC20(USDT).balanceOf(address(this));
        uint256 usdc_balance = IERC20(USDC).balanceOf(address(this));

        Trade memory trade =
            adapter.swap(pair, USDT, USDC, side, specifiedAmount);

        if (trade.calculatedAmount > 0) {
            assertEq(
                specifiedAmount,
                usdt_balance - IERC20(USDT).balanceOf(address(this))
            );
            assertEq(
                trade.calculatedAmount,
                IERC20(USDC).balanceOf(address(this)) - usdc_balance
            );
        }
    }

    function testSwapSellIncreasing() public {
        executeIncreasingSwaps(OrderSide.Sell);
    }

    function executeIncreasingSwaps(OrderSide side) internal {
        bytes32 pair = bytes32(bytes20(USDT_USDC_POOL));

        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = 100 * (i + 1) * 10 ** 6; // specify sell usdt
        }

        deal(USDT, address(this), USDT_BALANCE);
        deal(USDC, address(this), USDC_BALANCE);

        Trade[] memory trades = new Trade[](TEST_ITERATIONS);
        uint256 beforeSwap;
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            beforeSwap = vm.snapshot();

            IERC20(USDT).approve(address(adapter), USDC_BALANCE);

            trades[i] = adapter.swap(pair, USDT, USDC, side, amounts[i]);
            vm.revertTo(beforeSwap);
        }

        for (uint256 i = 1; i < TEST_ITERATIONS - 1; i++) {
            assertLe(trades[i].calculatedAmount, trades[i + 1].calculatedAmount);
            assertLe(trades[i].gasUsed, trades[i + 1].gasUsed);
            assertEq(trades[i].price.compareFractions(trades[i + 1].price), 1);
        }
    }

    function testGetCapabilities() public view {
        bytes32 pair = bytes32(bytes20(USDT_USDC_POOL));
        Capability[] memory capabilities =
            adapter.getCapabilities(pair, USDT, USDC);

        assertEq(capabilities.length, 2);
        assertEq(uint256(capabilities[0]), uint256(Capability.SellOrder));
        assertEq(uint256(capabilities[1]), uint256(Capability.PriceFunction));
    }

    function testGetTokens() public view {
        bytes32 pair = bytes32(bytes20(USDT_USDC_POOL));
        address[] memory tokens = adapter.getTokens(pair);

        assertEq(tokens.length, 2);
        assertEq(tokens[0], USDT);
        assertEq(tokens[1], USDC);
    }

    function testGetPoolIds() public {
        vm.expectRevert(
            abi.encodeWithSelector(
                NotImplemented.selector, "DodoV2SwapAdapter.getPoolIds"
            )
        );
        adapter.getPoolIds(100, 200);
    }

    function testMavV2PoolBehaviour() public {
        bytes32[] memory poolIds = new bytes32[](1);
        poolIds[0] = bytes32(bytes20(USDT_USDC_POOL));
        runPoolBehaviourTest(adapter, poolIds);
    }
}

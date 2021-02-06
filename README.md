# aUSD stablecoin system

aUSD stablecoin, aka "artificial US Dollars", is a decentralized stablecoin protocol built on NEAR. It consists of three required components and one optional component:

- aGov token contract
- aUSD token contract
- NEAR and aGov price oracle
- (Optional) decentralized, uniswap-like pool for swapping aGov and aUSD with NEAR.

## aGov Token

aGov token, aka "artificial Governance" token, is based on Lockable Fungible token. But locking mechanism is changed:

- Locking must be initiated from the aUSD Token contract, and it's called "deposit".
- User lock aGov token to mint aUSD token at same time, it's called "deposit_and_mint".
- Amount of USD token mint is equal to 20% of the USD values of the aGov token.
- aGov token have a deposit reward that is similar to the inflation rate as NEAR. Total deposit reward is `(total deposit + total undeposit) * inflation rate`. And reward distributed proportionally based on shared of deposit. Undeposited aGov would not receive deposit reward. Deposit reward is added to the undeposit balance and it's unlocked.
- To withdraw deposit balance, user is required to burn aUSD token that's equivalent to the 20% of the USD values of the aGov token at the time of withdraw. This operation is called "burn_and_withdraw"

## aUSD Token

aUSD token is the main stablecoin token that issued from this system.

- User can freely use aUSD token (it's not locked) and transfer
- Once user want to unlock the aGov token, 20% value of the aGov token of aUSD token must be burnt with "burn_and_withdraw"
- There is no deposit reward for holding aUSD token, so for the holder it's an opportunity loss to not receiving staking reward of NEAR or deposit reward aGov, but the benefit is the stable 1:1 USD value
- The aUSD's stable is implicitly guaranteed in this mint-deposit-burn-withdraw semantic. And also explictly as Yyou can always swap aUSD to aGov at price `1/x` if aGov is priced at `x` at this moment with `owner`. Owner will take your aUSD and issue you to your available balance. You must have zero deposit before the swap, otherwise you can always call burn_and_withdraw first. Reversely, you can also buy from owner aUSD by swap aGov
- The rely on owner might seem centralized at first glance, but owner will be owned by multisignature account of all aGov holders in future. They'll also have avility to vote given the portion they owned for proposals of change 20%, upgrade contract, etc. That's why it's called governance token

## NEAR and aGov price oracle

In order for this system to work, it's crucial to have a price indicate how much currently NEAR and aGov is worth in US Dollars. This require a out of chain oracle to fetch and upload price on chain. At the initial stage, this oracle has to be run from trusted centralized providers. In a future version, this would be decentralized and people are paid incentives to run oracle. People have to deposit sufficient number of aGov to run an oracle and must commit price accuracy with other oracles (othwerwise their deposit will be defeited). The benefit to run an oracle is gain extra deposit reward compare to who don't run one.

First implementation of centralized oracle is simple: just read NEAR price from coinmarketcap and (if aGov also on exchange) read aGov price from coinmarketcap. If aGov is not on exchange it's read from the uniswap-like aGov-NEAR exchange and calculated to aGov/\$.

## Decentralized, uniswap-like pool for swapping aGov and aUSD with NEAR.

This provide an alternative way to exchange aGov and aUSD with NEAR and may as the initial way to obtain aGov tokens

## Economics

### aGov has a higher reward rate than NEAR staking reward

Assume aGov/NEAR at a stable level as they designed to be so, and when market demand and supply is in average aligned. For example 1 aGov = 10 NEAR = 20 USD. Alice have 1000 NEAR, and assume she will get 10% APY if staking on NEAR. If she swap to aGov she'll get 100 aGov, equivallent to 2000 USD. She deposit 2000 USD worth of aGov to mint 400 aUSD. She can then sell 400 aUSD to get back $400 and still receive 10% APY of 2000 USD as aGov's deposit reward. So, she use only $1600 to earn the reward which would only available when invest $2000. This is equivalent to a 12.5% APY instead of 10%. Once she want to withdraw, she will need to buy (assume NEAR and aGov price is same as when she deposit) 400 aUSD to unlock her $2000 worth of deposits.

### aUSD stablecoin as the basis of DeFi

Assume Bob doesn't want to bother with aGov at all, he just want a stablecoin that he can use to participate in DeFi. He can simply buy aUSD from an exchange or from the contract. He will not get any reward but it's a stalecoin that can reserve value and as basis of using other DeFi applications.

### When aUSD supply insufficient

As we can see all aUSD comes from mint by deposit aGov. Although there is guaranteed rate to exchange to aGov and then exchange to USD (assume aGov has been on exchanges and have a similar circulation supply as NEAR), there is not necessarilly enough aUSD available to use, which will bring a increase of actual price of aUSD if supply is not sufficiently meet demands. However this will make deposit more aGov to mint aUSD profitable and more user would choose to do so. And this will bring supply of aUSD high until it is no longer profitable to deposit aGov

### When aUSD supply redundant

In above scenario aUSD supply would not infinitely increase because once there's more supply than demand on market, the aUSD price on exchanges will drop below \$1, which make buying aUSD to burn to withdraw aGov profitable, because you need to pay less than usual to withdraw same aGov. As more aUSD is burnt, it's supply will be bring back to normal

### Insufficient liquidity when aGov decrease drastically

If aGov increase in value, there will be no problem as user would bring more aUSD to unlock aGov (20%). However if aGov decrease in value signifcantly, it's theoratically possible that system doesn't have enough liquidity to exchange aUSD to aGov. This is practically impossible, been tested in the Maker and Synthetix system and by chosen a 20% rate to mint, although we don't have a mathematical proof. Consider this example: Alice at day 1 deposit $1000 worth of aGov to mint $200 worth of aUSD. At day 2, aGov price drops 80%, so alice's aGov worth $200 and she also have $200 aUSD. Bob deposit $1000 worth of aGov to mint $200 aUSD. Now system have $1200 worth of aGov and issued $400 aUSD, so even both of them try to exchange aUSD with aGov there's still a \$800 redundancy. In practical even when aGov price drops signifcantly, the weighted average of aGov in system doesn't drop that much and often hedged as there's more users.

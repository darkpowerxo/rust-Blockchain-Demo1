import React, { useState, useEffect, useCallback } from 'react'
import { ArrowUpDown, Settings, RefreshCw, TrendingUp, AlertCircle } from 'lucide-react'
import { useTradingQuote, useSupportedTokens } from '../hooks/useApi'

interface Token {
  symbol: string
  name: string
  address: string
  logoUrl?: string
  balance?: string
  decimals?: number
}

const TradingDashboard: React.FC = () => {
  const { data: supportedTokens } = useSupportedTokens()
  const { data: quote, getQuote } = useTradingQuote()
  
  const [fromToken, setFromToken] = useState<Token>({
    symbol: 'ETH',
    name: 'Ethereum',
    address: '0x0000000000000000000000000000000000000000',
    balance: '2.5834'
  })
  const [toToken, setToToken] = useState<Token>({
    symbol: 'USDC',
    name: 'USD Coin',
    address: '0xa0b86a33e6411d3e1d0b1a06e0a91e6b7a7f8b91',
    balance: '1250.00'
  })
  const [amountIn, setAmountIn] = useState('')
  const [amountOut, setAmountOut] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const [slippage, setSlippage] = useState(0.5)
  const [error, setError] = useState<string | null>(null)

  // Use supported tokens from API or fallback to mock data
  const tokens: Token[] = supportedTokens?.map(token => ({
    symbol: token.symbol,
    name: token.name,
    address: token.address,
    balance: token.balance || '0',
    decimals: token.decimals
  })) || [
    { symbol: 'ETH', name: 'Ethereum', address: '0x0000000000000000000000000000000000000000', balance: '2.5834' },
    { symbol: 'USDC', name: 'USD Coin', address: '0xa0b86a33e6411d3e1d0b1a06e0a91e6b7a7f8b91', balance: '1250.00' },
    { symbol: 'USDT', name: 'Tether USD', address: '0xdac17f958d2ee523a2206206994597c13d831ec7', balance: '0' },
    { symbol: 'UNI', name: 'Uniswap', address: '0x1f9840a85d5af5bf1d1762f925bdaddc4201f984', balance: '45.23' },
    { symbol: 'LINK', name: 'Chainlink', address: '0x514910771af9ca656af840dff83e8264ecf986ca', balance: '89.67' },
    { symbol: 'AAVE', name: 'Aave', address: '0x7fc66500c84a76ad7e9c93437bfc5ac33e2ddae9', balance: '12.45' },
  ]

  const handleGetQuote = useCallback(async () => {
    if (!amountIn || parseFloat(amountIn) === 0) return

    setIsLoading(true)
    setError(null)

    try {
      // Use real API to get quote
      await getQuote(fromToken.address, toToken.address, amountIn)
      // Update amount out from quote if successful
      if (quote) {
        setAmountOut(quote.output_amount)
      }
    } catch (err) {
      console.error('Error getting quote:', err)
      setError('Failed to get swap quote')
      
      // Fallback to mock calculation
      const mockPrice = fromToken.symbol === 'ETH' ? 1750 : 1 / 1750
      const calculatedAmountOut = (parseFloat(amountIn) * mockPrice * (1 - slippage / 100)).toString()
      setAmountOut(calculatedAmountOut)
    } finally {
      setIsLoading(false)
    }
  }, [amountIn, fromToken.address, fromToken.symbol, toToken.address, slippage, getQuote, quote])

  useEffect(() => {
    if (amountIn && parseFloat(amountIn) > 0) {
      handleGetQuote()
    } else {
      setAmountOut('')
    }
  }, [amountIn, handleGetQuote])

  const swapTokens = () => {
    const temp = fromToken
    setFromToken(toToken)
    setToToken(temp)
    setAmountIn(amountOut)
    setAmountOut(amountIn)
  }

  const executeSwap = async () => {
    if (!quote) return

    setIsLoading(true)
    try {
      // Simulate swap execution
      await new Promise(resolve => setTimeout(resolve, 2000))
      alert('Swap executed successfully! (This is a demo)')
    } catch {
      setError('Failed to execute swap')
    } finally {
      setIsLoading(false)
    }
  }

  const TokenSelector: React.FC<{
    token: Token
    onSelect: (token: Token) => void
    label: string
  }> = ({ token, onSelect, label }) => (
    <div className="space-y-2">
      <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
        {label}
      </label>
      <select
        value={token.symbol}
        onChange={(e) => {
          const selectedToken = tokens.find(t => t.symbol === e.target.value)
          if (selectedToken) onSelect(selectedToken)
        }}
        className="w-full p-3 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
      >
        {tokens.map((t) => (
          <option key={t.symbol} value={t.symbol}>
            {t.symbol} - {t.name}
          </option>
        ))}
      </select>
      {token.balance && (
        <p className="text-sm text-gray-500 dark:text-gray-400">
          Balance: {token.balance} {token.symbol}
        </p>
      )}
    </div>
  )

  return (
    <div className="space-y-6">
      {/* Swap Interface */}
      <div className="card max-w-lg mx-auto">
        <div className="card-header">
          <div className="flex items-center justify-between">
            <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
              Token Swap
            </h2>
            <button className="p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200">
              <Settings className="h-4 w-4" />
            </button>
          </div>
        </div>
        <div className="card-content space-y-4">
          {/* From Token */}
          <div className="p-4 bg-gray-50 dark:bg-gray-800 rounded-lg">
            <TokenSelector
              token={fromToken}
              onSelect={setFromToken}
              label="From"
            />
            <div className="mt-3">
              <input
                type="number"
                placeholder="0.0"
                value={amountIn}
                onChange={(e) => setAmountIn(e.target.value)}
                className="w-full p-3 text-2xl bg-transparent border-none outline-none text-gray-900 dark:text-white"
              />
            </div>
          </div>

          {/* Swap Button */}
          <div className="flex justify-center">
            <button
              onClick={swapTokens}
              className="p-2 border-2 border-gray-200 dark:border-gray-600 rounded-full bg-white dark:bg-gray-800 hover:bg-gray-50 dark:hover:bg-gray-700"
            >
              <ArrowUpDown className="h-5 w-5 text-gray-600 dark:text-gray-400" />
            </button>
          </div>

          {/* To Token */}
          <div className="p-4 bg-gray-50 dark:bg-gray-800 rounded-lg">
            <TokenSelector
              token={toToken}
              onSelect={setToToken}
              label="To"
            />
            <div className="mt-3">
              <input
                type="number"
                placeholder="0.0"
                value={amountOut}
                readOnly
                className="w-full p-3 text-2xl bg-transparent border-none outline-none text-gray-900 dark:text-white"
              />
              {isLoading && (
                <div className="flex items-center mt-2">
                  <RefreshCw className="h-4 w-4 animate-spin mr-2" />
                  <span className="text-sm text-gray-500">Getting best price...</span>
                </div>
              )}
            </div>
          </div>

          {/* Quote Details */}
          {quote && (
            <div className="p-4 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
              <div className="space-y-2 text-sm">
                <div className="flex justify-between">
                  <span className="text-gray-600 dark:text-gray-400">Price Impact</span>
                  <span className={`${quote.price_impact > 2 ? 'text-red-600' : 'text-gray-900 dark:text-white'}`}>
                    {quote.price_impact.toFixed(2)}%
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-600 dark:text-gray-400">Trading Fee</span>
                  <span className="text-gray-900 dark:text-white">{quote.gas_estimate}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-600 dark:text-gray-400">Slippage Tolerance</span>
                  <span className="text-gray-900 dark:text-white">{slippage}%</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-600 dark:text-gray-400">Route</span>
                  <span className="text-gray-900 dark:text-white">{quote.route.join(' → ')}</span>
                </div>
              </div>
            </div>
          )}

          {/* Error Display */}
          {error && (
            <div className="p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
              <div className="flex items-center">
                <AlertCircle className="h-4 w-4 text-red-600 dark:text-red-400 mr-2" />
                <span className="text-red-600 dark:text-red-400 text-sm">{error}</span>
              </div>
            </div>
          )}

          {/* Slippage Settings */}
          <div className="p-4 border border-gray-200 dark:border-gray-600 rounded-lg">
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Slippage Tolerance
            </label>
            <div className="flex space-x-2">
              {[0.1, 0.5, 1.0].map((value) => (
                <button
                  key={value}
                  onClick={() => setSlippage(value)}
                  className={`px-3 py-1 rounded text-sm ${
                    slippage === value
                      ? 'bg-blue-600 text-white'
                      : 'bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300'
                  }`}
                >
                  {value}%
                </button>
              ))}
              <input
                type="number"
                step="0.1"
                min="0.1"
                max="10"
                value={slippage}
                onChange={(e) => setSlippage(parseFloat(e.target.value) || 0.5)}
                className="w-20 px-2 py-1 text-sm border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-800"
              />
            </div>
          </div>

          {/* Execute Swap Button */}
          <button
            onClick={executeSwap}
            disabled={!quote || isLoading || !amountIn}
            className="w-full btn-primary disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {isLoading ? 'Processing...' : 'Swap Tokens'}
          </button>
        </div>
      </div>

      {/* Recent Transactions */}
      <div className="card">
        <div className="card-header">
          <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
            Recent Transactions
          </h2>
        </div>
        <div className="card-content">
          <div className="text-center py-8 text-gray-500 dark:text-gray-400">
            <TrendingUp className="h-12 w-12 mx-auto mb-4 opacity-50" />
            <p>No recent transactions</p>
            <p className="text-sm">Your trading history will appear here</p>
          </div>
        </div>
      </div>
    </div>
  )
}

export default TradingDashboard
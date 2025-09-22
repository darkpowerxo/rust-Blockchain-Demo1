import React, { useState, useEffect, useCallback } from 'react'
import { TrendingUp, TrendingDown, RefreshCw, Eye, EyeOff } from 'lucide-react'

interface TokenBalance {
  symbol: string
  name: string
  balance: string
  value: number
  change24h: number
  address: string
}

interface PortfolioSummary {
  totalValue: number
  change24h: number
  changePercent24h: number
}

// Mock token balances for demonstration
const mockTokenBalances: TokenBalance[] = [
  {
    symbol: 'ETH',
    name: 'Ethereum',
    balance: '2.45',
    value: 4902.35,
    change24h: -2.34,
    address: '0x0000000000000000000000000000000000000000',
  },
  {
    symbol: 'USDC',
    name: 'USD Coin',
    balance: '1,234.56',
    value: 1234.56,
    change24h: 0.01,
    address: '0xa0b86a33e6988c4d2b3f76bffd14d5b5e02b8be5',
  },
  {
    symbol: 'UNI',
    name: 'Uniswap',
    balance: '125.89',
    value: 1007.12,
    change24h: 5.67,
    address: '0x1f9840a85d5af5bf1d1762f925bdaddc4201f984',
  },
  {
    symbol: 'AAVE',
    name: 'Aave',
    balance: '12.34',
    value: 987.23,
    change24h: 3.45,
    address: '0x7fc66500c84a76ad7e9c93437bfc5ac33e2ddae9',
  },
  {
    symbol: 'LINK',
    name: 'Chainlink',
    balance: '89.67',
    value: 623.69,
    change24h: 1.23,
    address: '0x514910771af9ca656af840dff83e8264ecf986ca',
  },
]

const Portfolio: React.FC = () => {
  const [portfolioSummary, setPortfolioSummary] = useState<PortfolioSummary>({
    totalValue: 0,
    change24h: 0,
    changePercent24h: 0,
  })
  
  const [tokenBalances, setTokenBalances] = useState<TokenBalance[]>([])
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [hideSmallBalances, setHideSmallBalances] = useState(false)
  const [isBalanceVisible, setIsBalanceVisible] = useState(true)

  const loadPortfolioData = useCallback(async () => {
    setIsLoading(true)
    setError(null)
    
    try {
      // In a real implementation, this would call the backend API
      // const response = await fetch('http://localhost:3000/api/portfolio')
      // const data = await response.json()
      
      // Simulate API delay
      await new Promise(resolve => setTimeout(resolve, 1000))
      
      const totalValue = mockTokenBalances.reduce((sum, token) => sum + token.value, 0)
      const totalChange24h = mockTokenBalances.reduce((sum, token) => sum + (token.value * token.change24h / 100), 0)
      const changePercent24h = (totalValue > 0) ? (totalChange24h / totalValue) * 100 : 0

      setTokenBalances(mockTokenBalances)
      setPortfolioSummary({
        totalValue,
        change24h: totalChange24h,
        changePercent24h,
      })
      
      setIsLoading(false)
    } catch {
      setError('Failed to load portfolio data')
      setIsLoading(false)
    }
  }, [])

  useEffect(() => {
    loadPortfolioData()
  }, [loadPortfolioData])

  const formatCurrency = (amount: number): string => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
    }).format(amount)
  }

  const formatPercent = (percent: number): string => {
    return `${percent >= 0 ? '+' : ''}${percent.toFixed(2)}%`
  }

  const filteredTokens = hideSmallBalances 
    ? tokenBalances.filter(token => token.value >= 1)
    : tokenBalances

  const handleRefresh = () => {
    loadPortfolioData()
  }

  const toggleBalanceVisibility = () => {
    setIsBalanceVisible(!isBalanceVisible)
  }

  const toggleSmallBalances = () => {
    setHideSmallBalances(!hideSmallBalances)
  }

  if (isLoading) {
    return (
      <div className="p-6 max-w-7xl mx-auto">
        <div className="animate-pulse">
          <div className="h-8 bg-gray-200 dark:bg-gray-700 rounded mb-4"></div>
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-4 mb-6">
            <div className="h-24 bg-gray-200 dark:bg-gray-700 rounded"></div>
            <div className="h-24 bg-gray-200 dark:bg-gray-700 rounded"></div>
            <div className="h-24 bg-gray-200 dark:bg-gray-700 rounded"></div>
          </div>
          <div className="h-64 bg-gray-200 dark:bg-gray-700 rounded"></div>
        </div>
      </div>
    )
  }

  if (error) {
    return (
      <div className="p-6 max-w-7xl mx-auto">
        <div className="bg-red-50 dark:bg-red-900/50 border border-red-200 dark:border-red-800 rounded-lg p-4">
          <div className="text-red-800 dark:text-red-200 font-medium">Error</div>
          <div className="text-red-700 dark:text-red-300">{error}</div>
          <button
            onClick={handleRefresh}
            className="mt-2 px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors"
          >
            Retry
          </button>
        </div>
      </div>
    )
  }

  return (
    <div className="p-6 max-w-7xl mx-auto">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between mb-6">
        <h1 className="text-2xl font-bold text-gray-900 dark:text-white mb-4 sm:mb-0">
          Portfolio Overview
        </h1>
        <div className="flex items-center space-x-2">
          <button
            onClick={toggleBalanceVisibility}
            className="p-2 text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 transition-colors"
            title={isBalanceVisible ? 'Hide balances' : 'Show balances'}
          >
            {isBalanceVisible ? <EyeOff size={20} /> : <Eye size={20} />}
          </button>
          <button
            onClick={handleRefresh}
            className="flex items-center space-x-2 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
          >
            <RefreshCw size={16} />
            <span>Refresh</span>
          </button>
        </div>
      </div>

      {/* Portfolio Summary */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
        <div className="bg-white dark:bg-gray-800 p-6 rounded-xl border border-gray-200 dark:border-gray-700">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-gray-600 dark:text-gray-400">Total Value</p>
              <p className="text-2xl font-bold text-gray-900 dark:text-white">
                {isBalanceVisible ? formatCurrency(portfolioSummary.totalValue) : '••••••'}
              </p>
            </div>
            <div className="w-12 h-12 bg-blue-100 dark:bg-blue-900/50 rounded-lg flex items-center justify-center">
              <TrendingUp className="w-6 h-6 text-blue-600 dark:text-blue-400" />
            </div>
          </div>
        </div>

        <div className="bg-white dark:bg-gray-800 p-6 rounded-xl border border-gray-200 dark:border-gray-700">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-gray-600 dark:text-gray-400">24h Change</p>
              <p className={`text-2xl font-bold ${
                portfolioSummary.change24h >= 0 
                  ? 'text-green-600 dark:text-green-400' 
                  : 'text-red-600 dark:text-red-400'
              }`}>
                {isBalanceVisible ? formatCurrency(portfolioSummary.change24h) : '••••••'}
              </p>
            </div>
            <div className={`w-12 h-12 rounded-lg flex items-center justify-center ${
              portfolioSummary.change24h >= 0 
                ? 'bg-green-100 dark:bg-green-900/50' 
                : 'bg-red-100 dark:bg-red-900/50'
            }`}>
              {portfolioSummary.change24h >= 0 ? (
                <TrendingUp className="w-6 h-6 text-green-600 dark:text-green-400" />
              ) : (
                <TrendingDown className="w-6 h-6 text-red-600 dark:text-red-400" />
              )}
            </div>
          </div>
        </div>

        <div className="bg-white dark:bg-gray-800 p-6 rounded-xl border border-gray-200 dark:border-gray-700">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-gray-600 dark:text-gray-400">24h Change %</p>
              <p className={`text-2xl font-bold ${
                portfolioSummary.changePercent24h >= 0 
                  ? 'text-green-600 dark:text-green-400' 
                  : 'text-red-600 dark:text-red-400'
              }`}>
                {isBalanceVisible ? formatPercent(portfolioSummary.changePercent24h) : '••••••'}
              </p>
            </div>
            <div className={`w-12 h-12 rounded-lg flex items-center justify-center ${
              portfolioSummary.changePercent24h >= 0 
                ? 'bg-green-100 dark:bg-green-900/50' 
                : 'bg-red-100 dark:bg-red-900/50'
            }`}>
              {portfolioSummary.changePercent24h >= 0 ? (
                <TrendingUp className="w-6 h-6 text-green-600 dark:text-green-400" />
              ) : (
                <TrendingDown className="w-6 h-6 text-red-600 dark:text-red-400" />
              )}
            </div>
          </div>
        </div>
      </div>

      {/* Token Balances */}
      <div className="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700">
        <div className="p-6 border-b border-gray-200 dark:border-gray-700">
          <div className="flex items-center justify-between">
            <h2 className="text-lg font-semibold text-gray-900 dark:text-white">Token Balances</h2>
            <label className="flex items-center space-x-2 text-sm text-gray-600 dark:text-gray-400">
              <input
                type="checkbox"
                checked={hideSmallBalances}
                onChange={toggleSmallBalances}
                className="rounded border-gray-300 dark:border-gray-600 text-blue-600 focus:ring-blue-500"
              />
              <span>Hide small balances</span>
            </label>
          </div>
        </div>
        
        <div className="overflow-x-auto">
          <table className="w-full">
            <thead>
              <tr className="border-b border-gray-200 dark:border-gray-700">
                <th className="text-left py-3 px-6 text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                  Token
                </th>
                <th className="text-left py-3 px-6 text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                  Balance
                </th>
                <th className="text-left py-3 px-6 text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                  Value
                </th>
                <th className="text-left py-3 px-6 text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                  24h Change
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-200 dark:divide-gray-700">
              {filteredTokens.map((token) => (
                <tr key={token.address} className="hover:bg-gray-50 dark:hover:bg-gray-700/50">
                  <td className="py-4 px-6">
                    <div className="flex items-center">
                      <div className="w-8 h-8 bg-gray-200 dark:bg-gray-600 rounded-full flex items-center justify-center mr-3">
                        <span className="text-xs font-medium text-gray-600 dark:text-gray-300">
                          {token.symbol.substring(0, 2)}
                        </span>
                      </div>
                      <div>
                        <div className="text-sm font-medium text-gray-900 dark:text-white">
                          {token.symbol}
                        </div>
                        <div className="text-xs text-gray-500 dark:text-gray-400">
                          {token.name}
                        </div>
                      </div>
                    </div>
                  </td>
                  <td className="py-4 px-6 text-sm text-gray-900 dark:text-white">
                    {isBalanceVisible ? token.balance : '••••••'}
                  </td>
                  <td className="py-4 px-6 text-sm font-medium text-gray-900 dark:text-white">
                    {isBalanceVisible ? formatCurrency(token.value) : '••••••'}
                  </td>
                  <td className="py-4 px-6">
                    <div className={`flex items-center text-sm font-medium ${
                      token.change24h >= 0 
                        ? 'text-green-600 dark:text-green-400' 
                        : 'text-red-600 dark:text-red-400'
                    }`}>
                      {token.change24h >= 0 ? (
                        <TrendingUp size={16} className="mr-1" />
                      ) : (
                        <TrendingDown size={16} className="mr-1" />
                      )}
                      {isBalanceVisible ? formatPercent(token.change24h) : '••••••'}
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  )
}

export default Portfolio
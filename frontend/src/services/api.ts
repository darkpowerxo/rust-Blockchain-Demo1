import axios from 'axios'
import type { AxiosResponse } from 'axios'

// API Base Configuration
const API_BASE_URL = 'http://localhost:3000/api/v1'

const api = axios.create({
  baseURL: API_BASE_URL,
  timeout: 10000,
  headers: {
    'Content-Type': 'application/json',
  },
})

// Request interceptor for adding auth tokens if needed
api.interceptors.request.use(
  (config) => {
    // Add auth token here if needed
    // const token = localStorage.getItem('authToken')
    // if (token) {
    //   config.headers.Authorization = `Bearer ${token}`
    // }
    return config
  },
  (error) => {
    return Promise.reject(error)
  }
)

// Response interceptor for error handling
api.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401) {
      // Handle unauthorized access
      console.error('Unauthorized access')
    }
    return Promise.reject(error)
  }
)

// Type definitions for API responses
export interface DefiProtocolStats {
  name: string
  tvl: string
  total_borrowed: string
  total_supplied: string
  utilization_rate: number
  average_supply_apy: number
  average_borrow_apy: number
  active_users: number
  health_factor: number
}

export interface YieldOpportunity {
  protocol: string
  asset: string
  apy: number
  risk_level: string
  minimum_deposit: string
  available_liquidity: string
}

export interface UserPortfolio {
  user: string
  total_supplied_usd: number
  total_borrowed_usd: number
  net_worth_usd: number
  overall_health_factor: number
  positions: PositionInfo[]
}

export interface PositionInfo {
  protocol: string
  asset: string
  supplied_amount: string
  borrowed_amount: string
  supply_apy: number
  borrow_apy: number
}

export interface TokenInfo {
  address: string
  symbol: string
  name: string
  decimals: number
  balance?: string
  price_usd?: number
}

export interface SecurityAlert {
  id: string
  level: 'LOW' | 'MEDIUM' | 'HIGH' | 'CRITICAL'
  title: string
  description: string
  timestamp: string
  resolved: boolean
}

export interface RiskAssessment {
  address: string
  risk_score: number
  risk_level: 'LOW' | 'MEDIUM' | 'HIGH' | 'CRITICAL'
  factors: string[]
  recommendations: string[]
}

export interface TransactionScanResult {
  tx_hash: string
  risk_score: number
  threats_detected: string[]
  is_safe: boolean
  analysis: {
    gas_analysis: string
    contract_analysis: string
    recipient_analysis: string
  }
}

export interface WalletInfo {
  address: string
  balance: string
  transaction_count: number
  first_seen: string
  last_activity: string
  risk_score: number
}

export interface TradingQuote {
  input_token: string
  output_token: string
  input_amount: string
  output_amount: string
  price_impact: number
  gas_estimate: string
  route: string[]
  slippage_tolerance: number
}

// DeFi API Services
export const defiApi = {
  // Get supported DeFi protocols
  getProtocols: async (): Promise<string[]> => {
    const response: AxiosResponse<string[]> = await api.get('/defi/protocols')
    return response.data
  },

  // Get protocol statistics
  getProtocolStats: async (protocol: string): Promise<DefiProtocolStats> => {
    const response: AxiosResponse<DefiProtocolStats> = await api.get(`/defi/protocols/${protocol}/stats`)
    return response.data
  },

  // Get yield opportunities
  getYieldOpportunities: async (): Promise<YieldOpportunity[]> => {
    const response: AxiosResponse<YieldOpportunity[]> = await api.get('/defi/opportunities')
    return response.data
  },

  // Get user portfolio
  getUserPortfolio: async (userAddress: string): Promise<UserPortfolio> => {
    const response: AxiosResponse<UserPortfolio> = await api.get(`/defi/portfolio/${userAddress}`)
    return response.data
  },

  // Supply asset to protocol
  supplyAsset: async (protocol: string, asset: string, amount: string, user: string): Promise<string> => {
    const response: AxiosResponse<string> = await api.post(`/defi/protocols/${protocol}/supply`, {
      asset,
      amount,
      user,
    })
    return response.data
  },

  // Withdraw asset from protocol
  withdrawAsset: async (protocol: string, asset: string, amount: string, user: string): Promise<string> => {
    const response: AxiosResponse<string> = await api.post(`/defi/protocols/${protocol}/withdraw`, {
      asset,
      amount,
      user,
    })
    return response.data
  },
}

// Trading API Services
export const tradingApi = {
  // Get trading quote
  getQuote: async (inputToken: string, outputToken: string, inputAmount: string): Promise<TradingQuote> => {
    const response: AxiosResponse<TradingQuote> = await api.get('/dex/quote', {
      params: {
        input_token: inputToken,
        output_token: outputToken,
        input_amount: inputAmount,
      },
    })
    return response.data
  },

  // Execute swap
  executeSwap: async (inputToken: string, outputToken: string, inputAmount: string, minOutputAmount: string, user: string): Promise<string> => {
    const response: AxiosResponse<string> = await api.post('/dex/swap', {
      input_token: inputToken,
      output_token: outputToken,
      input_amount: inputAmount,
      min_output_amount: minOutputAmount,
      user,
    })
    return response.data
  },

  // Get supported tokens
  getSupportedTokens: async (): Promise<TokenInfo[]> => {
    const response: AxiosResponse<TokenInfo[]> = await api.get('/tokens')
    return response.data
  },
}

// Security API Services
export const securityApi = {
  // Get security alerts
  getAlerts: async (): Promise<SecurityAlert[]> => {
    const response: AxiosResponse<SecurityAlert[]> = await api.get('/security/alerts')
    return response.data
  },

  // Get risk assessment for address
  getRiskAssessment: async (address: string): Promise<RiskAssessment> => {
    const response: AxiosResponse<RiskAssessment> = await api.get(`/security/risk-assessment/${address}`)
    return response.data
  },

  // Scan transaction for risks
  scanTransaction: async (txHash: string): Promise<TransactionScanResult> => {
    const response: AxiosResponse<TransactionScanResult> = await api.get(`/security/scan-transaction/${txHash}`)
    return response.data
  },
}

// Wallet API Services
export const walletApi = {
  // Get wallet info
  getWalletInfo: async (address: string): Promise<WalletInfo> => {
    const response: AxiosResponse<WalletInfo> = await api.get(`/wallets/${address}`)
    return response.data
  },

  // Get token balances
  getTokenBalances: async (address: string): Promise<TokenInfo[]> => {
    const response: AxiosResponse<TokenInfo[]> = await api.get(`/wallets/${address}/tokens`)
    return response.data
  },
}

// Utility functions
export const utils = {
  // Convert hex string to decimal
  hexToDecimal: (hex: string): number => {
    return parseInt(hex, 16)
  },

  // Format large numbers
  formatNumber: (num: number): string => {
    if (num >= 1e9) {
      return `${(num / 1e9).toFixed(2)}B`
    } else if (num >= 1e6) {
      return `${(num / 1e6).toFixed(2)}M`
    } else if (num >= 1e3) {
      return `${(num / 1e3).toFixed(2)}K`
    }
    return num.toFixed(2)
  },

  // Format percentage
  formatPercentage: (value: number): string => {
    return `${(value * 100).toFixed(2)}%`
  },

  // Format currency
  formatCurrency: (amount: number): string => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
    }).format(amount)
  },
}

export default api
import { useState, useEffect, useCallback } from 'react'
import { defiApi, tradingApi, securityApi, walletApi } from '../services/api'
import type { 
  DefiProtocolStats, 
  YieldOpportunity, 
  UserPortfolio, 
  SecurityAlert, 
  TradingQuote, 
  TokenInfo,
  RiskAssessment,
  TransactionScanResult,
  WalletInfo
} from '../services/api'

// Generic API state interface
interface ApiState<T> {
  data: T | null
  loading: boolean
  error: string | null
}

// Hook for DeFi protocols
export const useDefiProtocols = () => {
  const [state, setState] = useState<ApiState<string[]>>({
    data: null,
    loading: false,
    error: null,
  })

  const fetchProtocols = useCallback(async () => {
    setState(prev => ({ ...prev, loading: true, error: null }))
    try {
      const protocols = await defiApi.getProtocols()
      setState({ data: protocols, loading: false, error: null })
    } catch (error) {
      setState({ data: null, loading: false, error: (error as Error).message })
    }
  }, [])

  useEffect(() => {
    fetchProtocols()
  }, [fetchProtocols])

  return { ...state, refetch: fetchProtocols }
}

// Hook for protocol statistics
export const useProtocolStats = (protocol: string) => {
  const [state, setState] = useState<ApiState<DefiProtocolStats>>({
    data: null,
    loading: false,
    error: null,
  })

  const fetchStats = useCallback(async () => {
    if (!protocol) return
    setState(prev => ({ ...prev, loading: true, error: null }))
    try {
      const stats = await defiApi.getProtocolStats(protocol)
      setState({ data: stats, loading: false, error: null })
    } catch (error) {
      setState({ data: null, loading: false, error: (error as Error).message })
    }
  }, [protocol])

  useEffect(() => {
    fetchStats()
  }, [fetchStats])

  return { ...state, refetch: fetchStats }
}

// Hook for yield opportunities
export const useYieldOpportunities = () => {
  const [state, setState] = useState<ApiState<YieldOpportunity[]>>({
    data: null,
    loading: false,
    error: null,
  })

  const fetchOpportunities = useCallback(async () => {
    setState(prev => ({ ...prev, loading: true, error: null }))
    try {
      const opportunities = await defiApi.getYieldOpportunities()
      setState({ data: opportunities, loading: false, error: null })
    } catch (error) {
      setState({ data: null, loading: false, error: (error as Error).message })
    }
  }, [])

  useEffect(() => {
    fetchOpportunities()
  }, [fetchOpportunities])

  return { ...state, refetch: fetchOpportunities }
}

// Hook for user portfolio
export const useUserPortfolio = (userAddress: string | null) => {
  const [state, setState] = useState<ApiState<UserPortfolio>>({
    data: null,
    loading: false,
    error: null,
  })

  const fetchPortfolio = useCallback(async () => {
    if (!userAddress) return
    setState(prev => ({ ...prev, loading: true, error: null }))
    try {
      const portfolio = await defiApi.getUserPortfolio(userAddress)
      setState({ data: portfolio, loading: false, error: null })
    } catch (error) {
      setState({ data: null, loading: false, error: (error as Error).message })
    }
  }, [userAddress])

  useEffect(() => {
    fetchPortfolio()
  }, [fetchPortfolio])

  return { ...state, refetch: fetchPortfolio }
}

// Hook for trading quotes
export const useTradingQuote = () => {
  const [state, setState] = useState<ApiState<TradingQuote>>({
    data: null,
    loading: false,
    error: null,
  })

  const getQuote = useCallback(async (inputToken: string, outputToken: string, inputAmount: string) => {
    setState(prev => ({ ...prev, loading: true, error: null }))
    try {
      const quote = await tradingApi.getQuote(inputToken, outputToken, inputAmount)
      setState({ data: quote, loading: false, error: null })
    } catch (error) {
      setState({ data: null, loading: false, error: (error as Error).message })
    }
  }, [])

  return { ...state, getQuote }
}

// Hook for supported tokens
export const useSupportedTokens = () => {
  const [state, setState] = useState<ApiState<TokenInfo[]>>({
    data: null,
    loading: false,
    error: null,
  })

  const fetchTokens = useCallback(async () => {
    setState(prev => ({ ...prev, loading: true, error: null }))
    try {
      const tokens = await tradingApi.getSupportedTokens()
      setState({ data: tokens, loading: false, error: null })
    } catch (error) {
      setState({ data: null, loading: false, error: (error as Error).message })
    }
  }, [])

  useEffect(() => {
    fetchTokens()
  }, [fetchTokens])

  return { ...state, refetch: fetchTokens }
}

// Hook for security alerts
export const useSecurityAlerts = () => {
  const [state, setState] = useState<ApiState<SecurityAlert[]>>({
    data: null,
    loading: false,
    error: null,
  })

  const fetchAlerts = useCallback(async () => {
    setState(prev => ({ ...prev, loading: true, error: null }))
    try {
      const alerts = await securityApi.getAlerts()
      setState({ data: alerts, loading: false, error: null })
    } catch (error) {
      setState({ data: null, loading: false, error: (error as Error).message })
    }
  }, [])

  useEffect(() => {
    fetchAlerts()
  }, [fetchAlerts])

  return { ...state, refetch: fetchAlerts }
}

// Hook for risk assessment
export const useRiskAssessment = () => {
  const [state, setState] = useState<ApiState<RiskAssessment>>({
    data: null,
    loading: false,
    error: null,
  })

  const assessRisk = useCallback(async (address: string) => {
    setState(prev => ({ ...prev, loading: true, error: null }))
    try {
      const assessment = await securityApi.getRiskAssessment(address)
      setState({ data: assessment, loading: false, error: null })
    } catch (error) {
      setState({ data: null, loading: false, error: (error as Error).message })
    }
  }, [])

  return { ...state, assessRisk }
}

// Hook for transaction scanning
export const useTransactionScan = () => {
  const [state, setState] = useState<ApiState<TransactionScanResult>>({
    data: null,
    loading: false,
    error: null,
  })

  const scanTransaction = useCallback(async (txHash: string) => {
    setState(prev => ({ ...prev, loading: true, error: null }))
    try {
      const result = await securityApi.scanTransaction(txHash)
      setState({ data: result, loading: false, error: null })
    } catch (error) {
      setState({ data: null, loading: false, error: (error as Error).message })
    }
  }, [])

  return { ...state, scanTransaction }
}

// Hook for wallet info
export const useWalletInfo = (address: string | null) => {
  const [state, setState] = useState<ApiState<WalletInfo>>({
    data: null,
    loading: false,
    error: null,
  })

  const fetchWalletInfo = useCallback(async () => {
    if (!address) return
    setState(prev => ({ ...prev, loading: true, error: null }))
    try {
      const walletInfo = await walletApi.getWalletInfo(address)
      setState({ data: walletInfo, loading: false, error: null })
    } catch (error) {
      setState({ data: null, loading: false, error: (error as Error).message })
    }
  }, [address])

  useEffect(() => {
    fetchWalletInfo()
  }, [fetchWalletInfo])

  return { ...state, refetch: fetchWalletInfo }
}

// Hook for token balances
export const useTokenBalances = (address: string | null) => {
  const [state, setState] = useState<ApiState<TokenInfo[]>>({
    data: null,
    loading: false,
    error: null,
  })

  const fetchBalances = useCallback(async () => {
    if (!address) return
    setState(prev => ({ ...prev, loading: true, error: null }))
    try {
      const balances = await walletApi.getTokenBalances(address)
      setState({ data: balances, loading: false, error: null })
    } catch (error) {
      setState({ data: null, loading: false, error: (error as Error).message })
    }
  }, [address])

  useEffect(() => {
    fetchBalances()
  }, [fetchBalances])

  return { ...state, refetch: fetchBalances }
}
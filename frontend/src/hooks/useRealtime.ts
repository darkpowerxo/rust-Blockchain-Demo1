import { useEffect, useRef, useCallback, useState } from 'react'
import { defiApi } from '../services/api'
import type { DefiProtocolStats, YieldOpportunity } from '../services/api'

interface RealtimeData {
  protocols: string[]
  aaveStats: DefiProtocolStats | null
  opportunities: YieldOpportunity[]
  lastUpdate: number
}

interface UseRealtimeOptions {
  interval?: number
  enabled?: boolean
}

export const useRealtime = (options: UseRealtimeOptions = {}) => {
  const { interval = 10000, enabled = true } = options // Poll every 10 seconds
  const dataRef = useRef<RealtimeData>({
    protocols: [],
    aaveStats: null,
    opportunities: [],
    lastUpdate: 0
  })
  const intervalRef = useRef<NodeJS.Timeout | null>(null)
  const callbacksRef = useRef<Set<(data: RealtimeData) => void>>(new Set())

  const updateData = useCallback(async () => {
    try {
      const [protocols, aaveStats, opportunities] = await Promise.allSettled([
        defiApi.getProtocols(),
        defiApi.getProtocolStats('aave'),
        defiApi.getYieldOpportunities()
      ])

      const newData: RealtimeData = {
        protocols: protocols.status === 'fulfilled' ? protocols.value : dataRef.current.protocols,
        aaveStats: aaveStats.status === 'fulfilled' ? aaveStats.value : dataRef.current.aaveStats,
        opportunities: opportunities.status === 'fulfilled' ? opportunities.value : dataRef.current.opportunities,
        lastUpdate: Date.now()
      }

      dataRef.current = newData
      
      // Notify all subscribers
      callbacksRef.current.forEach(callback => {
        try {
          callback(newData)
        } catch (error) {
          console.error('Error in realtime callback:', error)
        }
      })
    } catch (error) {
      console.error('Error updating realtime data:', error)
    }
  }, [])

  const startPolling = useCallback(() => {
    if (intervalRef.current) {
      clearInterval(intervalRef.current)
    }
    
    // Initial fetch
    updateData()
    
    // Set up polling
    intervalRef.current = setInterval(updateData, interval)
  }, [updateData, interval])

  const stopPolling = useCallback(() => {
    if (intervalRef.current) {
      clearInterval(intervalRef.current)
      intervalRef.current = null
    }
  }, [])

  const subscribe = useCallback((callback: (data: RealtimeData) => void) => {
    callbacksRef.current.add(callback)
    
    // Send current data immediately
    callback(dataRef.current)
    
    return () => {
      callbacksRef.current.delete(callback)
    }
  }, [])

  useEffect(() => {
    if (enabled) {
      startPolling()
    } else {
      stopPolling()
    }

    return () => {
      stopPolling()
    }
  }, [enabled, startPolling, stopPolling])

  return {
    subscribe,
    startPolling,
    stopPolling,
    getCurrentData: () => dataRef.current
  }
}

// Hook for components to use realtime data
export const useRealtimeData = (options: UseRealtimeOptions = {}) => {
  const [data, setData] = useState<RealtimeData>({
    protocols: [],
    aaveStats: null,
    opportunities: [],
    lastUpdate: 0
  })
  const [isConnected, setIsConnected] = useState(false)
  
  const realtime = useRealtime(options)

  useEffect(() => {
    setIsConnected(true)
    const unsubscribe = realtime.subscribe((newData) => {
      setData(newData)
    })

    return () => {
      unsubscribe()
      setIsConnected(false)
    }
  }, [realtime])

  return {
    data,
    isConnected,
    lastUpdate: new Date(data.lastUpdate)
  }
}

// Price simulation service for demo purposes
export class PriceSimulator {
  private volatility = 0.02 // 2% volatility
  private trend = 0.001 // Small upward trend
  private subscribers = new Set<(prices: Record<string, number>) => void>()
  private interval: NodeJS.Timeout | null = null
  private currentPrices: Record<string, number> = {
    ETH: 1750,
    BTC: 42000,
    USDC: 1.00,
    USDT: 1.00,
    UNI: 8.5,
    AAVE: 87,
    LINK: 15.2,
    COMP: 45,
  }

  start() {
    if (this.interval) return

    this.interval = setInterval(() => {
      // Update ETH with realistic price movement
      const change = (Math.random() - 0.5) * this.volatility + this.trend
      this.currentPrices.ETH *= (1 + change)
      
      // Update other tokens with correlated movement
      Object.keys(this.currentPrices).forEach(token => {
        if (token === 'ETH' || token === 'USDC' || token === 'USDT') return
        
        const correlation = Math.random() * 0.7 + 0.3 // 30-100% correlation with ETH
        const independentMove = (Math.random() - 0.5) * this.volatility * 0.5
        const correlatedMove = change * correlation
        
        this.currentPrices[token] *= (1 + correlatedMove + independentMove)
      })

      // Keep stablecoins stable
      this.currentPrices.USDC = 1.00 + (Math.random() - 0.5) * 0.002
      this.currentPrices.USDT = 1.00 + (Math.random() - 0.5) * 0.002

      // Notify subscribers
      this.subscribers.forEach(callback => {
        try {
          callback({ ...this.currentPrices })
        } catch (error) {
          console.error('Error in price subscriber:', error)
        }
      })
    }, 5000) // Update every 5 seconds
  }

  stop() {
    if (this.interval) {
      clearInterval(this.interval)
      this.interval = null
    }
  }

  subscribe(callback: (prices: Record<string, number>) => void) {
    this.subscribers.add(callback)
    // Send current prices immediately
    callback({ ...this.currentPrices })
    
    return () => {
      this.subscribers.delete(callback)
    }
  }

  getCurrentPrices() {
    return { ...this.currentPrices }
  }
}

// Singleton instance
export const priceSimulator = new PriceSimulator()

// Hook for price data
export const useLivePrices = () => {
  const [prices, setPrices] = useState<Record<string, number>>({})
  const [isActive, setIsActive] = useState(false)

  useEffect(() => {
    setIsActive(true)
    priceSimulator.start()
    
    const unsubscribe = priceSimulator.subscribe(setPrices)

    return () => {
      unsubscribe()
      setIsActive(false)
    }
  }, [])

  return {
    prices,
    isActive,
    getPrice: (token: string) => prices[token] || 0,
    formatPrice: (token: string) => {
      const price = prices[token]
      if (!price) return '$0.00'
      
      if (price < 1) {
        return `$${price.toFixed(4)}`
      } else if (price < 100) {
        return `$${price.toFixed(2)}`
      } else {
        return `$${Math.round(price).toLocaleString()}`
      }
    }
  }
}
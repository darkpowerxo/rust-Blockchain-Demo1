import React, { useState } from 'react'
import { useDefiProtocols, useProtocolStats, useYieldOpportunities } from '../hooks/useApi'
import { useRealtimeData, useLivePrices } from '../hooks/useRealtime'
import { utils } from '../services/api'
import { TrendingUp, DollarSign, Users, Zap, RefreshCw, Activity, Wifi, WifiOff, Clock, BarChart3 } from 'lucide-react'

const DefiDashboard: React.FC = () => {
  const { data: protocols, loading: protocolsLoading, error: protocolsError, refetch: refetchProtocols } = useDefiProtocols()
  const { data: aaveStats, loading: aaveLoading, error: aaveError, refetch: refetchAave } = useProtocolStats('aave')
  const { data: opportunities, loading: opportunitiesLoading, error: opportunitiesError, refetch: refetchOpportunities } = useYieldOpportunities()
  
  // Real-time features
  const { data: realtimeData, isConnected, lastUpdate } = useRealtimeData({ interval: 8000 })
  const { prices, isActive: pricesActive, formatPrice, getPrice } = useLivePrices()
  const [showRealtime, setShowRealtime] = useState(true)

  // Use real-time data when available, fallback to API data
  const displayProtocols = realtimeData.protocols.length > 0 ? realtimeData.protocols : protocols
  const displayAaveStats = realtimeData.aaveStats || aaveStats

  if (protocolsLoading || aaveLoading || opportunitiesLoading) {
    return (
      <div className="min-h-screen bg-gray-50 dark:bg-gray-900 p-6">
        <div className="max-w-7xl mx-auto">
          <div className="text-center py-12">
            <RefreshCw className="h-8 w-8 animate-spin text-blue-600 mx-auto mb-4" />
            <p className="text-gray-600 dark:text-gray-400">Loading DeFi data...</p>
          </div>
        </div>
      </div>
    )
  }

  if (protocolsError || aaveError || opportunitiesError) {
    return (
      <div className="min-h-screen bg-gray-50 dark:bg-gray-900 p-6">
        <div className="max-w-7xl mx-auto">
          <div className="text-center py-12">
            <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-6">
              <h3 className="text-lg font-semibold text-red-800 dark:text-red-200 mb-2">
                Error Loading Data
              </h3>
              <p className="text-red-600 dark:text-red-400 mb-4">
                {protocolsError || aaveError || opportunitiesError}
              </p>
              <button
                onClick={() => {
                  refetchProtocols()
                  refetchAave()
                  refetchOpportunities()
                }}
                className="bg-red-600 text-white px-4 py-2 rounded-lg hover:bg-red-700 transition-colors"
              >
                Retry
              </button>
            </div>
          </div>
        </div>
      </div>
    )
  }

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900 p-6">
      <div className="max-w-7xl mx-auto space-y-8">
        {/* Header with Real-time Status */}
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold text-gray-900 dark:text-white flex items-center gap-3">
              DeFi Protocol Dashboard
              <div className={`flex items-center gap-2 text-sm font-normal ${isConnected ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'}`}>
                {isConnected ? <Wifi className="h-4 w-4" /> : <WifiOff className="h-4 w-4" />}
                <span>{isConnected ? 'Live' : 'Offline'}</span>
              </div>
            </h1>
            <p className="text-gray-600 dark:text-gray-400 mt-2 flex items-center gap-2">
              Real-time data from blockchain protocols
              {lastUpdate.getTime() > 0 && (
                <span className="text-xs flex items-center gap-1">
                  <Clock className="h-3 w-3" />
                  Updated {Math.round((Date.now() - lastUpdate.getTime()) / 1000)}s ago
                </span>
              )}
            </p>
          </div>
          <div className="flex items-center gap-3">
            <button
              onClick={() => setShowRealtime(!showRealtime)}
              className={`px-4 py-2 rounded-lg font-medium transition-colors ${showRealtime ? 'bg-green-100 text-green-700 dark:bg-green-900/20 dark:text-green-400' : 'bg-gray-100 text-gray-700 dark:bg-gray-700 dark:text-gray-300'}`}
            >
              <Activity className="h-4 w-4 inline mr-2" />
              Real-time {showRealtime ? 'On' : 'Off'}
            </button>
            <button
              onClick={() => {
                refetchProtocols()
                refetchAave()
                refetchOpportunities()
              }}
              className="flex items-center gap-2 bg-blue-600 text-white px-4 py-2 rounded-lg hover:bg-blue-700 transition-colors"
            >
              <RefreshCw className="h-4 w-4" />
              Refresh Data
            </button>
          </div>
        </div>

        {/* Live Price Ticker */}
        {pricesActive && showRealtime && (
          <div className="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 p-4">
            <h2 className="text-lg font-semibold text-gray-900 dark:text-white mb-4 flex items-center gap-2">
              <BarChart3 className="h-5 w-5 text-blue-600" />
              Live Token Prices
            </h2>
            <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-8 gap-4">
              {Object.entries(prices).map(([token, price]) => {
                const prevPrice = getPrice(token) || price
                const isUp = price >= prevPrice
                const change = ((price - prevPrice) / prevPrice) * 100

                return (
                  <div key={token} className="text-center p-3 bg-gray-50 dark:bg-gray-700 rounded-lg">
                    <div className="font-semibold text-gray-900 dark:text-white text-sm">{token}</div>
                    <div className="text-lg font-bold text-gray-900 dark:text-white">
                      {formatPrice(token)}
                    </div>
                    <div className={`text-xs flex items-center justify-center gap-1 ${isUp ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'}`}>
                      {isUp ? <TrendingUp className="h-3 w-3" /> : <TrendingUp className="h-3 w-3 rotate-180" />}
                      {Math.abs(change) > 0.01 ? `${change.toFixed(2)}%` : '0.00%'}
                    </div>
                  </div>
                )
              })}
            </div>
          </div>
        )}

        {/* Supported Protocols */}
        <div className="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 p-6">
          <h2 className="text-xl font-semibold text-gray-900 dark:text-white mb-4 flex items-center gap-2">
            <Zap className="h-5 w-5 text-blue-600" />
            Supported Protocols
            {isConnected && showRealtime && (
              <div className="ml-auto">
                <div className="flex items-center gap-2 text-sm text-green-600 dark:text-green-400">
                  <div className="h-2 w-2 bg-green-500 rounded-full animate-pulse"></div>
                  Live
                </div>
              </div>
            )}
          </h2>
          <div className="flex flex-wrap gap-3">
            {displayProtocols?.map((protocol) => (
              <div
                key={protocol}
                className="bg-blue-50 dark:bg-blue-900/20 text-blue-700 dark:text-blue-300 px-4 py-2 rounded-full text-sm font-medium border border-blue-200 dark:border-blue-800 hover:bg-blue-100 dark:hover:bg-blue-900/30 transition-colors cursor-pointer"
              >
                {protocol.charAt(0).toUpperCase() + protocol.slice(1)}
              </div>
            ))}
          </div>
        </div>

        {/* Aave Protocol Stats */}
        {displayAaveStats && (
          <div className="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 p-6">
            <h2 className="text-xl font-semibold text-gray-900 dark:text-white mb-6 flex items-center gap-2">
              <img 
                src="https://cryptologos.cc/logos/aave-aave-logo.png" 
                alt="Aave" 
                className="h-5 w-5"
                onError={(e) => {
                  e.currentTarget.style.display = 'none'
                }}
              />
              Aave Protocol Statistics
            </h2>
            
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
              <div className="bg-gradient-to-r from-green-50 to-emerald-50 dark:from-green-900/20 dark:to-emerald-900/20 p-4 rounded-lg border border-green-200 dark:border-green-800">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-green-600 dark:text-green-400 text-sm font-medium">Total Value Locked</p>
                    <p className="text-2xl font-bold text-green-700 dark:text-green-300">
                      {displayAaveStats.tvl}
                    </p>
                  </div>
                  <DollarSign className="h-8 w-8 text-green-600 dark:text-green-400" />
                </div>
              </div>

              <div className="bg-gradient-to-r from-blue-50 to-cyan-50 dark:from-blue-900/20 dark:to-cyan-900/20 p-4 rounded-lg border border-blue-200 dark:border-blue-800">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-blue-600 dark:text-blue-400 text-sm font-medium">Utilization Rate</p>
                    <p className="text-2xl font-bold text-blue-700 dark:text-blue-300">
                      {utils.formatPercentage(displayAaveStats.utilization_rate / 100)}
                    </p>
                  </div>
                  <TrendingUp className="h-8 w-8 text-blue-600 dark:text-blue-400" />
                </div>
              </div>

              <div className="bg-gradient-to-r from-purple-50 to-violet-50 dark:from-purple-900/20 dark:to-violet-900/20 p-4 rounded-lg border border-purple-200 dark:border-purple-800">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-purple-600 dark:text-purple-400 text-sm font-medium">Average Supply APY</p>
                    <p className="text-2xl font-bold text-purple-700 dark:text-purple-300">
                      {utils.formatPercentage(displayAaveStats.average_supply_apy / 100)}
                    </p>
                  </div>
                  <TrendingUp className="h-8 w-8 text-purple-600 dark:text-purple-400" />
                </div>
              </div>

              <div className="bg-gradient-to-r from-orange-50 to-red-50 dark:from-orange-900/20 dark:to-red-900/20 p-4 rounded-lg border border-orange-200 dark:border-orange-800">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-orange-600 dark:text-orange-400 text-sm font-medium">Active Users</p>
                    <p className="text-2xl font-bold text-orange-700 dark:text-orange-300">
                      {utils.formatNumber(displayAaveStats.active_users)}
                    </p>
                  </div>
                  <Users className="h-8 w-8 text-orange-600 dark:text-orange-400" />
                </div>
              </div>
            </div>

            <div className="mt-6 grid grid-cols-1 md:grid-cols-3 gap-4">
              <div className="bg-gray-50 dark:bg-gray-700 p-4 rounded-lg">
                <p className="text-sm text-gray-600 dark:text-gray-400">Total Supplied</p>
                <p className="text-lg font-semibold text-gray-900 dark:text-white">{displayAaveStats.total_supplied}</p>
              </div>
              <div className="bg-gray-50 dark:bg-gray-700 p-4 rounded-lg">
                <p className="text-sm text-gray-600 dark:text-gray-400">Total Borrowed</p>
                <p className="text-lg font-semibold text-gray-900 dark:text-white">{displayAaveStats.total_borrowed}</p>
              </div>
              <div className="bg-gray-50 dark:bg-gray-700 p-4 rounded-lg">
                <p className="text-sm text-gray-600 dark:text-gray-400">Health Factor</p>
                <p className="text-lg font-semibold text-gray-900 dark:text-white">{displayAaveStats.health_factor.toFixed(2)}</p>
              </div>
            </div>
          </div>
        )}

        {/* Yield Opportunities */}
        {opportunities && opportunities.length > 0 && (
          <div className="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 p-6">
            <h2 className="text-xl font-semibold text-gray-900 dark:text-white mb-6 flex items-center gap-2">
              <TrendingUp className="h-5 w-5 text-green-600" />
              Current Yield Opportunities
            </h2>
            
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
              {opportunities.map((opportunity, index) => (
                <div
                  key={index}
                  className="bg-gradient-to-br from-green-50 to-emerald-50 dark:from-green-900/10 dark:to-emerald-900/10 p-6 rounded-lg border border-green-200 dark:border-green-800"
                >
                  <div className="flex items-center justify-between mb-4">
                    <h3 className="font-semibold text-gray-900 dark:text-white">
                      {opportunity.protocol.charAt(0).toUpperCase() + opportunity.protocol.slice(1)}
                    </h3>
                    <span className={`px-2 py-1 rounded text-xs font-medium ${
                      opportunity.risk_level === 'LOW' ? 'bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400' :
                      opportunity.risk_level === 'MEDIUM' ? 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/20 dark:text-yellow-400' :
                      'bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-400'
                    }`}>
                      {opportunity.risk_level}
                    </span>
                  </div>
                  
                  <div className="space-y-3">
                    <div className="flex justify-between">
                      <span className="text-gray-600 dark:text-gray-400">Asset:</span>
                      <span className="font-medium text-gray-900 dark:text-white">{opportunity.asset}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-gray-600 dark:text-gray-400">APY:</span>
                      <span className="font-bold text-green-600 dark:text-green-400">
                        {utils.formatPercentage(opportunity.apy / 100)}
                      </span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-gray-600 dark:text-gray-400">Min Deposit:</span>
                      <span className="font-medium text-gray-900 dark:text-white">{opportunity.minimum_deposit}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-gray-600 dark:text-gray-400">Liquidity:</span>
                      <span className="font-medium text-gray-900 dark:text-white">{opportunity.available_liquidity}</span>
                    </div>
                  </div>
                  
                  <button className="w-full mt-4 bg-green-600 text-white py-2 px-4 rounded-lg hover:bg-green-700 transition-colors font-medium">
                    View Details
                  </button>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* API Connection Status */}
        <div className="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 p-6">
          <h2 className="text-xl font-semibold text-gray-900 dark:text-white mb-4">
            API Connection Status
          </h2>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div className="flex items-center gap-3 p-4 bg-green-50 dark:bg-green-900/20 rounded-lg border border-green-200 dark:border-green-800">
              <div className="h-3 w-3 bg-green-500 rounded-full"></div>
              <span className="text-green-700 dark:text-green-300 font-medium">Protocols API: Connected</span>
            </div>
            <div className="flex items-center gap-3 p-4 bg-green-50 dark:bg-green-900/20 rounded-lg border border-green-200 dark:border-green-800">
              <div className="h-3 w-3 bg-green-500 rounded-full"></div>
              <span className="text-green-700 dark:text-green-300 font-medium">Stats API: Connected</span>
            </div>
            <div className="flex items-center gap-3 p-4 bg-green-50 dark:bg-green-900/20 rounded-lg border border-green-200 dark:border-green-800">
              <div className="h-3 w-3 bg-green-500 rounded-full"></div>
              <span className="text-green-700 dark:text-green-300 font-medium">Opportunities API: Connected</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

export default DefiDashboard
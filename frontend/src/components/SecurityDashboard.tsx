import React, { useState, useEffect } from 'react'
import { Shield, AlertTriangle, CheckCircle, XCircle, RefreshCw, AlertCircle } from 'lucide-react'

interface SecurityAlert {
  id: string
  type: 'high' | 'medium' | 'low' | 'info'
  title: string
  description: string
  timestamp: string
  resolved: boolean
}

interface SecurityMetrics {
  riskScore: number
  threatsDetected: number
  vulnerabilitiesFound: number
  lastScanTime: string
}

interface TransactionSecurity {
  hash: string
  riskLevel: 'low' | 'medium' | 'high'
  issues: string[]
  timestamp: string
  value: string
}

const SecurityDashboard: React.FC = () => {
  const [securityMetrics, setSecurityMetrics] = useState<SecurityMetrics>({
    riskScore: 0,
    threatsDetected: 0,
    vulnerabilitiesFound: 0,
    lastScanTime: '',
  })
  const [alerts, setAlerts] = useState<SecurityAlert[]>([])
  const [recentTransactions, setRecentTransactions] = useState<TransactionSecurity[]>([])
  const [isLoading, setIsLoading] = useState(true)
  const [isScanning, setIsScanning] = useState(false)

  // Mock security data
  const mockAlerts: SecurityAlert[] = [
    {
      id: '1',
      type: 'high',
      title: 'Suspicious Transaction Pattern Detected',
      description: 'Unusual transaction pattern detected from address 0x1234...5678. Recommend additional verification.',
      timestamp: '2025-09-22T10:30:00Z',
      resolved: false,
    },
    {
      id: '2',
      type: 'medium',
      title: 'Price Manipulation Warning',
      description: 'Significant price deviation detected on UNI/ETH pair. Consider waiting before large trades.',
      timestamp: '2025-09-22T09:15:00Z',
      resolved: false,
    },
    {
      id: '3',
      type: 'low',
      title: 'Gas Price Spike',
      description: 'Gas prices are currently 50% above average. Consider delaying non-urgent transactions.',
      timestamp: '2025-09-22T08:45:00Z',
      resolved: true,
    },
    {
      id: '4',
      type: 'info',
      title: 'Security Scan Completed',
      description: 'Regular security scan completed successfully. No new vulnerabilities detected.',
      timestamp: '2025-09-22T08:00:00Z',
      resolved: true,
    },
  ]

  const mockTransactions: TransactionSecurity[] = [
    {
      hash: '0xabc123...def456',
      riskLevel: 'low',
      issues: [],
      timestamp: '2025-09-22T10:45:00Z',
      value: '0.5 ETH',
    },
    {
      hash: '0x789xyz...012abc',
      riskLevel: 'medium',
      issues: ['High gas usage', 'Unusual recipient'],
      timestamp: '2025-09-22T10:30:00Z',
      value: '1.2 ETH',
    },
    {
      hash: '0x456def...789ghi',
      riskLevel: 'high',
      issues: ['MEV bot interaction', 'Price manipulation risk', 'Unverified contract'],
      timestamp: '2025-09-22T10:15:00Z',
      value: '2.8 ETH',
    },
  ]

  useEffect(() => {
    loadSecurityData()
  }, [])

  const loadSecurityData = async () => {
    setIsLoading(true)
    
    try {
      // Simulate API call
      await new Promise(resolve => setTimeout(resolve, 1000))
      
      setSecurityMetrics({
        riskScore: 23,
        threatsDetected: 2,
        vulnerabilitiesFound: 1,
        lastScanTime: new Date().toISOString(),
      })
      
      setAlerts(mockAlerts)
      setRecentTransactions(mockTransactions)
    } catch (error) {
      console.error('Error loading security data:', error)
    } finally {
      setIsLoading(false)
    }
  }

  const runSecurityScan = async () => {
    setIsScanning(true)
    
    try {
      // Simulate security scan
      await new Promise(resolve => setTimeout(resolve, 3000))
      
      // Update metrics after scan
      setSecurityMetrics(prev => ({
        ...prev,
        lastScanTime: new Date().toISOString(),
        riskScore: Math.max(0, prev.riskScore - 5),
      }))
      
      // Add scan completion alert
      const newAlert: SecurityAlert = {
        id: Date.now().toString(),
        type: 'info',
        title: 'Security Scan Completed',
        description: 'Full security scan completed successfully. System is secure.',
        timestamp: new Date().toISOString(),
        resolved: false,
      }
      
      setAlerts(prev => [newAlert, ...prev])
    } catch (error) {
      console.error('Error running security scan:', error)
    } finally {
      setIsScanning(false)
    }
  }

  const dismissAlert = (alertId: string) => {
    setAlerts(prev => prev.map(alert => 
      alert.id === alertId ? { ...alert, resolved: true } : alert
    ))
  }

  const formatTimestamp = (timestamp: string) => {
    return new Date(timestamp).toLocaleString()
  }

  const getRiskScoreColor = (score: number) => {
    if (score < 20) return 'text-green-600 dark:text-green-400'
    if (score < 50) return 'text-yellow-600 dark:text-yellow-400'
    return 'text-red-600 dark:text-red-400'
  }

  const getRiskScoreBackground = (score: number) => {
    if (score < 20) return 'bg-green-100 dark:bg-green-900'
    if (score < 50) return 'bg-yellow-100 dark:bg-yellow-900'
    return 'bg-red-100 dark:bg-red-900'
  }

  const getAlertIcon = (type: SecurityAlert['type']) => {
    switch (type) {
      case 'high':
        return <XCircle className="h-5 w-5 text-red-600" />
      case 'medium':
        return <AlertTriangle className="h-5 w-5 text-yellow-600" />
      case 'low':
        return <AlertCircle className="h-5 w-5 text-blue-600" />
      default:
        return <CheckCircle className="h-5 w-5 text-green-600" />
    }
  }

  const getAlertBorder = (type: SecurityAlert['type']) => {
    switch (type) {
      case 'high':
        return 'border-red-200 dark:border-red-800'
      case 'medium':
        return 'border-yellow-200 dark:border-yellow-800'
      case 'low':
        return 'border-blue-200 dark:border-blue-800'
      default:
        return 'border-green-200 dark:border-green-800'
    }
  }

  if (isLoading) {
    return (
      <div className="space-y-6">
        <div className="animate-pulse">
          <div className="h-32 bg-gray-200 dark:bg-gray-700 rounded-lg"></div>
        </div>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          {[1, 2].map(i => (
            <div key={i} className="animate-pulse">
              <div className="h-64 bg-gray-200 dark:bg-gray-700 rounded-lg"></div>
            </div>
          ))}
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Security Overview */}
      <div className="card">
        <div className="card-header">
          <div className="flex items-center justify-between">
            <h2 className="text-lg font-semibold text-gray-900 dark:text-white flex items-center">
              <Shield className="h-5 w-5 mr-2" />
              Security Overview
            </h2>
            <button
              onClick={runSecurityScan}
              disabled={isScanning}
              className="btn-primary disabled:opacity-50"
            >
              {isScanning ? (
                <>
                  <RefreshCw className="h-4 w-4 mr-2 animate-spin" />
                  Scanning...
                </>
              ) : (
                'Run Security Scan'
              )}
            </button>
          </div>
        </div>
        <div className="card-content">
          <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
            <div className="text-center">
              <div className={`text-3xl font-bold mb-2 ${getRiskScoreColor(securityMetrics.riskScore)}`}>
                {securityMetrics.riskScore}
              </div>
              <div className="text-sm text-gray-500 dark:text-gray-400">Risk Score</div>
              <div className={`mt-2 px-3 py-1 rounded-full text-xs font-medium ${getRiskScoreBackground(securityMetrics.riskScore)} ${getRiskScoreColor(securityMetrics.riskScore)}`}>
                {securityMetrics.riskScore < 20 ? 'Low Risk' : securityMetrics.riskScore < 50 ? 'Medium Risk' : 'High Risk'}
              </div>
            </div>
            <div className="text-center">
              <div className="text-3xl font-bold text-red-600 dark:text-red-400 mb-2">
                {securityMetrics.threatsDetected}
              </div>
              <div className="text-sm text-gray-500 dark:text-gray-400">Active Threats</div>
            </div>
            <div className="text-center">
              <div className="text-3xl font-bold text-yellow-600 dark:text-yellow-400 mb-2">
                {securityMetrics.vulnerabilitiesFound}
              </div>
              <div className="text-sm text-gray-500 dark:text-gray-400">Vulnerabilities</div>
            </div>
            <div className="text-center">
              <div className="text-sm font-medium text-gray-900 dark:text-white mb-1">
                Last Scan
              </div>
              <div className="text-xs text-gray-500 dark:text-gray-400">
                {formatTimestamp(securityMetrics.lastScanTime)}
              </div>
            </div>
          </div>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Security Alerts */}
        <div className="card">
          <div className="card-header">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
              Security Alerts
            </h3>
          </div>
          <div className="card-content">
            <div className="space-y-4 max-h-96 overflow-y-auto">
              {alerts.filter(alert => !alert.resolved).length === 0 ? (
                <div className="text-center py-8 text-gray-500 dark:text-gray-400">
                  <CheckCircle className="h-12 w-12 mx-auto mb-4 opacity-50" />
                  <p>No active alerts</p>
                  <p className="text-sm">Your system is secure</p>
                </div>
              ) : (
                alerts.filter(alert => !alert.resolved).map((alert) => (
                  <div
                    key={alert.id}
                    className={`p-4 border rounded-lg ${getAlertBorder(alert.type)}`}
                  >
                    <div className="flex items-start justify-between">
                      <div className="flex items-start space-x-3">
                        {getAlertIcon(alert.type)}
                        <div className="flex-1 min-w-0">
                          <h4 className="text-sm font-medium text-gray-900 dark:text-white">
                            {alert.title}
                          </h4>
                          <p className="text-sm text-gray-600 dark:text-gray-400 mt-1">
                            {alert.description}
                          </p>
                          <p className="text-xs text-gray-500 dark:text-gray-400 mt-2">
                            {formatTimestamp(alert.timestamp)}
                          </p>
                        </div>
                      </div>
                      <button
                        onClick={() => dismissAlert(alert.id)}
                        className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200"
                      >
                        ×
                      </button>
                    </div>
                  </div>
                ))
              )}
            </div>
          </div>
        </div>

        {/* Transaction Security */}
        <div className="card">
          <div className="card-header">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
              Recent Transaction Analysis
            </h3>
          </div>
          <div className="card-content">
            <div className="space-y-4 max-h-96 overflow-y-auto">
              {recentTransactions.map((tx) => (
                <div
                  key={tx.hash}
                  className="p-4 border border-gray-200 dark:border-gray-700 rounded-lg"
                >
                  <div className="flex items-center justify-between mb-2">
                    <code className="text-sm font-mono text-gray-900 dark:text-white">
                      {tx.hash}
                    </code>
                    <span className={`px-2 py-1 rounded text-xs font-medium ${
                      tx.riskLevel === 'low' 
                        ? 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200'
                        : tx.riskLevel === 'medium'
                        ? 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200'
                        : 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200'
                    }`}>
                      {tx.riskLevel.toUpperCase()} RISK
                    </span>
                  </div>
                  <div className="text-sm text-gray-600 dark:text-gray-400 mb-2">
                    Value: {tx.value} • {formatTimestamp(tx.timestamp)}
                  </div>
                  {tx.issues.length > 0 && (
                    <div className="mt-2">
                      <div className="text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">
                        Security Issues:
                      </div>
                      <div className="flex flex-wrap gap-1">
                        {tx.issues.map((issue, index) => (
                          <span
                            key={index}
                            className="inline-block px-2 py-1 text-xs bg-red-100 dark:bg-red-900 text-red-800 dark:text-red-200 rounded"
                          >
                            {issue}
                          </span>
                        ))}
                      </div>
                    </div>
                  )}
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

export default SecurityDashboard
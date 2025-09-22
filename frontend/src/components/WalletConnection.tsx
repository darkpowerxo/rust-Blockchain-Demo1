import React, { useState, useEffect } from 'react'
import { Wallet, ChevronDown, Copy, ExternalLink, AlertCircle } from 'lucide-react'

interface WalletState {
  isConnected: boolean
  address: string | null
  balance: string | null
  chainId: number | null
  chainName: string
  isConnecting: boolean
}

const WalletConnection: React.FC = () => {
  const [walletState, setWalletState] = useState<WalletState>({
    isConnected: false,
    address: null,
    balance: null,
    chainId: null,
    chainName: 'Unknown',
    isConnecting: false,
  })
  const [isDropdownOpen, setIsDropdownOpen] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const chainNames: { [key: number]: string } = {
    1: 'Ethereum',
    137: 'Polygon',
    42161: 'Arbitrum One',
    56: 'BSC',
    250: 'Fantom',
    43114: 'Avalanche',
  }

  useEffect(() => {
    // Check if wallet is already connected
    checkWalletConnection()
  }, [])

  const checkWalletConnection = async () => {
    if (typeof window.ethereum !== 'undefined') {
      try {
        const accounts = await window.ethereum.request({ method: 'eth_accounts' })
        if (accounts.length > 0) {
          const chainId = await window.ethereum.request({ method: 'eth_chainId' })
          const chainIdNum = parseInt(chainId, 16)
          setWalletState(prev => ({
            ...prev,
            isConnected: true,
            address: accounts[0],
            chainId: chainIdNum,
            chainName: chainNames[chainIdNum] || `Chain ${chainIdNum}`,
          }))
          
          // Get balance
          getBalance(accounts[0])
        }
      } catch (error) {
        console.error('Error checking wallet connection:', error)
      }
    }
  }

  const getBalance = async (address: string) => {
    try {
      const balance = await window.ethereum.request({
        method: 'eth_getBalance',
        params: [address, 'latest']
      })
      const balanceInEth = (parseInt(balance, 16) / Math.pow(10, 18)).toFixed(4)
      setWalletState(prev => ({ ...prev, balance: balanceInEth }))
    } catch (error) {
      console.error('Error getting balance:', error)
    }
  }

  const connectWallet = async () => {
    if (typeof window.ethereum === 'undefined') {
      setError('MetaMask is not installed. Please install MetaMask to continue.')
      return
    }

    setWalletState(prev => ({ ...prev, isConnecting: true }))
    setError(null)

    try {
      const accounts = await window.ethereum.request({ method: 'eth_requestAccounts' })
      const chainId = await window.ethereum.request({ method: 'eth_chainId' })
      const chainIdNum = parseInt(chainId, 16)
      
      setWalletState(prev => ({
        ...prev,
        isConnected: true,
        address: accounts[0],
        chainId: chainIdNum,
        chainName: chainNames[chainIdNum] || `Chain ${chainIdNum}`,
        isConnecting: false,
      }))

      // Get balance
      getBalance(accounts[0])

      // Listen for account changes
      window.ethereum.on('accountsChanged', (accounts: string[]) => {
        if (accounts.length === 0) {
          disconnectWallet()
        } else {
          setWalletState(prev => ({ ...prev, address: accounts[0] }))
          getBalance(accounts[0])
        }
      })

      // Listen for chain changes
      window.ethereum.on('chainChanged', (chainId: string) => {
        const chainIdNum = parseInt(chainId, 16)
        setWalletState(prev => ({
          ...prev,
          chainId: chainIdNum,
          chainName: chainNames[chainIdNum] || `Chain ${chainIdNum}`,
        }))
      })

    } catch (error: any) {
      console.error('Error connecting wallet:', error)
      setError(error.message || 'Failed to connect wallet')
      setWalletState(prev => ({ ...prev, isConnecting: false }))
    }
  }

  const disconnectWallet = () => {
    setWalletState({
      isConnected: false,
      address: null,
      balance: null,
      chainId: null,
      chainName: 'Unknown',
      isConnecting: false,
    })
    setIsDropdownOpen(false)
  }

  const copyAddress = () => {
    if (walletState.address) {
      navigator.clipboard.writeText(walletState.address)
      // You could add a toast notification here
    }
  }

  const formatAddress = (address: string) => {
    return `${address.slice(0, 6)}...${address.slice(-4)}`
  }

  if (error) {
    return (
      <div className="flex items-center space-x-2 text-red-600 dark:text-red-400">
        <AlertCircle className="h-4 w-4" />
        <span className="text-sm">{error}</span>
        <button
          onClick={() => setError(null)}
          className="text-xs hover:underline"
        >
          Dismiss
        </button>
      </div>
    )
  }

  if (!walletState.isConnected) {
    return (
      <button
        onClick={connectWallet}
        disabled={walletState.isConnecting}
        className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
      >
        <Wallet className="h-4 w-4 mr-2" />
        {walletState.isConnecting ? 'Connecting...' : 'Connect Wallet'}
      </button>
    )
  }

  return (
    <div className="relative">
      <button
        onClick={() => setIsDropdownOpen(!isDropdownOpen)}
        className="inline-flex items-center px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
      >
        <div className="flex items-center space-x-3">
          <div className="h-2 w-2 bg-green-500 rounded-full"></div>
          <div className="flex flex-col items-start">
            <span className="text-sm font-medium">
              {formatAddress(walletState.address!)}
            </span>
            <span className="text-xs text-gray-500 dark:text-gray-400">
              {walletState.chainName}
            </span>
          </div>
        </div>
        <ChevronDown className="h-4 w-4 ml-2" />
      </button>

      {isDropdownOpen && (
        <div className="absolute right-0 mt-2 w-80 bg-white dark:bg-gray-800 rounded-md shadow-lg border border-gray-200 dark:border-gray-700 z-50">
          <div className="p-4">
            <div className="flex items-center justify-between mb-3">
              <h3 className="text-sm font-semibold text-gray-900 dark:text-white">
                Wallet Details
              </h3>
              <button
                onClick={() => setIsDropdownOpen(false)}
                className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200"
              >
                ×
              </button>
            </div>
            
            <div className="space-y-3">
              <div>
                <label className="block text-xs font-medium text-gray-500 dark:text-gray-400 mb-1">
                  Address
                </label>
                <div className="flex items-center space-x-2">
                  <span className="text-sm text-gray-900 dark:text-white font-mono">
                    {formatAddress(walletState.address!)}
                  </span>
                  <button
                    onClick={copyAddress}
                    className="p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200"
                  >
                    <Copy className="h-3 w-3" />
                  </button>
                  <a
                    href={`https://etherscan.io/address/${walletState.address}`}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200"
                  >
                    <ExternalLink className="h-3 w-3" />
                  </a>
                </div>
              </div>

              <div>
                <label className="block text-xs font-medium text-gray-500 dark:text-gray-400 mb-1">
                  Balance
                </label>
                <span className="text-sm text-gray-900 dark:text-white">
                  {walletState.balance} ETH
                </span>
              </div>

              <div>
                <label className="block text-xs font-medium text-gray-500 dark:text-gray-400 mb-1">
                  Network
                </label>
                <span className="text-sm text-gray-900 dark:text-white">
                  {walletState.chainName} (Chain ID: {walletState.chainId})
                </span>
              </div>
            </div>

            <div className="mt-4 pt-3 border-t border-gray-200 dark:border-gray-600">
              <button
                onClick={disconnectWallet}
                className="w-full px-3 py-2 text-sm text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-md"
              >
                Disconnect Wallet
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

// Extend Window interface for TypeScript
declare global {
  interface Window {
    ethereum?: any
  }
}

export default WalletConnection
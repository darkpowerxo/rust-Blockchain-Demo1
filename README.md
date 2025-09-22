# Blockchain Demo

A comprehensive Rust-based blockchain demo application showcasing professional-level crypto blockchain development skills.

## Features

### Multi-Chain Blockchain Integration
- Support for Ethereum mainnet, Polygon, and Arbitrum
- Chain-specific transaction handling and gas optimization strategies
- RPC connection pooling and retry mechanisms

### Wallet Integration & Key Management
- Support for MetaMask, WalletConnect, and Ledger wallets
- Secure key management with hardware wallet support
- Multi-signature wallet functionality

### DEX Integration & Trading
- Integration with Uniswap V3 and SushiSwap
- Automated market maker (AMM) interaction functions
- Token swap functionality with slippage protection and MEV resistance

### DeFi Protocol Integration
- Connection to Aave and Compound lending protocols
- Flash loan functionality demonstration
- Yield farming strategies and cross-protocol arbitrage detection

### Security Implementation
- Comprehensive input validation and sanitization
- Transaction simulation and security checks
- Reentrancy protection patterns
- Proper error handling and recovery mechanisms

### Real-time Data & Analytics
- WebSocket connections to blockchain nodes
- Real-time price feeds and market data
- Transaction mempool monitoring
- Gas price optimization algorithms

## Quick Start

### Prerequisites

- Rust 1.70 or higher
- PostgreSQL (optional, for data persistence)
- API keys for blockchain RPC providers

### Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd blockchain-demo
```

2. Copy environment configuration:
```bash
cp .env.example .env
```

3. Update the `.env` file with your API keys and configuration.

4. Build the project:
```bash
cargo build --release
```

5. Run the application:
```bash
cargo run
```

The API will be available at `http://localhost:3000` with Swagger UI at `http://localhost:3000/swagger-ui`.

## API Endpoints

### Health Check
- `GET /api/v1/health` - System health and blockchain connectivity status

### Portfolio Management
- `GET /api/v1/portfolio` - Get portfolio overview
- `GET /api/v1/portfolio/{address}` - Get portfolio for specific address

### DEX Integration
- `GET /api/v1/dex/quote` - Get swap quote
- `POST /api/v1/dex/swap` - Execute token swap

### DeFi Integration
- `GET /api/v1/defi/yield` - Get yield opportunities
- `GET /api/v1/defi/lending` - Get lending positions

## Architecture

```
blockchain-demo/
├── src/
│   ├── chains/       # Multi-chain support modules
│   ├── dex/          # DEX integration layer
│   ├── wallets/      # Wallet abstraction
│   ├── contracts/    # Smart contract interactions
│   ├── defi/         # DeFi protocol integrations
│   ├── security/     # Security utilities
│   ├── analytics/    # Market data and analysis
│   └── api/          # REST API endpoints
├── contracts/        # Sample smart contracts
├── tests/            # Comprehensive test suite
└── docs/             # Documentation and examples
```

## Development

### Running Tests
```bash
cargo test
```

### Running with Debug Logs
```bash
RUST_LOG=debug cargo run
```

### Building Documentation
```bash
cargo doc --open
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## License

MIT License - see LICENSE file for details.

## Security

This is a demo application. Do not use in production without proper security auditing and testing.

## Disclaimer

This software is provided for educational and demonstration purposes only. Use at your own risk.

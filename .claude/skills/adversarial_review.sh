#!/usr/bin/env bash
set -euo pipefail

COMMAND="${1:-help}"
SCOPE="${2:-all}"

case "$COMMAND" in
  review|crypto_review|transport_review|routing_review|privacy_review|full_review)
    # Determine scope from command
    case "$COMMAND" in
      crypto_review) SCOPE="crypto" ;;
      transport_review) SCOPE="transport" ;;
      routing_review) SCOPE="routing" ;;
      privacy_review) SCOPE="privacy" ;;
      full_review) SCOPE="all" ;;
    esac

    echo "=== Adversarial Review: ${SCOPE} ==="
    echo "Model: deepseek-v3.2:cloud (primary) / deepseek-v4-pro:cloud (fallback)"
    echo "Scope: ${SCOPE}"
    echo ""

    # List files in scope
    case "$SCOPE" in
      crypto)
        echo "Files under review:"
        find core/src/crypto -name "*.rs" 2>/dev/null | sort
        ;;
      transport)
        echo "Files under review:"
        find core/src/transport -name "*.rs" 2>/dev/null | sort
        ;;
      routing)
        echo "Files under review:"
        find core/src/routing -name "*.rs" 2>/dev/null | sort
        ;;
      privacy)
        echo "Files under review:"
        find core/src/privacy -name "*.rs" 2>/dev/null | sort
        ;;
      all)
        echo "Files under review:"
        find core/src/crypto core/src/transport core/src/routing core/src/privacy -name "*.rs" 2>/dev/null | sort
        ;;
    esac

    echo ""
    echo "Review protocol:"
    echo "1. Race condition analysis on all Arc<RwLock<...>> boundaries"
    echo "2. Null/edge-case analysis on unwrap(), expect(), indexed access"
    echo "3. Crypto/protocol analysis for constant-time ops, info leakage"
    echo "4. Resource exhaustion check for unbounded allocations"
    echo "5. Supply chain check for unexpected dependency changes"
    echo ""
    echo "Output format: Severity / Category / Location / Description / Proof / Fix"
    echo ""
    echo "NOTE: This skill defines the review protocol. The actual review"
    echo "should be launched via the coordinator agent with the precision-validator"
    echo "or deep-analyst role using the adversarial_reviewer prompt template."
    ;;
  help|*)
    echo "Usage: adversarial_review.sh <command> [scope]"
    echo ""
    echo "Commands:"
    echo "  review <scope>     Launch review of specified scope (crypto|transport|routing|privacy|all)"
    echo "  crypto_review     Shortcut for crypto scope"
    echo "  transport_review  Shortcut for transport scope"
    echo "  routing_review    Shortcut for routing scope"
    echo "  privacy_review    Shortcut for privacy scope"
    echo "  full_review       Review all security-critical modules"
    echo "  help              Show this help"
    ;;
esac

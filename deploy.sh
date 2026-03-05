#!/bin/bash
# --- QuantumFortress Unified Deployment ---

GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}🛡️ Initiating QuantumFortress Sentinel Deployment...${NC}"

# 1. Verification of Secrets
if [ ! -f .env ]; then
    echo -e "${RED}⚠️ Critical Error: .env missing. Creating deployment default...${NC}"
    cat <<EOF > .env
SERVER_PORT=3000
MIN_ENTROPY=4.5
RUST_LOG=info
RPC_URL=your_rpc_here
PRIVATE_KEY=your_key_here
CONTRACT_ADDRESS=your_address_here
EOF
fi

# 2. Hardened Build Process
echo -e "${BLUE}🏗️ Building Distroless Production Image...${NC}"
docker build -t quantum-fortress . || { echo -e "${RED}❌ Build Failed${NC}"; exit 1; }

# 3. Instance Refresh
docker stop quantum-fortress-instance 2>/dev/null || true
docker rm quantum-fortress-instance 2>/dev/null || true

# 4. Secure Launch (Resource Constraints & Sandbox)
echo -e "${BLUE}🚀 Deploying Sentinel Container...${NC}"
docker run -d \
    --name quantum-fortress-instance \
    -p 3000:3000 \
    --env-file .env \
    --memory="512m" \
    --cpus="0.5" \
    --security-opt no-new-privileges:true \
    --restart unless-stopped \
    quantum-fortress

# 5. Final Health Verification
sleep 3
if curl -s http://localhost:3000/api/stats | grep -q "Operational"; then
    echo -e "${GREEN}✨ SYSTEM SECURED: QuantumFortress is now guarding your infrastructure.${NC}"
else
    echo -e "${RED}⚠️ Warning: Sentinel is initializing or failed. Check logs.${NC}"
fi

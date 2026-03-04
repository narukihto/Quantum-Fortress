#!/bin/bash

# --- QuantumFortress Deployment Sentinel ---
# This script automates the build, test, and deployment process.

# 1. Colors for professional output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}🛡️ Starting QuantumFortress Deployment Sequence...${NC}"

# 2. Environment Check
if [ ! -f .env ]; then
    echo -e "${RED}⚠️ Error: .env file not found! Generating default...${NC}"
    echo "SERVER_PORT=3000" > .env
    echo "ENTROPY_THRESHOLD=4.0" >> .env
    echo "RUST_LOG=info" >> .env
fi

# 3. Clean and Build Docker Image
echo -e "${BLUE}🏗️ Building Secure Distroless Container...${NC}"
docker build -t quantum-fortress .

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Docker Build Successful.${NC}"
else
    echo -e "${RED}❌ Docker Build Failed! Check the logs above.${NC}"
    exit 1
fi

# 4. Stop existing instances to prevent port conflicts
echo -e "${BLUE}🔄 Refreshing Sentinel Instances...${NC}"
docker stop quantum-fortress-instance 2>/dev/null || true
docker rm quantum-fortress-instance 2>/dev/null || true

# 5. Launch the Fortress
echo -e "${BLUE}🚀 Launching Quantum-Ready Gateway on Port 3000...${NC}"
docker run -d \
    --name quantum-fortress-instance \
    -p 3000:3000 \
    --env-file .env \
    --restart unless-stopped \
    quantum-fortress

# 6. Final Health Check
echo -e "${BLUE}🔍 Running Health Check...${NC}"
sleep 2 # Wait for the service to initialize
if curl -s http://localhost:3000/api/stats > /dev/null; then
    echo -e "${GREEN}✨ DEPLOYMENT COMPLETE!${NC}"
    echo -e "${GREEN}Dashboard & API are LIVE at http://localhost:3000${NC}"
else
    echo -e "${RED}⚠️ Deployment Warning: Server is not responding yet. Check 'docker logs quantum-fortress-instance'${NC}"
fi

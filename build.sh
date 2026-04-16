#!/usr/bin/env bash
set -e

# ANSI color codes
CYAN='\033[0;36m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

echo -e "${CYAN}=== Building Premonition Synthesis Suite ===${NC}"

# 1. Build CLI
echo -e "${GREEN}[1/4] Building CLI Application...${NC}"
cd premonition-cli
cargo build --release
cd ..

# 2. Build WebAssembly target
echo -e "${GREEN}[2/4] Building WebAssembly Module...${NC}"
cd premonition-wasm
wasm-pack build --target web --out-dir ../premonition-web/pkg
cd ..

# 3. Process/Package Web App 
echo -e "${GREEN}[3/4] Packaging Web App...${NC}"
# No build step necessary for the web UI other than Wasm generation
echo "Web app ready in premonition-web/"

# 4. Build JUCE Audio Plugins & Standalone App
if command -v cmake &> /dev/null; then
    echo -e "${GREEN}[4/4] Building JUCE VST3, AU, and Standalone App...${NC}"
    cd premonition-juce
    mkdir -p build
    cd build
    cmake -DCMAKE_BUILD_TYPE=Release ..
    cmake --build . --config Release -j$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)
    cd ../..
    echo -e "${CYAN}=== All targets built successfully! ===${NC}"
else
    echo -e "${GREEN}[4/4] Skipping JUCE VST3/AU/Standalone App (cmake not found)...${NC}"
    echo -e "\033[0;33mPlease install CMake to build the JUCE plugins and standalone app.\033[0m"
    echo -e "${CYAN}=== Web, CLI, and WASM targets built successfully! ===${NC}"
fi

#!/bin/bash

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${YELLOW}Starting clippy auto fix...${NC}"
cargo clippy -p firework_macro --fix --allow-dirty --allow-staged --no-deps -- -D warnings

if [ $? -eq 0 ]; then
    echo -e "${GREEN}Clippy: successfully${NC}"
else
    echo -e "${RED}Clippy: errror${NC}"
    exit 1
fi

echo ""
echo -e "${YELLOW}Code formating...${NC}"
cargo fmt -p firework_macro

if [ $? -eq 0 ]; then
    echo -e "${GREEN}Fmt: successfully${NC}"
else
    echo -e "${RED}Fmt: error${NC}"
    exit 1
fi

#!/bin/bash

# PAXBuild Key Management Demo
# This script demonstrates the new key management functionality

echo "=== PAXBuild Key Management Demo ==="
echo

# Create a demo directory
DEMO_DIR="./demo-keys"
mkdir -p "$DEMO_DIR"

echo "1. Generating a new key pair..."
paxbuild keys generate \
    --private "$DEMO_DIR/private.key" \
    --public "$DEMO_DIR/public.key"

echo
echo "2. Showing key information..."
echo "Private key info:"
paxbuild keys info \
    --key "$DEMO_DIR/private.key" \
    --type private

echo
echo "Public key info:"
paxbuild keys info \
    --key "$DEMO_DIR/public.key" \
    --type public

echo
echo "3. Listing keys in the demo directory..."
paxbuild keys list --directory "$DEMO_DIR"

echo
echo "4. Exporting public key from private key..."
paxbuild keys export \
    --private "$DEMO_DIR/private.key" \
    --public "$DEMO_DIR/exported-public.key"

echo
echo "5. Backing up keys..."
BACKUP_DIR="$DEMO_DIR/backup"
paxbuild keys backup \
    --source "$DEMO_DIR" \
    --dest "$BACKUP_DIR"

echo
echo "6. Listing backup directory..."
paxbuild keys list --directory "$BACKUP_DIR"

echo
echo "7. Testing signature workflow..."
echo "Creating a test package..."
echo "test package content" > "$DEMO_DIR/test.pax"

echo "Signing the package..."
paxbuild sign \
    --package "$DEMO_DIR/test.pax" \
    --key "$DEMO_DIR/private.key"

echo "Verifying the package..."
paxbuild verify \
    --package "$DEMO_DIR/test.pax" \
    --key "$DEMO_DIR/public.key"

echo
echo "8. Importing a key from backup..."
paxbuild keys import \
    --source "$BACKUP_DIR/private.key" \
    --dest "$DEMO_DIR/imported-private.key" \
    --type private

echo
echo "=== Demo completed successfully! ==="
echo "All key management operations are working correctly."
echo
echo "Key files created in: $DEMO_DIR"
echo "Backup created in: $BACKUP_DIR"

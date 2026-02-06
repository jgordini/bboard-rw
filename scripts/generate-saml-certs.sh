#!/bin/bash
# =============================================================================
# Legacy Certificate Generation Helper
# =============================================================================
# Generates self-signed key/cert files under /var/bboard-rw/ssl.
# NOTE: bboard-rw does not use SAML directly; this is only useful if you
# terminate TLS with a reverse proxy and want local test certificates.
#
# Usage:
#   ./generate-saml-certs.sh <domain>
#
# Example:
#   ./generate-saml-certs.sh 138.26.48.197
#
# =============================================================================

set -e

DOMAIN="${1}"

if [ -z "$DOMAIN" ]; then
    echo "ERROR: Domain name required"
    echo "Usage: $0 <domain>"
    echo "Example: $0 138.26.48.197"
    exit 1
fi

SSL_DIR="/var/bboard-rw/ssl"

echo "========================================================================"
echo "Generating self-signed certificates for $DOMAIN"
echo "========================================================================"

# Create SSL directory if it doesn't exist
mkdir -p "$SSL_DIR"
cd "$SSL_DIR"

# Check if certificates already exist
if [ -f "sp.key" ] && [ -f "sp.crt" ]; then
    echo ""
    read -r -p "Certificates already exist. Overwrite? (y/N): " confirm
    if [ "$confirm" != "y" ] && [ "$confirm" != "Y" ]; then
        echo "Cancelled."
        exit 0
    fi
    echo "Backing up existing certificates..."
    mv sp.key "sp.key.backup.$(date +%Y%m%d_%H%M%S)" 2>/dev/null || true
    mv sp.crt "sp.crt.backup.$(date +%Y%m%d_%H%M%S)" 2>/dev/null || true
fi

# Generate private key
echo ""
echo "[1/3] Generating private key..."
openssl genrsa -out sp.key 2048

# Generate certificate signing request
echo ""
echo "[2/3] Generating certificate signing request..."
openssl req -new -key sp.key -out sp.csr \
    -subj "/C=US/ST=Alabama/L=Birmingham/O=University of Alabama at Birmingham/CN=$DOMAIN"

# Generate self-signed certificate (valid for 10 years)
echo ""
echo "[3/3] Generating self-signed certificate (valid 10 years)..."
openssl x509 -req -days 3650 -in sp.csr -signkey sp.key -out sp.crt

# Secure the private key
chmod 600 sp.key
chmod 644 sp.crt

# Clean up CSR
rm sp.csr

echo ""
echo "========================================================================"
echo "Certificates Generated Successfully!"
echo "========================================================================"
echo ""
echo "Files created in $SSL_DIR:"
echo "  - sp.key (private key) - permissions: 600"
echo "  - sp.crt (certificate) - permissions: 644"
echo ""
echo "Certificate Details:"
openssl x509 -in sp.crt -noout -subject -dates
echo ""
echo "Next Steps:"
echo "1. Configure your reverse proxy to use $SSL_DIR/sp.key and $SSL_DIR/sp.crt"
echo "2. Expose HTTPS (443) in cloud.rc security groups and firewall exception"
echo "3. Redeploy your proxy stack"
echo ""
echo "To view certificate:"
echo "  openssl x509 -in $SSL_DIR/sp.crt -text -noout"
echo ""
echo "To view public key:"
echo "  cat $SSL_DIR/sp.crt"
echo ""

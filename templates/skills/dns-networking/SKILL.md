---
name: dns-networking
description: DNS resolution, IP addressing, subnetting, network protocols, and diagnostic tools. Use when configuring DNS records, debugging connectivity, setting up networking, or troubleshooting network issues.
allowed-tools: Bash(dig:*) Bash(nslookup:*) Bash(traceroute:*) Bash(curl:*) Bash(ss:*) Read Write
---

# DNS & Networking

## When to Use

- Setting up or modifying DNS records for a domain
- Debugging DNS resolution failures or propagation delays
- Troubleshooting network connectivity between services
- Configuring firewall rules or port forwarding
- Analyzing HTTP/HTTPS connection issues or TLS errors
- Calculating subnets or planning IP address allocation
- Diagnosing latency, packet loss, or routing problems
- Setting up load balancing or reverse proxies

## Instructions

### DNS Fundamentals

DNS maps domain names to IP addresses through a hierarchical resolution chain: stub resolver, recursive resolver, root servers, TLD servers, authoritative servers.

**Record types and their purpose:**
- **A / AAAA** -- IPv4 / IPv6 address mapping
- **CNAME** -- Alias to another domain (cannot coexist with other records at zone apex)
- **MX** -- Mail server with priority (lower number = higher priority)
- **TXT** -- Arbitrary text; used for SPF, DKIM, domain verification
- **SRV** -- Service discovery with priority, weight, port, target
- **NS** -- Delegates a zone to nameservers
- **SOA** -- Zone metadata: primary NS, admin email, serial, refresh/retry/expire timers
- **PTR** -- Reverse DNS (IP to hostname)

**TTL:** Time-to-live in seconds. Lower TTL before migrations (300s), raise after (3600-86400s). Check current TTL before changes: `dig +noall +answer example.com A`.

### IP Addressing & Subnetting

**CIDR notation:** `192.168.1.0/24` means 24-bit network prefix, 8 bits for hosts (254 usable).

**Private ranges (RFC 1918):**
- `10.0.0.0/8` -- 16M addresses, used in large networks and cloud VPCs
- `172.16.0.0/12` -- 1M addresses, common in Docker default bridges
- `192.168.0.0/16` -- 65K addresses, typical home/office networks

**Quick subnet math:** /24 = 256 IPs, /25 = 128, /26 = 64, /27 = 32, /28 = 16. Subtract 2 for usable hosts (network + broadcast).

### Common Protocols

**TCP vs UDP:** TCP provides reliable ordered delivery with connection setup (3-way handshake). UDP is connectionless, lower overhead, used where speed matters more than guaranteed delivery.

- **HTTP/HTTPS** -- TCP port 80/443. HTTPS adds TLS encryption.
- **DNS** -- UDP port 53 for queries, TCP port 53 for zone transfers and large responses.
- **SSH** -- TCP port 22. Key-based auth preferred over passwords.
- **SMTP/IMAP** -- TCP 25/587 (submission) and 143/993 (IMAP/IMAPS).

### TLS and HTTPS

TLS handshake: ClientHello (supported ciphers) -> ServerHello (chosen cipher + certificate) -> key exchange -> encrypted session.

**Common issues:** expired certificates, hostname mismatch, missing intermediate CA, TLS version mismatch. Debug with `curl -vI https://example.com` or `openssl s_client -connect example.com:443`.

### Port Management

List listening ports: `ss -tlnp` (TCP) or `ss -ulnp` (UDP). Check if a specific port is in use: `ss -tlnp | grep :8080`. Find which process owns a port: `ss -tlnp sport = :443`.

Well-known ports: 0-1023 (require root). Registered: 1024-49151. Ephemeral: 49152-65535.

### Firewall Basics

**ufw (Ubuntu/Debian):**
```
ufw allow 80/tcp
ufw allow from 10.0.0.0/8 to any port 5432
ufw status numbered
```

**iptables (direct):**
```
iptables -A INPUT -p tcp --dport 443 -j ACCEPT
iptables -A INPUT -s 192.168.1.0/24 -j ACCEPT
iptables -L -n --line-numbers
```

Always allow SSH (port 22) before enabling a firewall. Test rules before persisting.

### Load Balancing Concepts

- **Round-robin DNS** -- Multiple A records; no health checking, simple distribution
- **Reverse proxy** (nginx/HAProxy) -- Layer 7, can route by path/header, health checks
- **Layer 4 LB** -- TCP-level forwarding, lower overhead, no content inspection

Health checks should verify application readiness, not just TCP connectivity.

### Diagnostic Workflow

When troubleshooting connectivity, work bottom-up:

1. **DNS resolution** -- `dig example.com` -- does the name resolve?
2. **Reachability** -- `ping -c 3 <ip>` -- is the host reachable?
3. **Route** -- `traceroute <ip>` -- where does the path break?
4. **Port** -- `curl -v telnet://<ip>:<port>` or `ss -tlnp` -- is the port open?
5. **Application** -- `curl -vI https://example.com` -- does the service respond?

See `references/troubleshooting-tools.md` for detailed command examples.

## Examples

### Scenario 1: DNS record not propagating after change

Check current authoritative answer vs cached answer, compare TTLs, and verify the change reached the authoritative server:
```
dig @8.8.8.8 example.com A          # Public resolver (may be cached)
dig @ns1.provider.com example.com A  # Authoritative (should show new value)
dig +trace example.com A             # Full resolution chain
```
If authoritative shows the new record but public resolvers do not, wait for the old TTL to expire.

### Scenario 2: Service unreachable on specific port

Isolate whether the problem is DNS, network, or application:
```
dig api.example.com                   # Verify resolution
curl -v telnet://api.example.com:443  # Test TCP connectivity
curl -vI https://api.example.com      # Test full HTTP/TLS stack
ss -tlnp sport = :443                 # On the server: is anything listening?
```

### Scenario 3: Setting up DNS for a new subdomain

Create an A record or CNAME, verify propagation, then set up TLS:
```
# After adding the DNS record at your provider:
dig +short staging.example.com         # Verify resolution
curl -I http://staging.example.com     # Test HTTP before TLS
# Set up TLS (e.g., certbot):
certbot certonly --nginx -d staging.example.com
curl -vI https://staging.example.com   # Verify HTTPS works
```

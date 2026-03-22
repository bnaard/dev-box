---
name: threat-modeling
description: Threat modeling using STRIDE methodology. Data flow diagrams, trust boundaries, attack surface mapping, and risk assessment. Use when analyzing system security, designing secure architectures, or conducting security reviews.
---

# Threat Modeling

Analyze systems for security threats using the STRIDE methodology. Produce data flow
diagrams, identify trust boundaries, map attack surfaces, and assess risk.

## When to Use

- Designing a new system or feature that handles sensitive data.
- Reviewing architecture for security before implementation.
- Conducting a security review or threat assessment.
- Identifying trust boundaries and attack surfaces.
- Prioritizing security work by risk (likelihood x impact).

## Instructions

### STRIDE Methodology

Analyze each component and data flow for six threat categories:

| Category                  | Threat                          | Property Violated   |
|---------------------------|---------------------------------|---------------------|
| **S**poofing              | Pretending to be someone else   | Authentication      |
| **T**ampering             | Modifying data or code          | Integrity           |
| **R**epudiation           | Denying an action occurred      | Non-repudiation     |
| **I**nformation Disclosure| Exposing data to unauthorized   | Confidentiality     |
| **D**enial of Service     | Making the system unavailable   | Availability        |
| **E**levation of Privilege| Gaining unauthorized access     | Authorization       |

### Step 1: Data Flow Diagram

Map the system showing processes, data stores, data flows, and external entities:

```
                    Trust Boundary
                    +---------------------------------+
  [User Browser] ---|---> [Web Server] ---> [App Server] ---> [Database]
                    |         |                  |
                    |         v                  v
                    |    [Static Files]    [Cache (Redis)]
                    +---------------------------------+
                           |
                           v
                    [External Auth Provider]
```

Label each element:
- **Processes** (circles/boxes): Web server, app server, background workers.
- **Data stores** (parallel lines): Database, cache, file system, message queue.
- **Data flows** (arrows): HTTP requests, database queries, API calls.
- **External entities** (rectangles): Users, third-party services, admin.
- **Trust boundaries** (dashed lines): Network segments, process boundaries.

### Step 2: Identify Trust Boundaries

Trust boundaries exist where data crosses between different trust levels:

- Internet to DMZ (user requests entering the network).
- DMZ to internal network (web server to application server).
- Application to database (app server to data store).
- Service to service (between microservices).
- User privilege levels (regular user vs admin).

Every trust boundary crossing is a potential attack point.

### Step 3: Apply STRIDE to Each Element

For each process, data store, and data flow, ask:

- **Spoofing:** Can an attacker impersonate a legitimate entity?
- **Tampering:** Can data be modified in transit or at rest?
- **Repudiation:** Can a user deny performing an action?
- **Info Disclosure:** Can sensitive data leak?
- **DoS:** Can this component be overwhelmed or crashed?
- **Elevation:** Can a user gain higher privileges?

### Step 4: Risk Assessment

Rate each threat using likelihood and impact:

| Rating | Likelihood                        | Impact                           |
|--------|-----------------------------------|----------------------------------|
| High   | Exploitable with public tools     | Data breach, full compromise     |
| Medium | Requires some skill or access     | Partial data exposure, downtime  |
| Low    | Requires insider access or luck   | Minor data exposure, degradation |

**Risk = Likelihood x Impact**

| Likelihood \ Impact | High     | Medium   | Low      |
|---------------------|----------|----------|----------|
| High                | Critical | High     | Medium   |
| Medium              | High     | Medium   | Low      |
| Low                 | Medium   | Low      | Low      |

### Step 5: Mitigation Strategies

For each identified threat, document a mitigation:

| STRIDE Category       | Common Mitigations                                    |
|-----------------------|-------------------------------------------------------|
| Spoofing              | MFA, certificate pinning, mutual TLS                  |
| Tampering             | Digital signatures, checksums, immutable audit logs    |
| Repudiation           | Audit logging, tamper-evident logs, digital signatures |
| Information Disclosure| Encryption (TLS, AES), access controls, data masking  |
| Denial of Service     | Rate limiting, auto-scaling, CDN, circuit breakers     |
| Elevation of Privilege| Least privilege, input validation, sandboxing          |

### Attack Surface Mapping

Enumerate all entry points an attacker could use:

- **Network:** Open ports, exposed services, public endpoints.
- **Application:** API endpoints, file upload, search, user input fields.
- **Authentication:** Login, password reset, session management.
- **Data:** Database access, file storage, backups, logs.
- **Infrastructure:** Cloud console, CI/CD pipeline, container registry.
- **Human:** Phishing targets, social engineering, insider threats.

### Output Format

Structure threat model output as:

```
## Threat Model: [System Name]

### System Description
Brief description of the system and its purpose.

### Data Flow Diagram
ASCII diagram showing components and trust boundaries.

### Assets
List of valuable data and resources to protect.

### Threats
| ID | STRIDE | Component | Threat | Risk | Mitigation |
|----|--------|-----------|--------|------|------------|
| T1 | S      | Login API | Credential stuffing | High | Rate limiting, MFA |
| T2 | I      | Database  | SQL injection leaks PII | Critical | Parameterized queries |

### Recommendations
Prioritized list of security improvements.
```

## Examples

### Example 1: Threat model a web application

```
User: Create a threat model for our e-commerce application.

Agent: Reads the architecture and codebase, then produces:
  1. Data flow diagram showing: browser -> CDN -> load balancer -> API servers
     -> database / cache / payment gateway. Trust boundaries at each hop.
  2. STRIDE analysis of each component:
     - Spoofing: session hijacking on API server (Medium risk).
     - Tampering: price manipulation in cart requests (High risk).
     - Info Disclosure: PII exposure in error messages (Medium risk).
     - DoS: no rate limiting on search endpoint (High risk).
  3. Risk-prioritized mitigation recommendations.
```

### Example 2: Assess a microservices architecture

```
User: Review our microservices architecture for security threats.

Agent: Maps service communication patterns:
  - Identifies 12 services with 28 inter-service data flows.
  - Trust boundaries: public internet, API gateway, service mesh, database tier.
  - Key findings:
    - 3 services communicate over unencrypted HTTP internally (Tampering, Info Disclosure).
    - No mutual TLS between services (Spoofing).
    - Shared database credentials across services (Elevation of Privilege).
    - No circuit breakers — cascade failure risk (DoS).
  Recommends: service mesh with mTLS, per-service DB credentials, circuit breakers.
```

### Example 3: Pre-launch security review

```
User: We're launching next week. Do a threat model focused on the highest risks.

Agent: Focuses on critical-path components:
  - Payment flow: identifies missing webhook signature verification (Tampering, Critical).
  - User registration: no rate limiting (DoS, High), email enumeration (Info Disclosure).
  - Admin panel: accessible without VPN (Elevation of Privilege, Critical).
  - File uploads: no type validation (Injection via uploaded HTML, High).
  Produces a prioritized punch list:
    1. [Critical] Add webhook signature verification before processing payments.
    2. [Critical] Restrict admin panel to VPN or IP allowlist.
    3. [High] Add rate limiting to registration and login.
    4. [High] Validate file upload types and serve from separate domain.
```

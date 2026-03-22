---
name: grpc-protobuf
description: Protocol Buffers schema design and gRPC service patterns including streaming, error handling, and backward compatibility. Use when designing gRPC services, writing .proto files, or implementing gRPC clients/servers.
allowed-tools: Bash(protoc:*) Bash(grpcurl:*) Read Write
---

# gRPC and Protocol Buffers

Design Protocol Buffer schemas and implement gRPC services — message design,
service patterns, streaming, error handling, and backward compatibility.

## When to Use

- Designing `.proto` files for new or existing services.
- Implementing gRPC services (unary, streaming) in any language.
- Choosing between gRPC communication patterns.
- Handling errors with gRPC status codes and error details.
- Ensuring backward compatibility when evolving schemas.
- Adding interceptors or middleware to gRPC services.

## Instructions

### Protocol Buffers Schema Design

See `references/proto-conventions.md` for naming conventions, field numbering, and
compatibility rules.

```protobuf
syntax = "proto3";
package myapp.orders.v1;
import "google/protobuf/timestamp.proto";

message Order {
  string order_id = 1;
  string customer_id = 2;
  repeated LineItem items = 3;
  OrderStatus status = 4;
  google.protobuf.Timestamp created_at = 5;
}

enum OrderStatus {
  ORDER_STATUS_UNSPECIFIED = 0;  // zero value = "not set"
  ORDER_STATUS_PENDING = 1;
  ORDER_STATUS_SHIPPED = 2;
}
```

- Always `syntax = "proto3"`. Package: `<org>.<service>.<version>`.
- Enum zero value must be `UNSPECIFIED`. Use `oneof` for mutually exclusive fields.
- Prefer `google.protobuf.Timestamp` over custom time representations.

### gRPC Service Patterns

| Pattern              | Use case                                    |
|----------------------|---------------------------------------------|
| Unary                | Standard request-response (CRUD, queries)   |
| Server streaming     | Real-time feeds, long-polling replacement    |
| Client streaming     | Bulk uploads, client-side event aggregation  |
| Bidirectional        | Chat, collaborative editing, real-time sync  |

```protobuf
service OrderService {
  rpc CreateOrder(CreateOrderRequest) returns (CreateOrderResponse);
  rpc WatchStatus(WatchStatusRequest) returns (stream StatusEvent);
  rpc UploadItems(stream LineItem) returns (UploadItemsResponse);
  rpc Chat(stream ChatMessage) returns (stream ChatMessage);
}
```

### Request/Response Wrappers

Wrap every RPC in dedicated Request/Response messages — never reuse domain types
directly. This allows adding fields without breaking the domain model.

### Error Handling

| Code              | When to use                                |
|-------------------|--------------------------------------------|
| INVALID_ARGUMENT  | Bad client input (validation failure)      |
| NOT_FOUND         | Resource does not exist                    |
| ALREADY_EXISTS    | Duplicate key conflict                     |
| PERMISSION_DENIED | Authenticated but not authorized           |
| UNAUTHENTICATED   | Missing or invalid credentials             |
| UNAVAILABLE       | Transient failure — client should retry    |
| DEADLINE_EXCEEDED | Operation took too long                    |

For rich error details, use `google.rpc.Status` with `BadRequest` or `ErrorInfo` packed
into the `details` field. Never return OK with an error payload in the response body.

### Backward Compatibility

**Safe:** add new fields/messages/RPCs/enum values.
**Breaking:** change field type or number, reuse deleted field numbers, change enum values.

When removing a field, mark it `reserved` to prevent accidental reuse:
```protobuf
message Order {
  reserved 6, 7;
  reserved "legacy_status";
}
```

### Interceptors

Use interceptors for cross-cutting concerns, chained in order:
auth -> logging -> metrics -> validation -> handler.

- **Auth**: validate tokens, set user context.
- **Logging**: log RPC method, duration, status code.
- **Metrics**: latency histograms and error rates per method.
- **Validation**: use `buf validate` for auto-generated validation from proto annotations.

### Tooling

Prefer **buf** over raw `protoc` — it handles linting, breaking change detection, and
code generation in one tool. Use **grpcurl** for command-line service invocation.

## Examples

### Example 1: Design a gRPC notification service

```
User: Design a gRPC service for sending and tracking notifications.

Agent: Creates notification/v1/notification.proto with messages
  (Notification, SendRequest/Response), enums (Channel, DeliveryStatus),
  and RPCs: SendNotification (unary), WatchDeliveryStatus (server stream),
  UpdatePreferences (unary). Configures buf.yaml for linting.
```

### Example 2: Fix backward compatibility issue

```
User: Change the price field from int32 to int64 — clients already use it.

Agent: Cannot change field type (breaking). Instead: adds price_cents (int64)
  with a new field number, deprecates old price field, server populates both
  during migration. Runs buf breaking to confirm no violations.
```

### Example 3: Add client-side retry with interceptors

```
User: gRPC calls to inventory service fail intermittently under load.

Agent: Implements retry interceptor: retries on UNAVAILABLE and
  DEADLINE_EXCEEDED only, exponential backoff (100ms/200ms/400ms, max 3),
  propagates original deadline. Adds per-method timeout configuration.
```

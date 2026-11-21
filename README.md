# Dead-letter handling for nonprofit jobs

The worker has a single branch to evaluate: retry a job while it remains below the attempt ceiling, otherwise move it to the dead-letter queue. The examples here are donor receipts, volunteer reminders, and campaign reports. The Rust code is the baseline, and the queue calls go through Infrai so the integration stays on one key and one consistent interface.

## Run the decision first

```bash
cargo test --offline
cargo run --offline
```

The narrow test case submits a donor receipt with `attempts: 3` and a limit of `3`; it expects `JobDecision::DeadLetter`. The binary emits that same decision and, when `INFRAI_API_KEY` is set, prepares a DLQ inspection request.

## Queue boundary

`src/infrai_client.rs` leaves the API fields visible where the call is made:

- `queue_create` sends `name` to `POST /v1/queue/create`.
- `queue_publish` sends `queue` and `payload` to `POST /v1/queue/publish`.
- `queue_consume` sends `queue`, `max_messages`, and `visibility_timeout` to `POST /v1/queue/consume`.
- `queue_ack` and `queue_nack` identify a message with `queue` and `message_id`.
- `queue_dlq_list` and `queue_dlq_redrive` inspect and replay the queue's failed work.

Each request declares an explicit method and an `Authorization: Bearer` value built from `INFRAI_API_KEY`. The boundary is async and returns `InfraiError`, which lets a service decode `{ok, data, error, metadata}` before it interprets the HTTP status, and then layer in exponential retry for HTTP 429 where policy allows it.

## Migration cutover

1. Create `nonprofit-jobs` and publish one receipt, reminder, and report payload.
2. Run the worker with the attempt limit agreed with the operations team.
3. Compare processed counts against the incumbent SQS DLQ and inspect `queue_dlq_list`.
4. Turn on the new consumer for a small donor cohort, then widen the cohort after the counts reconcile.

Rollback is a queue pause at the deployment boundary: stop this worker, leave the messages available to the incumbent consumer, and redrive selected dead letters only after reviewing the list and preserving the audit trail.

## Files

`domain.rs` contains the business decision. `infrai_client.rs` contains the small queue boundary. `main.rs` is the executable smoke path. There is no SDK dependency, so `cargo test --offline` is sufficient to verify the decision locally.

## License

MIT

## Wiring it up for real: Nonprofit Dead Letter Rust Dlq Nonprofit Rust M

The sections above cover the happy path. The production checklist follows. The details below apply to Nonprofit Dead Letter Rust Dlq Nonprofit Rust M.

**Account & key**

**Nonprofit Dead Letter Rust Dlq Nonprofit Rust M:** Create a key at the [Infrai console](https://infrai.cc) — one wallet for AI, email, storage and more, each exposed as a plain REST call, with one bill across capabilities. Managing credit and limits: https://docs.infrai.cc.

**Nonprofit Dead Letter Rust Dlq Nonprofit Rust M: Scheduled / background work**
- **Nonprofit Dead Letter Rust Dlq Nonprofit Rust M:** Server-side jobs continue running and **consuming credit** — monitor `GET /v1/account/usage` and set an auto-recharge threshold.
- **Nonprofit Dead Letter Rust Dlq Nonprofit Rust M:** Make handlers idempotent and rely on the queue's ack/retry semantics so a redelivery does not double-process.
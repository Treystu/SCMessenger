# AWS free-tier IAM setup for the SCMessenger cloud relay/farm-sim rig

Status: Active. Supports FARM_FINAL_PLAN.md WS-FARM-B (B4: cloud relays,
B5/B6: rig-based farm-topology simulation). Free tier only, $0 budget target.

## Why this file exists

`iam-policy-scmessenger-relay.json` is a scoped IAM policy so the orchestrator
(or a delegated agent) can provision/manage the relay EC2 instance(s) without
ever holding root credentials or broad account access. It is deliberately
restrictive:

- **Instance type locked to `t2.micro`/`t3.micro` only** (free-tier eligible) -
  `ec2:RunInstances` is denied for any other type via a `Condition`.
- **EBS volume capped at 30 GB, gp2/gp3 only** - matches the free-tier storage
  allowance.
- **Region locked to `us-east-1`** - keeps the free-tier hour pool
  unambiguous; change the `aws:RequestedRegion` values in the policy if you
  want a different home region, but pick ONE region and stick to it.
- **No Elastic IP allocation** - avoids the (small but real) charge for an EIP
  not attached to a running instance; the DDNS approach in the farm plan
  doesn't need one anyway.
- **Explicit deny list** for anything that costs money outside EC2/EBS basics
  or that expands blast radius: `iam:*`, `s3:*`, `rds:*`, spot/reserved
  instance purchasing, `ModifyInstanceAttribute` (blocks instance-type
  resizing past the free tier), etc. Deny statements win over allow
  statements in IAM evaluation, so this holds even if a later edit
  accidentally widens an Allow.
- **No self-service IAM** - this identity cannot create/modify IAM users,
  roles, or policies (including its own) - a compromised or buggy delegated
  agent can't escalate its own privileges.
- **Budgets/CloudWatch alarm permissions included** - so the same identity can
  also set up its own spend alarm (see below) without needing a separate
  higher-privilege pass.

This policy does NOT by itself guarantee zero spend - IAM can't prevent
free-tier hour/storage exhaustion if you leave instances running past the
750 hrs/month or 30 GB allowance. Pair it with an AWS Budget (step 4 below).

## Setup steps (operator-only, ~10 minutes)

1. **Do not use your AWS root account for this.** Log into the AWS Console as
   root only to create the IAM user in step 2, then use the IAM user for
   everything else, always.

2. **Create a dedicated IAM user** (IAM console -> Users -> Create user):
   - Name: `scmessenger-relay-orchestrator` (or similar).
   - Access type: Programmatic access (generates an access key + secret).
   - Do NOT enable console/password access for this user - API-only.

3. **Attach the policy:**
   - IAM console -> Policies -> Create policy -> JSON tab -> paste the
     contents of `iam-policy-scmessenger-relay.json` -> name it
     `SCMessengerRelayFreeTierOnly` -> Create.
   - Back on the user (step 2) -> Add permissions -> Attach policy directly ->
     select `SCMessengerRelayFreeTierOnly`.

4. **Set an AWS Budget (defense in depth, do this even though the policy
   restricts spend-causing actions):**
   - Billing console -> Budgets -> Create budget -> Cost budget.
   - Amount: $1.00 (a $0 budget can't alert - $1 catches literally any real
     spend before it compounds).
   - Alert threshold: 1% actual spend (fires almost immediately on any
     charge).
   - Email the alert to yourself. Optionally add a second budget at $5 as a
     "something is seriously wrong" tripwire.

5. **Generate the access key** (IAM console -> the user -> Security
   credentials -> Create access key -> "Application running outside AWS").
   Copy both the Access Key ID and Secret Access Key immediately - AWS only
   shows the secret once.

6. **Store credentials the same way every other lane's key is stored in this
   repo - never in chat, never committed:**
   ```
   # ~/.config/scmorc/aws.env  (outside the repo tree, gitignored by location)
   AWS_ACCESS_KEY_ID=AKIA...
   AWS_SECRET_ACCESS_KEY=...
   AWS_DEFAULT_REGION=us-east-1
   ```
   If you ever paste a key into a chat/session transcript by accident, treat
   it as burned and rotate it (IAM console -> the user -> Security
   credentials -> deactivate old key -> create a new one) - same policy as
   this repo's other credential-leak incidents on record.

7. **Tell the orchestrator it's ready.** Once `~/.config/scmorc/aws.env`
   exists, provisioning work (Terraform or plain `aws` CLI calls scoped to
   this policy) can proceed. Every `terraform plan` / `aws ec2 run-instances`
   equivalent should be reviewed before it executes against the real account,
   even though the IAM policy itself is the hard backstop.

## If you'd rather not hand over any AWS key at all

The orchestrator can still write and validate the Terraform/compose
configuration, and dry-run everything against the LOCAL docker rig
(`cloud/mesh/docker-compose.mesh-test.yml`) with zero cloud dependency - the
farm-topology simulation (B6) works entirely locally. Cloud deployment (B4)
is the only piece that needs real AWS access.

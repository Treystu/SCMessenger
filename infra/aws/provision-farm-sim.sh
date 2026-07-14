#!/usr/bin/env bash
# Provision the SCMessenger farm simulation EC2 instance (m7i-flex.large).
#
# Every action below is a plain, individually-reviewable `aws` CLI call - no
# Terraform state file, no black-box plan/apply. Read each command before
# running this for real; it is a DRY RUN (prints commands, executes nothing)
# unless you pass --apply.
#
# Prerequisites:
#   - AWS CLI v2 installed (https://aws.amazon.com/cli/)
#   - Credentials from infra/aws/set-aws-credentials.sh already written to
#     ~/.config/scmorc/aws.env (AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY,
#     AWS_DEFAULT_REGION=us-east-1).
#
# What this creates:
#   - One security group allowing tcp/22 (SSH, restricted to your current public
#     IP only), plus the P2P/relay ports: tcp/4001, udp/4001, tcp/4002, udp/4002.
#   - One m7i-flex.large EC2 instance, 30GB gp3 root volume, AL2023 x86_64 AMI.
#   - user_data that installs Docker, git, and Docker Compose, clones the public
#     SCMessenger repo, builds/starts the multi-network topology, and captures logs.
#
# Usage:
#   bash infra/aws/provision-farm-sim.sh           # dry run, prints every command
#   bash infra/aws/provision-farm-sim.sh --apply   # actually runs them

set -euo pipefail

APPLY=false
if [ "${1:-}" = "--apply" ]; then
    APPLY=true
fi

if [ -f "$HOME/.config/scmorc/aws.env" ]; then
    set -a
    # shellcheck source=/dev/null
    source "$HOME/.config/scmorc/aws.env"
    set +a
fi

REGION="${AWS_DEFAULT_REGION:-us-east-1}"
INSTANCE_TYPE="m7i-flex.large"
KEY_NAME="scmessenger-farm-sim-key"
SG_NAME="scmessenger-farm-sim-sg"
TAG_NAME="scmessenger-farm-relay"

run() {
    echo "+ $*"
    if $APPLY; then
        "$@"
    fi
}

echo "=== SCMessenger farm-sim provisioning ($( $APPLY && echo APPLY || echo DRY-RUN ), region=$REGION) ==="
echo

if ! command -v aws >/dev/null 2>&1; then
    echo "[ERROR] aws CLI not found. Install it, run 'aws configure' or rely on"
    echo "        ~/.config/scmorc/aws.env, then re-run this script."
    exit 1
fi

MY_IP="$(curl -s https://checkip.amazonaws.com || echo '0.0.0.0')"
echo "[INFO] Your current public IP (for SSH restriction): $MY_IP"
echo

# 1. Key pair (only if it doesn't already exist)
echo "--- Step 1: SSH key pair ---"
if aws ec2 describe-key-pairs --key-names "$KEY_NAME" --region "$REGION" >/dev/null 2>&1; then
    echo "[INFO] Key pair '$KEY_NAME' already exists, skipping."
else
    if $APPLY; then
        aws ec2 create-key-pair --key-name "$KEY_NAME" --region "$REGION" \
            --query 'KeyMaterial' --output text > "${KEY_NAME}.pem"
        chmod 400 "${KEY_NAME}.pem"
        echo "[NOTE] Saved private key to local file '${KEY_NAME}.pem' and set chmod 400."
    else
        echo "+ aws ec2 create-key-pair --key-name \"$KEY_NAME\" --region \"$REGION\" ..."
    fi
fi
echo

# 2. Security group - SSH from operator IP only + P2P ports
echo "--- Step 2: Security group ($SG_NAME) ---"
if aws ec2 describe-security-groups --group-names "$SG_NAME" --region "$REGION" >/dev/null 2>&1; then
    echo "[INFO] Security group '$SG_NAME' already exists, skipping creation."
else
    run aws ec2 create-security-group --group-name "$SG_NAME" --region "$REGION" \
        --description "SCMessenger farm sim - Restricted SSH + P2P ports" \
        --tag-specifications "ResourceType=security-group,Tags=[{Key=Name,Value=$SG_NAME}]"

    run aws ec2 authorize-security-group-ingress --group-name "$SG_NAME" --region "$REGION" \
        --protocol tcp --port 22 --cidr "${MY_IP}/32"
    run aws ec2 authorize-security-group-ingress --group-name "$SG_NAME" --region "$REGION" \
        --protocol tcp --port 4001 --cidr 0.0.0.0/0
    run aws ec2 authorize-security-group-ingress --group-name "$SG_NAME" --region "$REGION" \
        --protocol udp --port 4001 --cidr 0.0.0.0/0
    run aws ec2 authorize-security-group-ingress --group-name "$SG_NAME" --region "$REGION" \
        --protocol tcp --port 4002 --cidr 0.0.0.0/0
    run aws ec2 authorize-security-group-ingress --group-name "$SG_NAME" --region "$REGION" \
        --protocol udp --port 4002 --cidr 0.0.0.0/0
fi
echo

# 3. Latest Amazon Linux 2023 AMI
echo "--- Step 3: Resolve AMI ---"
AMI_ID="resolve:ssm:/aws/service/ami-amazon-linux-latest/al2023-ami-kernel-default-x86_64"
echo "[INFO] Using SSM parameter alias for latest Amazon Linux 2023: $AMI_ID"
echo

# 4. user_data: install Docker, git, clone SCMessenger public repo, run simulation
echo "--- Step 4: Preparing user-data (cloud-init) ---"
USER_DATA=$(cat <<'CLOUDINIT'
#!/bin/bash
set -euo pipefail

# 1. Update and install standard packages (docker, git)
dnf update -y
dnf install -y docker git

# 2. Install Docker Compose V2 plugin manually for AL2023
mkdir -p /usr/local/lib/docker/cli-plugins
curl -SL https://github.com/docker/compose/releases/download/v2.24.5/docker-compose-linux-x86_64 -o /usr/local/lib/docker/cli-plugins/docker-compose
chmod +x /usr/local/lib/docker/cli-plugins/docker-compose
ln -s /usr/local/lib/docker/cli-plugins/docker-compose /usr/local/bin/docker-compose || true

# 3. Start Docker daemon
systemctl enable --now docker

# 4. Clone the public SCMessenger repository
mkdir -p /opt
git clone https://github.com/Sovereign-Communication/SCMessenger.git /opt/SCMessenger

# 5. Build and launch the simulation (docker-compose-extended.yml)
cd /opt/SCMessenger/docker
docker compose -f docker-compose-extended.yml build --parallel
docker compose -f docker-compose-extended.yml --profile test up -d

# 6. Stream simulation logs to /var/log/farm-sim.log
nohup docker compose -f docker-compose-extended.yml --profile test logs -f > /var/log/farm-sim.log 2>&1 &
CLOUDINIT
)
echo "[INFO] cloud-init payload prepared."
echo

# 5. Launch the instance
echo "--- Step 5: Launch instance ($INSTANCE_TYPE) ---"
if $APPLY; then
    LAUNCH_OUT=$(aws ec2 run-instances \
        --image-id "$AMI_ID" \
        --instance-type "$INSTANCE_TYPE" \
        --key-name "$KEY_NAME" \
        --security-groups "$SG_NAME" \
        --region "$REGION" \
        --block-device-mappings 'DeviceName=/dev/xvda,Ebs={VolumeSize=30,VolumeType=gp3}' \
        --tag-specifications "ResourceType=instance,Tags=[{Key=Name,Value=$TAG_NAME}]" \
        --user-data "$USER_DATA" \
        --output json)

    INSTANCE_ID=$(echo "$LAUNCH_OUT" | jq -r '.Instances[0].InstanceId')
    echo "[INFO] Instance launched successfully."
    echo "[INFO] Instance ID: $INSTANCE_ID"
    echo "Waiting for instance to obtain public IP address..."

    PUBLIC_IP=""
    for i in {1..30}; do
        PUBLIC_IP=$(aws ec2 describe-instances --instance-ids "$INSTANCE_ID" --region "$REGION" \
            --query "Reservations[0].Instances[0].PublicIpAddress" --output text)
        if [ "$PUBLIC_IP" != "None" ] && [ ! -z "$PUBLIC_IP" ]; then
            break
        fi
        sleep 2
    done

    if [ "$PUBLIC_IP" = "None" ] || [ -z "$PUBLIC_IP" ]; then
        echo "[WARNING] Could not retrieve public IP within 60 seconds."
        PUBLIC_IP="<instance-public-ip>"
    fi
else
    echo "+ aws ec2 run-instances \\"
    echo "    --image-id \"$AMI_ID\" \\"
    echo "    --instance-type \"$INSTANCE_TYPE\" \\"
    echo "    --key-name \"$KEY_NAME\" \\"
    echo "    --security-groups \"$SG_NAME\" \\"
    echo "    --region \"$REGION\" \\"
    echo "    --block-device-mappings 'DeviceName=/dev/xvda,Ebs={VolumeSize=30,VolumeType=gp3}' \\"
    echo "    --tag-specifications \"ResourceType=instance,Tags=[{Key=Name,Value=$TAG_NAME}]\" \\"
    echo "    --user-data <CLOUD_INIT_SCRIPT>"
    INSTANCE_ID="i-xxxxxxxxxxxxxxxxx"
    PUBLIC_IP="xx.xx.xx.xx"
fi

echo
echo "=== Provisioning Complete ==="
echo "Instance ID: $INSTANCE_ID"
echo "Public IP:   $PUBLIC_IP"
echo
echo "To connect to the instance via SSH:"
echo "  ssh -i ${KEY_NAME}.pem ec2-user@${PUBLIC_IP}"
echo
echo "To tail the simulation logs:"
echo "  ssh -i ${KEY_NAME}.pem ec2-user@${PUBLIC_IP} 'tail -f /var/log/farm-sim.log'"
echo
echo "To check the Docker Compose containers status:"
echo "  ssh -i ${KEY_NAME}.pem ec2-user@${PUBLIC_IP} 'docker compose -f /opt/SCMessenger/docker/docker-compose-extended.yml --profile test ps'"
echo
echo "=== Teardown / Cleanup Note ==="
echo "To terminate the instance and clean up resources:"
echo "  aws ec2 terminate-instances --instance-ids $INSTANCE_ID --region $REGION"
echo "  aws ec2 delete-security-group --group-name $SG_NAME --region $REGION"
echo "  aws ec2 delete-key-pair --key-name $KEY_NAME --region $REGION"
if [ -f "${KEY_NAME}.pem" ]; then
    echo "  rm ${KEY_NAME}.pem"
fi
echo

#!/usr/bin/env bash
# Management script for SCMessenger farm simulation instance.
# Sources AWS credentials, resolves instance by tag, provides common operations.
#
# Usage:
#   ./infra/aws/farm-sim-manage.sh start     # Start the instance
#   ./infra/aws/farm-sim-manage.sh stop      # Stop the instance (preserves EBS)
#   ./infra/aws/farm-sim-manage.sh status    # Show instance info
#   ./infra/aws/farm-sim-manage.sh ssh       # Open SSH session
#   ./infra/aws/farm-sim-manage.sh logs      # Tail simulation logs
#   ./infra/aws/farm-sim-manage.sh keepawake on|off  # Hold/release idle shutdown
#   ./infra/aws/farm-sim-manage.sh iterate   # Quick rebuild loop
#   ./infra/aws/farm-sim-manage.sh teardown  # Terminate instance + cleanup

set -euo pipefail

# Source AWS credentials
if [ -f "$HOME/.config/scmorc/aws.env" ]; then
    set -a
    # shellcheck source=/dev/null
    source "$HOME/.config/scmorc/aws.env"
    set +a
else
    echo "[ERROR] AWS config not found at ~/.config/scmorc/aws.env"
    exit 1
fi

# Ensure coreutils are in PATH for Git Bash compatibility
export PATH="/usr/bin:/bin:$PATH"

REGION="${AWS_DEFAULT_REGION:-us-east-1}"
KEY_FILE="scmessenger-farm-sim-key.pem"
TAG_NAME="scmessenger-farm-relay"

# Get instance details (excluding terminated states)
get_instance_details() {
    aws ec2 describe-instances --region "$REGION" \
        --filters "Name=tag:Name,Values=$TAG_NAME" \
                  "Name=instance-state-name,Values=pending,running,stopping,stopped" \
        --query 'Reservations[].Instances[0].[InstanceId,State.Name,PublicIpAddress,InstanceType]' \
        --output text
}

# Wait for SSH to become available
wait_for_ssh() {
    local ip="$1"
    local timeout=60
    local count=0
    
    echo "Waiting for SSH availability..."
    while [ $count -lt $timeout ]; do
        if ssh -o StrictHostKeyChecking=no -o ConnectTimeout=5 -i "$KEY_FILE" "ec2-user@$ip" "exit" 2>/dev/null; then
            return 0
        fi
        sleep 5
        count=$((count + 1))
    done
    
    echo "[ERROR] SSH did not become available within $timeout seconds"
    return 1
}

# Main command dispatcher
case "${1:-}" in
    start)
        DETAILS=$(get_instance_details)
        if [ -z "$DETAILS" ]; then
            echo "[ERROR] No instance found with tag Name=$TAG_NAME"
            exit 1
        fi
        
        INSTANCE_ID=$(echo "$DETAILS" | awk '{print $1}')
        STATE=$(echo "$DETAILS" | awk '{print $2}')
        
        if [ "$STATE" = "running" ]; then
            echo "[INFO] Instance is already running"
            PUBLIC_IP=$(echo "$DETAILS" | awk '{print $3}')
            echo "Public IP: $PUBLIC_IP"
            exit 0
        fi
        
        echo "Starting instance $INSTANCE_ID..."
        aws ec2 start-instances --instance-ids "$INSTANCE_ID" --region "$REGION"
        
        echo "Waiting for instance to reach running state..."
        aws ec2 wait instance-running --instance-ids "$INSTANCE_ID" --region "$REGION"
        
        # Get updated details with IP
        UPDATED_DETAILS=$(get_instance_details)
        PUBLIC_IP=$(echo "$UPDATED_DETAILS" | awk '{print $3}')
        echo "Instance started. Public IP: $PUBLIC_IP"
        ;;
        
    stop)
        DETAILS=$(get_instance_details)
        if [ -z "$DETAILS" ]; then
            echo "[ERROR] No instance found with tag Name=$TAG_NAME"
            exit 1
        fi
        
        INSTANCE_ID=$(echo "$DETAILS" | awk '{print $1}')
        echo "Stopping instance $INSTANCE_ID..."
        aws ec2 stop-instances --instance-ids "$INSTANCE_ID" --region "$REGION"
        ;;
        
    status)
        DETAILS=$(get_instance_details)
        if [ -z "$DETAILS" ]; then
            echo "No instance found with tag Name=$TAG_NAME"
            exit 0
        fi
        
        INSTANCE_ID=$(echo "$DETAILS" | awk '{print $1}')
        STATE=$(echo "$DETAILS" | awk '{print $2}')
        PUBLIC_IP=$(echo "$DETAILS" | awk '{print $3}')
        INSTANCE_TYPE=$(echo "$DETAILS" | awk '{print $4}')
        
        echo "Instance ID:     $INSTANCE_ID"
        echo "State:           $STATE"
        echo "Public IP:       $PUBLIC_IP"
        echo "Instance Type:   $INSTANCE_TYPE"
        ;;
        
    ssh)
        DETAILS=$(get_instance_details)
        if [ -z "$DETAILS" ]; then
            echo "[ERROR] No instance found with tag Name=$TAG_NAME"
            exit 1
        fi
        
        PUBLIC_IP=$(echo "$DETAILS" | awk '{print $3}')
        if [ "$PUBLIC_IP" = "None" ] || [ -z "$PUBLIC_IP" ]; then
            echo "[ERROR] Instance has no public IP address"
            exit 1
        fi
        
        exec ssh -o StrictHostKeyChecking=no -i "$KEY_FILE" "ec2-user@$PUBLIC_IP"
        ;;
        
    logs)
        DETAILS=$(get_instance_details)
        if [ -z "$DETAILS" ]; then
            echo "[ERROR] No instance found with tag Name=$TAG_NAME"
            exit 1
        fi
        
        PUBLIC_IP=$(echo "$DETAILS" | awk '{print $3}')
        if [ "$PUBLIC_IP" = "None" ] || [ -z "$PUBLIC_IP" ]; then
            echo "[ERROR] Instance has no public IP address"
            exit 1
        fi
        
        ssh -o StrictHostKeyChecking=no -i "$KEY_FILE" "ec2-user@$PUBLIC_IP" "sudo tail -f /var/log/farm-sim.log"
        ;;
        
    keepawake)
        if [ -z "${2:-}" ]; then
            echo "[ERROR] Usage: $0 keepawake on|off"
            exit 1
        fi
        
        DETAILS=$(get_instance_details)
        if [ -z "$DETAILS" ]; then
            echo "[ERROR] No instance found with tag Name=$TAG_NAME"
            exit 1
        fi
        
        PUBLIC_IP=$(echo "$DETAILS" | awk '{print $3}')
        if [ "$PUBLIC_IP" = "None" ] || [ -z "$PUBLIC_IP" ]; then
            echo "[ERROR] Instance has no public IP address"
            exit 1
        fi
        
        case "$2" in
            on)
                ssh -o StrictHostKeyChecking=no -i "$KEY_FILE" "ec2-user@$PUBLIC_IP" "sudo touch /var/run/farm-keepawake"
                echo "Keepawake enabled"
                ;;
            off)
                ssh -o StrictHostKeyChecking=no -i "$KEY_FILE" "ec2-user@$PUBLIC_IP" "sudo rm -f /var/run/farm-keepawake"
                echo "Keepawake disabled"
                ;;
            *)
                echo "[ERROR] Usage: $0 keepawake on|off"
                exit 1
                ;;
        esac
        ;;
        
    iterate)
        DETAILS=$(get_instance_details)
        if [ -z "$DETAILS" ]; then
            echo "[ERROR] No instance found with tag Name=$TAG_NAME"
            exit 1
        fi
        
        INSTANCE_ID=$(echo "$DETAILS" | awk '{print $1}')
        STATE=$(echo "$DETAILS" | awk '{print $2}')
        PUBLIC_IP=$(echo "$DETAILS" | awk '{print $3}')
        
        # Start instance if stopped
        if [ "$STATE" = "stopped" ]; then
            echo "Starting instance $INSTANCE_ID..."
            aws ec2 start-instances --instance-ids "$INSTANCE_ID" --region "$REGION"
            aws ec2 wait instance-running --instance-ids "$INSTANCE_ID" --region "$REGION"
            
            # Refresh IP after start
            UPDATED_DETAILS=$(get_instance_details)
            PUBLIC_IP=$(echo "$UPDATED_DETAILS" | awk '{print $3}')
        fi
        
        if [ "$PUBLIC_IP" = "None" ] || [ -z "$PUBLIC_IP" ]; then
            echo "[ERROR] Instance has no public IP address"
            exit 1
        fi
        
        # Wait for SSH
        wait_for_ssh "$PUBLIC_IP" || exit 1
        
        # Execute rebuild sequence
        ssh -o StrictHostKeyChecking=no -i "$KEY_FILE" "ec2-user@$PUBLIC_IP" \
            "sudo touch /var/run/farm-keepawake && cd /opt/SCMessenger && git pull && cd docker && docker compose -f docker-compose-extended.yml --profile test up -d --build && sudo rm -f /var/run/farm-keepawake"
        ;;
        
    teardown)
        DETAILS=$(get_instance_details)
        if [ -z "$DETAILS" ]; then
            echo "[ERROR] No instance found with tag Name=$TAG_NAME"
            exit 1
        fi
        
        INSTANCE_ID=$(echo "$DETAILS" | awk '{print $1}')
        echo "Terminating instance $INSTANCE_ID..."
        aws ec2 terminate-instances --instance-ids "$INSTANCE_ID" --region "$REGION"
        
        # Wait for instance termination to avoid DependencyViolation errors
        aws ec2 wait instance-terminated --instance-ids "$INSTANCE_ID" --region "$REGION"
        
        echo "Deleting security group..."
        aws ec2 delete-security-group --group-name "scmessenger-farm-sim-sg" --region "$REGION" || true
        
        echo "Deleting key pair..."
        aws ec2 delete-key-pair --key-name "scmessenger-farm-sim-key" --region "$REGION" || true
        
        if [ -f "$KEY_FILE" ]; then
            echo "Removing local key file..."
            rm -f "$KEY_FILE"
        fi
        
        echo "Teardown complete"
        ;;
        
    *)
        echo "Usage: $0 {start|stop|status|ssh|logs|keepawake on|off|iterate|teardown}"
        exit 1
        ;;
esac
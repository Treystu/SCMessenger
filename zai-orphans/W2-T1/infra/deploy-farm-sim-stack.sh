#!/bin/bash
# Deploy Farm-Sim Stack via CloudFormation
#
# BLOCKED as of 2026-07-18: the scmessenger-relay-orchestrator IAM user has
# cloudformation:ValidateTemplate and cloudformation:ListStacks explicitly
# denied (confirmed via live API probes), so CreateStack will fail too --
# CloudFormation is not usable with the current IAM policy. Use
# infra/ec2/launch-farm-sim.sh instead, which deploys the same 7-node
# topology via direct EC2 API calls (RunInstances/CreateSecurityGroup are
# both allowed) into the account's existing default VPC. Revisit this
# script only after cloudformation:* is added to that user's policy.
#
# Usage: ./deploy-farm-sim-stack.sh [action] [region]
# Actions: validate, create, update, delete, status
# Regions: us-east-1 (default), us-west-2, etc.

set -e

ACTION=${1:-create}
REGION=${2:-us-east-1}
STACK_NAME="scmessenger-farm-sim"
TEMPLATE_FILE="infra/cloudformation/farm-sim-stack.yaml"

echo "[INFO] Farm-Sim CloudFormation Stack Manager"
echo "[INFO] Action: $ACTION"
echo "[INFO] Region: $REGION"
echo "[INFO] Stack: $STACK_NAME"
echo ""

case $ACTION in
  validate)
    echo "[VALIDATE] Checking template syntax..."
    aws cloudformation validate-template \
      --template-body file://$TEMPLATE_FILE \
      --region $REGION
    echo "[OK] Template is syntactically valid"
    ;;

  create)
    echo "[VALIDATE] Checking template syntax before deploy..."
    aws cloudformation validate-template \
      --template-body file://$TEMPLATE_FILE \
      --region $REGION > /dev/null
    echo "[OK] Template valid"

    echo "[CREATE] Deploying new stack..."
    aws cloudformation create-stack \
      --stack-name $STACK_NAME \
      --template-body file://$TEMPLATE_FILE \
      --parameters ParameterKey=KeyName,ParameterValue=scmessenger-farm-sim-key \
      --region $REGION \
      --capabilities CAPABILITY_IAM

    echo "[WAIT] Waiting for stack creation..."
    aws cloudformation wait stack-create-complete \
      --stack-name $STACK_NAME \
      --region $REGION

    echo "[OK] Stack created successfully!"
    echo ""
    echo "[OUTPUTS] Instance IPs:"
    aws cloudformation describe-stacks \
      --stack-name $STACK_NAME \
      --region $REGION \
      --query 'Stacks[0].Outputs[*].[OutputKey,OutputValue]' \
      --output table
    ;;

  update)
    echo "[UPDATE] Updating existing stack..."
    aws cloudformation update-stack \
      --stack-name $STACK_NAME \
      --template-body file://$TEMPLATE_FILE \
      --region $REGION \
      --capabilities CAPABILITY_IAM 2>/dev/null || echo "[INFO] No changes to deploy"

    echo "[WAIT] Waiting for stack update..."
    aws cloudformation wait stack-update-complete \
      --stack-name $STACK_NAME \
      --region $REGION 2>/dev/null || true

    echo "[OK] Stack updated!"
    ;;

  delete)
    echo "[DELETE] DELETING stack (this is destructive)..."
    read -p "Are you sure? Type 'yes' to confirm: " confirm
    if [ "$confirm" != "yes" ]; then
      echo "[CANCEL] Deletion cancelled"
      exit 0
    fi

    aws cloudformation delete-stack \
      --stack-name $STACK_NAME \
      --region $REGION

    echo "[WAIT] Waiting for stack deletion..."
    aws cloudformation wait stack-delete-complete \
      --stack-name $STACK_NAME \
      --region $REGION

    echo "[OK] Stack deleted!"
    ;;

  status)
    echo "[STATUS] Stack information:"
    aws cloudformation describe-stacks \
      --stack-name $STACK_NAME \
      --region $REGION \
      --query 'Stacks[0].[StackName,StackStatus,CreationTime]' \
      --output table

    echo ""
    echo "[OUTPUTS] Instance IPs:"
    aws cloudformation describe-stacks \
      --stack-name $STACK_NAME \
      --region $REGION \
      --query 'Stacks[0].Outputs[*].[OutputKey,OutputValue]' \
      --output table
    ;;

  *)
    echo "[ERROR] Unknown action: $ACTION"
    echo "Usage: $0 [validate|create|update|delete|status] [region]"
    exit 1
    ;;
esac

echo ""
echo "[OK] Operation complete!"

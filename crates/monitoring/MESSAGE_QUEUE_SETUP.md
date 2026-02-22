# Message Queue Setup Guide

This guide explains how to set up AWS SQS for whale movement event publishing.

## Overview

The monitoring engine publishes whale movement events to an AWS SQS queue. These events are consumed by downstream services (AI analysis, notifications, etc.) for processing.

## Prerequisites

- AWS Account
- AWS CLI installed and configured
- Appropriate IAM permissions to create SQS queues

## Setup Instructions

### 1. Create SQS Queue

**For Development Environment:**

```bash
# Create a development queue
aws sqs create-queue \
  --queue-name whale-movements-dev \
  --attributes '{
    "MessageRetentionPeriod": "345600",
    "VisibilityTimeout": "300",
    "ReceiveMessageWaitTimeSeconds": "20"
  }' \
  --tags Key=Environment,Value=development \
  --profile dev
```

**For Production Environment:**

```bash
# Create a production queue with DLQ
aws sqs create-queue \
  --queue-name whale-movements-prod-dlq \
  --attributes '{
    "MessageRetentionPeriod": "1209600"
  }' \
  --tags Key=Environment,Value=production \
  --profile prod

# Get the DLQ ARN
DLQ_ARN=$(aws sqs get-queue-attributes \
  --queue-url $(aws sqs get-queue-url --queue-name whale-movements-prod-dlq --query 'QueueUrl' --output text --profile prod) \
  --attribute-names QueueArn \
  --query 'Attributes.QueueArn' \
  --output text \
  --profile prod)

# Create production queue with DLQ
aws sqs create-queue \
  --queue-name whale-movements-prod \
  --attributes '{
    "MessageRetentionPeriod": "345600",
    "VisibilityTimeout": "300",
    "ReceiveMessageWaitTimeSeconds": "20",
    "RedrivePolicy": "{\"deadLetterTargetArn\":\"'$DLQ_ARN'\",\"maxReceiveCount\":\"3\"}"
  }' \
  --tags Key=Environment,Value=production \
  --profile prod
```

### 2. Configure IAM Permissions

**Least-Privilege Policy for Monitoring Service (Send Only):**

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "sqs:SendMessage",
        "sqs:GetQueueAttributes",
        "sqs:GetQueueUrl"
      ],
      "Resource": [
        "arn:aws:sqs:us-east-1:123456789012:whale-movements-dev",
        "arn:aws:sqs:us-east-1:123456789012:whale-movements-prod"
      ]
    }
  ]
}
```

**For Consumer Services (Receive and Delete):**

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "sqs:ReceiveMessage",
        "sqs:DeleteMessage",
        "sqs:GetQueueAttributes",
        "sqs:GetQueueUrl"
      ],
      "Resource": [
        "arn:aws:sqs:us-east-1:123456789012:whale-movements-dev",
        "arn:aws:sqs:us-east-1:123456789012:whale-movements-prod"
      ]
    }
  ]
}
```

### 3. Configure Application

**Option A: Using AWS CLI Profiles (Development)**

```bash
# Configure AWS credentials
aws configure --profile dev
# Enter your AWS Access Key ID, Secret Access Key, and region

# Set environment variable
export AWS_PROFILE=dev
export AWS_SQS_QUEUE_URL=https://sqs.us-east-1.amazonaws.com/123456789012/whale-movements-dev
```

**Option B: Using Environment Variables**

```bash
export AWS_ACCESS_KEY_ID=your_access_key
export AWS_SECRET_ACCESS_KEY=your_secret_key
export AWS_REGION=us-east-1
export AWS_SQS_QUEUE_URL=https://sqs.us-east-1.amazonaws.com/123456789012/whale-movements-dev
```

**Option C: Using IAM Roles (Production - Recommended)**

When running on EC2, ECS, or Lambda, use IAM roles instead of credentials:

1. Create an IAM role with the least-privilege policy above
2. Attach the role to your EC2 instance, ECS task, or Lambda function
3. The AWS SDK will automatically use the role credentials
4. Only set the queue URL in your .env file:

```bash
AWS_SQS_QUEUE_URL=https://sqs.us-east-1.amazonaws.com/123456789012/whale-movements-prod
```

### 4. Update .env File

Add to your `.env` file:

```env
# For development
AWS_SQS_QUEUE_URL=https://sqs.us-east-1.amazonaws.com/123456789012/whale-movements-dev

# For production
# AWS_SQS_QUEUE_URL=https://sqs.us-east-1.amazonaws.com/123456789012/whale-movements-prod
```

## Message Format

The monitoring service publishes messages in the following JSON format:

```json
{
  "movement_id": "550e8400-e29b-41d4-a716-446655440000",
  "whale_address": "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU",
  "transaction_signature": "5j7s6NiJS3JAkvgkoc18WVAsiSaci2pxB2A6ueCJP4tprA2TFg9wSyTLeYouxPBJEMzJinENTkpA52YStRW5Dia7",
  "movement_type": "SELL",
  "token_mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
  "amount": "1000000",
  "percent_of_position": 10.5,
  "detected_at": "2024-01-15T10:30:00Z",
  "affected_user_ids": [
    "650e8400-e29b-41d4-a716-446655440001",
    "650e8400-e29b-41d4-a716-446655440002"
  ]
}
```

### Message Attributes

The following SQS message attributes are included for filtering:

- `whale_address` (String): The whale's Solana address
- `movement_type` (String): "BUY" or "SELL"
- `token_mint` (String): The token mint address

## Monitoring and Troubleshooting

### Check Queue Status

```bash
# Get queue attributes
aws sqs get-queue-attributes \
  --queue-url YOUR_QUEUE_URL \
  --attribute-names All \
  --profile dev

# Monitor message count
aws sqs get-queue-attributes \
  --queue-url YOUR_QUEUE_URL \
  --attribute-names ApproximateNumberOfMessages,ApproximateNumberOfMessagesNotVisible \
  --profile dev
```

### View Messages (Development Only)

```bash
# Receive messages without deleting (for debugging)
aws sqs receive-message \
  --queue-url YOUR_QUEUE_URL \
  --max-number-of-messages 1 \
  --visibility-timeout 0 \
  --profile dev
```

### Purge Queue (Development Only - Use with Caution)

```bash
# WARNING: This deletes all messages in the queue
aws sqs purge-queue \
  --queue-url YOUR_QUEUE_URL \
  --profile dev
```

## Security Best Practices

1. **Use IAM Roles**: In production, always use IAM roles instead of access keys
2. **Least Privilege**: Grant only the permissions needed (send for producers, receive for consumers)
3. **Separate Queues**: Use different queues for dev, staging, and production
4. **Enable Encryption**: Use SQS server-side encryption (SSE) for sensitive data
5. **Monitor Access**: Enable CloudTrail logging for SQS API calls
6. **Dead Letter Queue**: Configure DLQ for production to capture failed messages

## Cost Optimization

- **Long Polling**: Enabled by default (ReceiveMessageWaitTimeSeconds=20) to reduce empty receives
- **Batch Operations**: The monitoring service uses batch sends when possible (up to 10 messages)
- **Message Retention**: Set to 4 days (345600 seconds) to balance cost and reliability

## Integration with Monitoring Service

The monitoring service automatically:
1. Detects whale movements that exceed the 5% threshold
2. Stores the movement in PostgreSQL
3. Queries affected users from the database
4. Publishes the event to SQS with affected user IDs
5. Logs success/failure for monitoring

If the message queue is not configured, the monitoring service will:
- Continue to store movements in the database
- Log a debug message indicating no queue is configured
- Not fail the operation

## Next Steps

After setting up the message queue:
1. Configure the AI analysis service to consume from this queue
2. Set up CloudWatch alarms for queue depth and age
3. Implement consumer services for notifications and trading
4. Monitor queue metrics in CloudWatch

## Support

For issues with:
- AWS SQS setup: Refer to [AWS SQS Documentation](https://docs.aws.amazon.com/sqs/)
- IAM permissions: Refer to [AWS IAM Documentation](https://docs.aws.amazon.com/iam/)
- Application integration: Check the monitoring service logs

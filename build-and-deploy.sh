#!/bin/bash

set -e

AWS_REGION="eu-central-1"  

AWS_ACCOUNT_ID=$(aws sts get-caller-identity --query Account --output text)
ECR_BASE_URI="$AWS_ACCOUNT_ID.dkr.ecr.$AWS_REGION.amazonaws.com"

build_and_push() {
    local container_name=$1
    local dockerfile=$2
    local ecr_repo="$container_name"
    local ecr_uri="$ECR_BASE_URI/$ecr_repo"

    echo "=== Processing $container_name ==="
    
    echo "Checking ECR repository..."
    if ! aws ecr describe-repositories --repository-names $ecr_repo &>/dev/null; then
        echo "Creating ECR repository..."
        aws ecr create-repository --repository-name $ecr_repo
    fi

    echo "Building Docker image..."
    docker buildx build --platform linux/amd64 -f $dockerfile -t $ecr_repo .

    aws ecr get-login-password --region $AWS_REGION | docker login --username AWS --password-stdin $ecr_uri
    docker tag $ecr_repo:latest $ecr_uri:latest
    docker push $ecr_uri:latest

    echo "Done! Image pushed to: $ecr_uri:latest"
    echo "----------------------------------------"
}

build_and_push "stress" "Dockerfile"

echo "All containers have been built and pushed to ECR!" 
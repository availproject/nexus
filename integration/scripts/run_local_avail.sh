#!/bin/bash

mkdir -p avail-node

AVAIL_VERSION="toufeeq/scalable-da-2"
IMAGE_NAME="availnode"
IMAGE_TAG="latest"

if command -v docker &> /dev/null; then
    echo "Docker is installed. Continuing..."
    docker --version
else
    echo "Docker is NOT installed. Please install docker"
fi

if docker image inspect "$IMAGE_NAME:$IMAGE_TAG" > /dev/null 2>&1; then
    echo "Image $IMAGE_NAME:$IMAGE_TAG exists"
else
    echo "Image $IMAGE_NAME:$IMAGE_TAG does NOT exist. Building...."
    docker build -t $IMAGE_NAME -f ./integration/scripts/Dockerfile .
fi

echo "Running Avail dev chain"
mkdir -p output
docker run --rm -p 30334:30334 -p 9944:9944 -v ./output:/output availnode --dev --rpc-methods=unsafe --unsafe-rpc-external --rpc-cors=all &

#!/bin/bash

curr_path=`pwd`
script_full_name=$BASH_SOURCE
cd `dirname $script_full_name`
script_path=`pwd`
cd $curr_path

source $script_path/build.env

PROJ_NAME=mould
IMAGE_PATH=$REGISTRY$IMAGE_NAMESPACE/$PROJ_NAME

error_exit() {
	msg=$1
	echo $msg
	exit 1
}

echo "Start to upload image artifact."
echo $REGISTRY_PASSWORD | docker login -u $REGISTRY_USER --password-stdin $REGISTRY || error_exit "Login to registry failed!"
docker push $IMAGE_PATH:$IMAGE_TAG || error_exit "Push image failed!"
docker push $IMAGE_PATH:latest || error_exit "Push image failed!"
echo image path: $IMAGE_PATH:$IMAGE_TAG
echo "Build succeed."

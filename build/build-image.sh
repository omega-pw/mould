#!/bin/bash

curr_path=`pwd`
script_full_name=$BASH_SOURCE
cd `dirname $script_full_name`
script_path=`pwd`
cd $curr_path

source $script_path/build.env

WORKSPACE=$script_path/..
PROJ_NAME=mould
IMAGE_NAME=$PROJ_NAME-image-x86_64-$IMAGE_TAG.tar.gz
CONFIG_FILE=/config/config.json5
IMAGE_PATH=$REGISTRY$IMAGE_NAMESPACE/$PROJ_NAME

error_exit() {
	msg=$1
	echo $msg
	exit 1
}

echo "Start to package image."
IMAGE_ROOT=$script_path/image-root
cp $WORKSPACE/target/release/$PROJ_NAME $IMAGE_ROOT/
chmod +x $IMAGE_ROOT/lib/*
chmod +x $IMAGE_ROOT/lib64/*
chmod +x $IMAGE_ROOT/extensions/*
chmod +x $IMAGE_ROOT/$PROJ_NAME
cd $IMAGE_ROOT/
tar -zcvf $script_path/$IMAGE_NAME * || error_exit "Package image failed!"
cd $script_path/

echo "Start to build image."
docker import -c "CMD [\"/$PROJ_NAME\", \"$CONFIG_FILE\"]" $script_path/$IMAGE_NAME $IMAGE_PATH:$IMAGE_TAG || error_exit "Build image failed!"
docker tag $IMAGE_PATH:$IMAGE_TAG $IMAGE_PATH:latest || error_exit "Add image tag failed!"

rm -f $script_path/$IMAGE_NAME

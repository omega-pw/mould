#!/bin/bash

curr_path=`pwd`
script_full_name=$BASH_SOURCE
cd `dirname $script_full_name`
script_path=`pwd`
cd $curr_path

source $script_path/build.env

WORKSPACE=$script_path/..
PROJ_NAME=mould
GIT_COMMITID=`git rev-parse --short HEAD`
BIN_NAME=$PROJ_NAME-linux-x86_64-$GIT_COMMITID
LATEST_BIN_NAME=$PROJ_NAME-linux-x86_64-latest
IMAGE_NAME=$PROJ_NAME-image-x86_64-$GIT_COMMITID.tar.gz
LATEST_IMAGE_NAME=$PROJ_NAME-image-x86_64-latest.tar.gz
CONFIG_FILE=/config/config.json5
IMAGE_PATH=$REGISTRY$IMAGE_NAMESPACE/$PROJ_NAME

error_exit() {
	msg=$1
	echo $msg
	exit 1
}

is_ubuntu() {
    if [ -f /etc/os-release ]; then
        source /etc/os-release
        if [ "$ID" = "ubuntu" ]; then
            return 0
        else
            return 1
        fi
    else
        return 1
    fi
}
if is_ubuntu; then
    PKG_MGR=apt
else
    PKG_MGR=yum
fi

$PKG_MGR install dos2unix -y

apt-get update
apt-get install -y protobuf-compiler libprotobuf-dev

dos2unix $script_path/build-mould.sh
chmod +x $script_path/build-mould.sh
$script_path/build-mould.sh || error_exit "Build mould failed!"

dos2unix $script_path/build-extensions.sh
chmod +x $script_path/build-extensions.sh
$script_path/build-extensions.sh || error_exit "Build extensions failed!"

ls $WORKSPACE/target/release/

exit 0

echo "Start to package image."
IMAGE_ROOT=$script_path/image-root
cp $WORKSPACE/target/release/$PROJ_NAME $IMAGE_ROOT/
chmod +x $IMAGE_ROOT/lib64/*
chmod +x $IMAGE_ROOT/extensions/*
chmod +x $IMAGE_ROOT/$PROJ_NAME
cd $IMAGE_ROOT/
tar -zcvf $script_path/$IMAGE_NAME * || error_exit "Package image failed!"
cd $script_path/

echo "Start to build image."
docker import -c "CMD [\"/$PROJ_NAME\", \"$CONFIG_FILE\"]" $script_path/$IMAGE_NAME $IMAGE_PATH:$GIT_COMMITID || error_exit "Build image failed!"
docker tag $IMAGE_PATH:$GIT_COMMITID $IMAGE_PATH:latest || error_exit "Add image tag failed!"

rm -f $script_path/$IMAGE_NAME

echo "Start to upload image artifact."
echo $REGISTRY_PASSWORD | docker login -u $REGISTRY_USER --password-stdin $REGISTRY || error_exit "Login to registry failed!"
docker push $IMAGE_PATH:$GIT_COMMITID || error_exit "Push image failed!"
docker push $IMAGE_PATH:latest || error_exit "Push image failed!"
echo image path: $IMAGE_PATH:$GIT_COMMITID
echo "Build succeed."

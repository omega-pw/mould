#!/bin/bash

curr_path=`pwd`
script_full_name=$BASH_SOURCE
cd `dirname $script_full_name`
script_path=`pwd`
cd $curr_path

source $script_path/build.env

WORKSPACE=$script_path/..
EXTENSIONS_SRC_DIR=$WORKSPACE/extensions
EXTENSIONS_DIST_DIR=$script_path/image-root/extensions
RELEASE_DIR=$WORKSPACE/target/release

error_exit() {
	msg=$1
	echo $msg
	exit 1
}

cd $EXTENSIONS_SRC_DIR/etcd/
echo "Start to build etcd extension."
cargo build --release || error_exit "Build etcd extension failed!"

cd $EXTENSIONS_SRC_DIR/kubernetes/
echo "Start to build kubernetes extension."
cargo build --release || error_exit "Build kubernetes extension failed!"

cd $EXTENSIONS_SRC_DIR/mysql/
echo "Start to build mysql extension."
cargo build --release || error_exit "Build mysql extension failed!"

cd $EXTENSIONS_SRC_DIR/nacos/
echo "Start to build nacos extension."
cargo build --release || error_exit "Build nacos extension failed!"

cd $EXTENSIONS_SRC_DIR/postgresql/
echo "Start to build postgresql extension."
cargo build --release || error_exit "Build postgresql extension failed!"

cd $EXTENSIONS_SRC_DIR/s3/
echo "Start to build s3 extension."
cargo build --release || error_exit "Build s3 extension failed!"

cd $EXTENSIONS_SRC_DIR/server/
echo "Start to build server extension."
cargo build --release || error_exit "Build server extension failed!"

# clear dist first
rm -rf $EXTENSIONS_DIST_DIR
mkdir -p $EXTENSIONS_DIST_DIR
cp $RELEASE_DIR/libetcd.so $EXTENSIONS_DIST_DIR
cp $RELEASE_DIR/libkubernetes.so $EXTENSIONS_DIST_DIR
cp $RELEASE_DIR/libmysql.so $EXTENSIONS_DIST_DIR
cp $RELEASE_DIR/libnacos.so $EXTENSIONS_DIST_DIR
cp $RELEASE_DIR/libpostgresql.so $EXTENSIONS_DIST_DIR
cp $RELEASE_DIR/libs3.so $EXTENSIONS_DIST_DIR
cp $RELEASE_DIR/libserver.so $EXTENSIONS_DIST_DIR
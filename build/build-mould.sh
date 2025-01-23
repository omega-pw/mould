#!/bin/bash

curr_path=`pwd`
script_full_name=$BASH_SOURCE
cd `dirname $script_full_name`
script_path=`pwd`
cd $curr_path

WORKSPACE=$script_path/..
WEB_CLIENT_ROOT=$WORKSPACE/client

error_exit() {
	msg=$1
	echo $msg
	exit 1
}

cd $WEB_CLIENT_ROOT

# clear dist first
rm -rf $WEB_CLIENT_ROOT/dist

echo "Start to install dependencies."
npm install || error_exit "Install dependencies failed!"
echo "Start to build package."
npm run build || error_exit "Build package failed!"

cd $WEB_CLIENT_ROOT/dist || error_exit "Build code failed!"
git log -n 1 > $WEB_CLIENT_ROOT/dist/version.txt || error_exit "Add version information failed!"
cp -rf $WEB_CLIENT_ROOT/dist/* $WORKSPACE/server/static/ || error_exit "Copy client code to static directory failed!"

cd $WORKSPACE/server

echo "Start to build binary."
cargo build --release || error_exit "Build binary failed!"

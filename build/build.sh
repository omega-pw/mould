#!/bin/bash

curr_path=`pwd`
script_full_name=$BASH_SOURCE
cd `dirname $script_full_name`
script_path=`pwd`
cd $curr_path

source $script_path/build.env

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

dos2unix $script_path/build-mould.sh
chmod +x $script_path/build-mould.sh
$script_path/build-mould.sh || error_exit "Build mould failed!"

dos2unix $script_path/build-extensions.sh
chmod +x $script_path/build-extensions.sh
$script_path/build-extensions.sh || error_exit "Build extensions failed!"

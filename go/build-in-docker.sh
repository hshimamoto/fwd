#!/bin/bash

name=`basename $PWD`
if [ -e project ]; then
	pname=`cat project`
	if [ "$pname" != "" ]; then
		name=$pname
	fi
fi
uid=`id -u`
gid=`id -g`
work=/go/src/$name
cname=gobuild-$name

docker run -it --rm -v $PWD:$work -w $work -e HOME=$work --name $cname --hostname $cname -u $uid:$gid golang bash

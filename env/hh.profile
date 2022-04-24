#!/usr/bin/env bash

HHSCRIPT="$(dirname "${BASH_SOURCE}")/../target/release/hh"

function hs(){
	n=5
	if [ "$1" ] && [ -z "${1//[0-9]}" ];
	then
		n=`expr $1 \* 5`
	fi
	history $n|$HHSCRIPT $*
}

set -a
HISTTIMEFORMAT="%F %T "
HISTIGNORE="ls:ll:cd:pwd:bg:fg:history:cat:les:less:more:exit:clear"
alias hh="hs"
set +a
